// XXX maybe the session-to-db migration should move into the store module.
pub mod migrations;

mod groupv2;
mod linked_devices;
mod profile;
mod profile_upload;

pub use self::groupv2::*;
pub use self::linked_devices::*;
use self::migrations::MigrationCondVar;
pub use self::profile::*;
pub use self::profile_upload::*;
use libsignal_service::proto::data_message::Quote;
pub use libsignal_service::provisioning::{VerificationCodeResponse, VerifyAccountResponse};
pub use libsignal_service::push_service::DeviceInfo;
use zkgroup::profiles::ProfileKey;

use super::profile_refresh::OutdatedProfileStream;
use crate::actor::SessionActor;
use crate::gui::StorageReady;
use crate::millis_to_naive_chrono;
use crate::model::DeviceModel;
use crate::platform::QmlApp;
use crate::store::{orm, Storage};
use actix::prelude::*;
use anyhow::Context;
use chrono::prelude::*;
use futures::prelude::*;
use libsignal_service::configuration::SignalServers;
use libsignal_service::content::sync_message::Request as SyncRequest;
use libsignal_service::content::DataMessageFlags;
use libsignal_service::content::{
    sync_message, AttachmentPointer, ContentBody, DataMessage, GroupContextV2, Metadata,
    TypingMessage,
};
use libsignal_service::prelude::protocol::*;
use libsignal_service::prelude::*;
use libsignal_service::proto::typing_message::Action;
use libsignal_service::proto::{receipt_message, ReceiptMessage};
use libsignal_service::provisioning::ProvisioningManager;
use libsignal_service::push_service::{
    AccountAttributes, DeviceCapabilities, DeviceId, DEFAULT_DEVICE_ID,
};
use libsignal_service::sender::AttachmentSpec;
use libsignal_service::websocket::SignalWebSocket;
use libsignal_service::AccountManager;
use libsignal_service_actix::prelude::*;
use mime_classifier::{ApacheBugFlag, LoadContext, MimeClassifier, NoSniffFlag};
use phonenumber::PhoneNumber;
use qmeta_async::with_executor;
use qmetaobject::prelude::*;
use std::collections::HashSet;
use std::fs::remove_file;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

// Maximum theoretical TypingMessage send rate
const TM_MAX_RATE: f32 = 24.0; // messages per minute
const TM_CACHE_CAPACITY: f32 = 2.0; // 2 min
const TM_CACHE_TRESHOLD: f32 = 1.75; // 1 min 45 sec

#[derive(actix::Message, Debug)]
#[rtype(result = "()")]
pub struct QueueMessage {
    pub session_id: i32,
    pub message: String,
    pub attachment: String,
    pub quote: i32,
}

#[derive(Message)]
#[rtype(result = "()")]
/// Enqueue a message on socket by message id.
///
/// This will construct a DataMessage, and pass it to a DeliverMessage
pub struct SendMessage(pub i32);

/// Delivers a constructed T: Into<ContentBody> to a session.
#[derive(Message)]
#[rtype(result = "Result<(), anyhow::Error>")]
struct DeliverMessage<T> {
    content: T,
    timestamp: u64,
    online: bool,
    session_id: i32,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
/// Send a notification that we're typing on a certain session.
pub struct SendTypingNotification {
    pub session_id: i32,
    pub is_start: bool,
}

#[derive(Message)]
#[rtype(result = "()")]
struct AttachmentDownloaded {
    session_id: i32,
    message_id: i32,
}

#[derive(Message)]
#[rtype(result = "usize")]
pub struct CompactDb(usize);

#[derive(Message)]
#[rtype(result = "()")]
/// Reset a session with a certain recipient
pub struct EndSession(pub i32);

#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct ClientWorker {
    base: qt_base_class!(trait QObject),
    messageReceived: qt_signal!(sid: i32, mid: i32),
    messageReactionReceived: qt_signal!(sid: i32, mid: i32),
    attachmentDownloaded: qt_signal!(sid: i32, mid: i32),
    messageReceipt: qt_signal!(sid: i32, mid: i32),
    notifyMessage: qt_signal!(
        sid: i32,
        mid: i32,
        sessionName: QString,
        senderName: QString,
        senderIdentifier: QString,
        senderUuid: QString,
        message: QString,
        isGroup: bool
    ),
    promptResetPeerIdentity: qt_signal!(),
    messageSent: qt_signal!(sid: i32, mid: i32, message: QString),
    messageNotSent: qt_signal!(sid: i32, mid: i32),
    // FIXME: Rust "r#type" to Qt "type" doesn't work
    proofRequested: qt_signal!(token: QString, r#type: QString),
    proofCaptchaResult: qt_signal!(success: bool),

    send_typing_notification: qt_method!(fn(&self, id: i32, is_start: bool)),
    submit_proof_captcha: qt_method!(fn(&self, token: String, response: String)),

    connected: qt_property!(bool; NOTIFY connectedChanged),
    connectedChanged: qt_signal!(),

    actor: Option<Addr<ClientActor>>,
    session_actor: Option<Addr<SessionActor>>,
    device_model: Option<QObjectBox<DeviceModel>>,

    // Linked device management
    link_device: qt_method!(fn(&self, tsurl: String)),
    unlink_device: qt_method!(fn(&self, id: i64)),
    reload_linked_devices: qt_method!(fn(&self)),
    compact_db: qt_method!(fn(&self)),

    refresh_group_v2: qt_method!(fn(&self, session_id: usize)),

    delete_file: qt_method!(fn(&self, file_name: String)),

    refresh_profile: qt_method!(fn(&self, recipient_id: i32)),
    upload_profile: qt_method!(
        fn(&self, given_name: String, family_name: String, about: String, emoji: String)
    ),
}

/// ClientActor keeps track of the connection state.
pub struct ClientActor {
    inner: QObjectBox<ClientWorker>,

    migration_state: MigrationCondVar,

    credentials: Option<ServiceCredentials>,
    local_addr: Option<ServiceAddress>,
    storage: Option<Storage>,
    ws: Option<SignalWebSocket>,
    // XXX The cipher should be behind a Mutex.
    // By considering the session that needs to be accessed,
    // we could lock only a single session to enforce serialized access.
    // That's a lot of code though, and it should probably happen *inside* the ServiceCipher
    // instead.
    // Having ServiceCipher implement `Clone` is imo. a problem, now that everything is `async`.
    // Putting in behind a Mutex is a lot of work now though,
    // especially considering this should be *internal* to ServiceCipher.
    cipher: Option<
        ServiceCipher<
            crate::store::Storage,
            crate::store::Storage,
            crate::store::Storage,
            crate::store::Storage,
            crate::store::Storage,
            rand::rngs::ThreadRng,
        >,
    >,
    config: std::sync::Arc<crate::config::SignalConfig>,

    typing_message_timestamps: HashSet<u64>,

    start_time: DateTime<Local>,

    outdated_profile_stream_handle: Option<SpawnHandle>,
}

fn whisperfish_device_capabilities() -> DeviceCapabilities {
    DeviceCapabilities {
        announcement_group: false,
        gv2: true,
        storage: false,
        gv1_migration: true,
        sender_key: true,
        change_number: false,
        gift_badges: false,
        stories: false,
    }
}

impl ClientActor {
    pub fn new(
        app: &mut QmlApp,
        session_actor: Addr<SessionActor>,
        config: std::sync::Arc<crate::config::SignalConfig>,
    ) -> Result<Self, anyhow::Error> {
        let inner = QObjectBox::new(ClientWorker::default());
        let device_model = QObjectBox::new(DeviceModel::default());
        app.set_object_property("ClientWorker".into(), inner.pinned());
        app.set_object_property("DeviceModel".into(), device_model.pinned());

        inner.pinned().borrow_mut().session_actor = Some(session_actor);
        inner.pinned().borrow_mut().device_model = Some(device_model);

        let typing_message_timestamps: HashSet<u64> =
            HashSet::with_capacity((TM_CACHE_CAPACITY * TM_MAX_RATE) as _);

        Ok(Self {
            inner,
            migration_state: MigrationCondVar::new(),
            credentials: None,
            local_addr: None,
            storage: None,
            cipher: None,
            ws: None,
            config,

            typing_message_timestamps,

            start_time: Local::now(),

            outdated_profile_stream_handle: None,
        })
    }

    fn uuid(&self) -> Option<Uuid> {
        self.credentials.as_ref().and_then(|c| c.uuid)
    }

    fn user_agent(&self) -> String {
        crate::user_agent()
    }

    fn unauthenticated_service(&self) -> AwcPushService {
        let service_cfg = self.service_cfg();
        AwcPushService::new(service_cfg, None, self.user_agent())
    }

    fn authenticated_service_with_credentials(
        &self,
        credentials: ServiceCredentials,
    ) -> AwcPushService {
        let service_cfg = self.service_cfg();

        AwcPushService::new(service_cfg, Some(credentials), self.user_agent())
    }

    /// Panics if no authentication credentials are set.
    fn authenticated_service(&self) -> AwcPushService {
        self.authenticated_service_with_credentials(self.credentials.clone().unwrap())
    }

    fn message_sender(
        &self,
    ) -> MessageSender<
        AwcPushService,
        crate::store::Storage,
        crate::store::Storage,
        crate::store::Storage,
        crate::store::Storage,
        crate::store::Storage,
        rand::rngs::ThreadRng,
    > {
        let storage = self.storage.clone().unwrap();
        let service = self.authenticated_service();
        MessageSender::new(
            self.ws.clone().unwrap(),
            service,
            self.cipher.clone().unwrap(),
            rand::thread_rng(),
            storage.clone(),
            storage,
            self.local_addr.unwrap(),
            self.config.get_device_id(),
        )
    }

    fn service_cfg(&self) -> ServiceConfiguration {
        // XXX: read the configuration files!
        SignalServers::Production.into()
    }

    pub fn handle_needs_delivery_receipt(
        &mut self,
        ctx: &mut <Self as Actor>::Context,
        message: &DataMessage,
        metadata: &Metadata,
    ) -> Option<()> {
        let uuid = metadata.sender.uuid.to_string();
        let storage = self.storage.as_mut().expect("storage");
        let recipient = storage.fetch_recipient(None, Some(&uuid))?;
        let session = storage.fetch_or_insert_session_by_recipient_id(recipient.id);

        let content = ReceiptMessage {
            r#type: Some(receipt_message::Type::Delivery as _),
            timestamp: vec![message.timestamp?],
        };

        ctx.notify(DeliverMessage {
            content,
            timestamp: Utc::now().timestamp_millis() as u64,
            // XXX Session ID is artificial here.
            session_id: session.id,
            online: false,
        });

        Some(())
    }

    /// Process incoming message from Signal
    ///
    /// This was `MessageHandler` in Go.
    ///
    /// TODO: consider putting this as an actor `Handle<>` implementation instead.
    pub fn handle_message(
        &mut self,
        ctx: &mut <Self as Actor>::Context,
        // XXX: remove this argument
        source_e164: Option<String>,
        source_uuid: Option<String>,
        msg: &DataMessage,
        is_sync_sent: bool,
        metadata: &Metadata,
    ) -> Option<i32> {
        let timestamp = metadata.timestamp;
        let settings = crate::config::SettingsBridge::default();

        let storage = self.storage.as_mut().expect("storage");
        let sender_recipient = if source_e164.is_some() || source_uuid.is_some() {
            Some(storage.merge_and_fetch_recipient(
                source_e164.as_deref(),
                source_uuid.as_deref(),
                crate::store::TrustLevel::Certain,
            ))
        } else {
            None
        };

        if msg.flags() & DataMessageFlags::EndSession as u32 != 0 {
            let storage = storage.clone();
            if let Some(svc) = sender_recipient
                .as_ref()
                .and_then(|r| r.to_service_address())
            {
                actix::spawn(async move {
                    if let Err(e) = storage.delete_all_sessions(&svc).await {
                        log::error!("End session requested, but could not end session: {:?}", e);
                    }
                });
            } else {
                log::error!("Requested session reset but no service address associated");
            }
        }

        if msg.flags() & DataMessageFlags::ExpirationTimerUpdate as u32 != 0 {
            // XXX Update expiration timer and notify UI
        }

        if msg.flags() & DataMessageFlags::ProfileKeyUpdate as u32 != 0 {
            // XXX Update profile key (which happens just below); don't insert this message.
        }

        if (source_e164.is_some() || source_uuid.is_some()) && !is_sync_sent {
            if let Some(key) = msg.profile_key.as_deref() {
                let (recipient, was_updated) = storage.update_profile_key(
                    source_e164.as_deref(),
                    source_uuid.as_deref(),
                    key,
                    crate::store::TrustLevel::Certain,
                );
                if was_updated {
                    ctx.notify(RefreshProfile::ByRecipientId(recipient.id));
                }
            }
        }

        if msg.flags() & DataMessageFlags::ProfileKeyUpdate as u32 != 0 {
            log::info!("Message was ProfileKeyUpdate; not inserting.");
        }

        let alt_body = if let Some(reaction) = &msg.reaction {
            if let Some((message, session)) = storage.process_reaction(
                &sender_recipient
                    .clone()
                    .or_else(|| storage.fetch_self_recipient())
                    .expect("sender or self-sent"),
                msg,
                reaction,
            ) {
                log::info!("Reaction saved for message {}/{}", session.id, message.id);
                self.inner
                    .pinned()
                    .borrow_mut()
                    .messageReactionReceived(session.id, message.id);
            } else {
                log::error!("Could not find a message for this reaction. Dropping.");
                log::warn!(
                    "This probably indicates out-of-order receipt delivery. Please upvote issue #260"
                );
            }
            None
        } else if msg.flags() & DataMessageFlags::ExpirationTimerUpdate as u32 != 0 {
            Some(format!("Expiration timer has been changed ({:?} seconds), but unimplemented in Whisperfish.", msg.expire_timer))
        } else if let Some(GroupContextV2 {
            group_change: Some(ref _group_change),
            ..
        }) = msg.group_v2
        {
            Some(format!(
                "Group changed by {}",
                source_e164
                    .as_deref()
                    .or(source_uuid.as_deref())
                    .unwrap_or("nobody")
            ))
        } else if !msg.attachments.is_empty() {
            log::trace!("Received an attachment without body, replacing with empty text.");
            Some("".into())
        } else if msg.sticker.is_some() {
            log::warn!("Received a sticker, but inserting empty message.");
            Some("This is a sticker, but stickers are currently unsupported.".into())
        } else if msg.payment.is_some()
            || msg.delete.is_some()
            || msg.group_call_update.is_some()
            || !msg.contact.is_empty()
        {
            Some("Unimplemented message type".into())
        } else {
            None
        };

        let body = msg.body.clone().or(alt_body);
        let text = if let Some(body) = body {
            body
        } else {
            log::debug!("Message without (alt) body, not inserting");
            return None;
        };

        let new_message = crate::store::NewMessage {
            source_e164,
            source_uuid,
            text,
            flags: msg.flags() as i32,
            outgoing: is_sync_sent,
            is_unidentified: metadata.unidentified_sender,
            sent: is_sync_sent,
            timestamp: millis_to_naive_chrono(if is_sync_sent && timestamp > 0 {
                timestamp
            } else {
                msg.timestamp()
            }),
            has_attachment: !msg.attachments.is_empty(),
            mime_type: None,  // Attachments are further handled asynchronously
            received: false,  // This is set true by a receipt handler
            session_id: None, // Canary value checked later
            attachment: None,
            is_read: is_sync_sent,
            quote_timestamp: msg.quote.as_ref().and_then(|x| x.id),
        };

        let group = if let Some(group) = msg.group_v2.as_ref() {
            let mut key_stack = [0u8; zkgroup::GROUP_MASTER_KEY_LEN];
            key_stack.clone_from_slice(group.master_key.as_ref().expect("group message with key"));
            let key = GroupMasterKey::new(key_stack);
            let secret = GroupSecretParams::derive_from_master_key(key);

            let store_v2 = crate::store::GroupV2 {
                secret,
                revision: group.revision(),
            };

            // XXX handle group.group_change like a real client
            if let Some(_change) = group.group_change.as_ref() {
                log::warn!("We're not handling raw group changes yet. Let's trigger a group refresh for now.");
                ctx.notify(RequestGroupV2Info(store_v2.clone(), key_stack));
            } else if !storage.group_v2_exists(&store_v2) {
                log::info!(
                    "We don't know this group. We'll request it's structure from the server."
                );
                ctx.notify(RequestGroupV2Info(store_v2.clone(), key_stack));
            }

            Some(storage.fetch_or_insert_session_by_group_v2(&store_v2))
        } else {
            None
        };

        let (message, session) = storage.process_message(new_message, group);

        if settings.get_bool("attachment_log") && !msg.attachments.is_empty() {
            log::trace!("Logging message to the attachment log");
            // XXX Sync code, but it's not the only sync code in here...
            let mut log = self.attachment_log();

            writeln!(
                log,
                "[{}] {:?} for message ID {}",
                Utc::now(),
                msg,
                message.id
            )
            .expect("write to the attachment log");
        }

        if settings.get_bool("save_attachments") {
            for attachment in &msg.attachments {
                // Go used to always set has_attachment and mime_type, but also
                // in this method, as well as the generated path.
                // We have this function that returns a filesystem path, so we can
                // set it ourselves.
                let dir = settings.get_string("attachment_dir");
                let dest = Path::new(&dir);

                ctx.notify(FetchAttachment {
                    session_id: session.id,
                    message_id: message.id,
                    dest: dest.to_path_buf(),
                    ptr: attachment.clone(),
                });
            }
        }

        self.inner
            .pinned()
            .borrow_mut()
            .messageReceived(session.id, message.id);

        // XXX If from ourselves, skip
        if settings.get_bool("enable_notify") && !is_sync_sent && !session.is_muted {
            let session_name: &str = match &session.r#type {
                orm::SessionType::GroupV1(group) => &group.name,
                orm::SessionType::GroupV2(group) => &group.name,
                orm::SessionType::DirectMessage(recipient) => recipient.name(),
            };

            self.inner.pinned().borrow_mut().notifyMessage(
                session.id,
                message.id,
                session_name.into(),
                sender_recipient
                    .as_ref()
                    .map(|x| x.name().into())
                    .unwrap_or_else(|| "".into()),
                sender_recipient
                    .as_ref()
                    .map(|x| x.e164_or_uuid().into())
                    .unwrap_or_else(|| "".into()),
                sender_recipient
                    .map(|x| x.uuid().into())
                    .unwrap_or_else(|| "".into()),
                message.text.as_deref().unwrap_or("").into(),
                session.is_group(),
            );
        }
        Some(message.id)
    }

    fn handle_sync_request(&mut self, meta: Metadata, req: SyncRequest) {
        use sync_message::request::Type;
        log::trace!("Processing sync request {:?}", req.r#type());

        let local_addr = self.local_addr.unwrap();
        let storage = self.storage.clone().unwrap();
        let mut sender = self.message_sender();

        actix::spawn(async move {
            match req.r#type() {
                Type::Unknown => {
                    log::warn!("Unknown sync request from {:?}:{}. Please upgrade Whisperfish or file an issue.", meta.sender, meta.sender_device);
                    return Ok(());
                }
                Type::Contacts => {
                    use libsignal_service::sender::ContactDetails;
                    // In fact, we should query for registered contacts instead of sessions here.
                    // https://gitlab.com/whisperfish/whisperfish/-/issues/133
                    let recipients: Vec<orm::Recipient> = {
                        use crate::schema::recipients::dsl::*;
                        use diesel::prelude::*;
                        let mut db = storage.db();
                        recipients.load(&mut *db)?
                    };

                    let contacts = recipients.into_iter().map(|recipient| {
                            ContactDetails {
                                // XXX: expire timer from dm session
                                number: recipient.e164.clone(),
                                uuid: recipient.uuid.clone(),
                                name: recipient.profile_joined_name.clone(),
                                profile_key: recipient.profile_key,
                                // XXX other profile stuff
                                ..Default::default()
                            }
                    });

                    sender.send_contact_details(&local_addr, None, contacts, false, true).await?;
                },
                Type::Groups => {
                    use libsignal_service::sender::GroupDetails;
                    let sessions = storage.fetch_group_sessions();

                    let groups = sessions.into_iter().map(|session| {
                        let group = session.unwrap_group_v1();
                        let members = storage.fetch_group_members_by_group_v1_id(&group.id);
                        GroupDetails {
                            name: Some(group.name.clone()),
                            members_e164: members.iter().filter_map(|(_member, recipient)| recipient.e164.clone()).collect(),
                            // XXX: update proto file and add more.
                            // members: members.iter().filter_map(|(_member, recipient)| Member {e164: recipient.e164}).collect(),
                            // avatar, active?, color, ..., many cool things to add here!
                            // Tagging issue #204
                            ..Default::default()
                        }
                    });

                    sender.send_groups_details(&local_addr, None, groups, false).await?;
                }
                Type::Blocked => {
                    anyhow::bail!("Unimplemented {:?}", req.r#type());
                }
                Type::Configuration => {
                    anyhow::bail!("Unimplemented {:?}", req.r#type());
                }
                Type::Keys => {
                    anyhow::bail!("Unimplemented {:?}", req.r#type());
                }
                Type::PniIdentity => {
                    anyhow::bail!("Unimplemented {:?}", req.r#type());
                },
            };

            Ok::<_, anyhow::Error>(())
        }.map(|v| if let Err(e) = v {log::error!("{:?} in handle_sync_request()", e)}));
    }

    fn process_receipt(&mut self, msg: &Envelope) {
        let millis = msg.timestamp();

        // If the receipt timestamp matches a cached TypingMessage timestamp,
        // stop processing, since there's no such message in database.
        if self.typing_message_timestamps.contains(&millis) {
            log::info!("Received TypingMessage receipt: {}", millis);
            return;
        }

        log::info!("Received receipt: {}", millis);

        let storage = self.storage.as_mut().expect("storage initialized");
        let source = msg.source_address();

        let timestamp = millis_to_naive_chrono(millis);
        log::trace!(
            "Marking message from {:?} at {} ({}) as received.",
            source,
            timestamp,
            millis
        );
        if let Some((sess, msg)) = storage.mark_message_received(source.uuid, timestamp, None) {
            self.inner
                .pinned()
                .borrow_mut()
                .messageReceipt(sess.id, msg.id)
        }
    }

    fn process_envelope(
        &mut self,
        Content { body, metadata }: Content,
        ctx: &mut <Self as Actor>::Context,
    ) {
        let storage = self.storage.clone().expect("storage initialized");

        match body {
            ContentBody::NullMessage(_message) => {
                log::trace!("Ignoring NullMessage");
            }
            ContentBody::DataMessage(message) => {
                let uuid = metadata.sender.uuid;
                let message_id = self.handle_message(
                    ctx,
                    None,
                    Some(uuid.to_string()),
                    &message,
                    false,
                    &metadata,
                );
                if metadata.needs_receipt {
                    if let Some(_message_id) = message_id {
                        self.handle_needs_delivery_receipt(ctx, &message, &metadata);
                    }
                }
                if !metadata.unidentified_sender && message_id.is_some() {
                    // TODO: if the contact should have our profile key already, send it again.
                    //       if the contact should not yet have our profile key, this is ok, and we
                    //       should offer the user a message request.
                    //       Cfr. MessageContentProcessor, grep for handleNeedsDeliveryReceipt.
                    log::warn!("Received an unsealed message from {:?}. Assert that they have our profile key.", metadata.sender);
                }
            }
            ContentBody::SynchronizeMessage(message) => {
                let mut handled = false;
                if let Some(sent) = message.sent {
                    handled = true;
                    log::trace!("Sync sent message");
                    // These are messages sent through a paired device.

                    if let Some(message) = sent.message {
                        self.handle_message(
                            ctx,
                            // Empty string mainly when groups,
                            // but maybe needs a check. TODO
                            sent.destination_e164,
                            sent.destination_uuid,
                            &message,
                            true,
                            &metadata,
                        );
                    } else {
                        log::warn!(
                            "Dropping sync-sent without message; probably Stories related: {:?}",
                            sent
                        );
                    }
                }
                if let Some(request) = message.request {
                    handled = true;
                    log::trace!("Sync request message");
                    self.handle_sync_request(metadata, request);
                }
                if !message.read.is_empty() {
                    handled = true;
                    log::trace!("Sync read message");
                    for read in &message.read {
                        // XXX: this should probably not be based on ts alone.
                        let ts = read.timestamp();
                        let source = read.sender_uuid();
                        // Signal uses timestamps in milliseconds, chrono has nanoseconds
                        let ts = millis_to_naive_chrono(ts);
                        log::trace!(
                            "Marking message from {} at {} ({}) as read.",
                            source,
                            ts,
                            read.timestamp()
                        );
                        if let Some((sess, msg)) = storage.mark_message_read(ts) {
                            self.inner
                                .pinned()
                                .borrow_mut()
                                .messageReceipt(sess.id, msg.id)
                        } else {
                            log::warn!("Could not mark as received!");
                        }
                    }
                }
                if let Some(fetch) = message.fetch_latest {
                    handled = true;
                    match fetch.r#type() {
                        sync_message::fetch_latest::Type::Unknown => {
                            log::warn!("Sync FetchLatest with unknown type")
                        }
                        sync_message::fetch_latest::Type::LocalProfile => {
                            log::trace!("Scheduling local profile refresh");
                            ctx.notify(RefreshOwnProfile { force: true });
                        }
                        sync_message::fetch_latest::Type::StorageManifest => {
                            // XXX
                            log::warn!("Unimplemented: synchronize fetch request StorageManifest")
                        }
                        sync_message::fetch_latest::Type::SubscriptionStatus => {
                            log::warn!(
                                "Unimplemented: synchronize fetch request SubscriptionStatus"
                            )
                        }
                    }
                }
                if !handled {
                    log::warn!("Sync message without known sync type");
                }
            }
            ContentBody::TypingMessage(typing) => {
                log::info!("{:?} is typing.", metadata.sender);
                let res = self
                    .inner
                    .pinned()
                    .borrow()
                    .session_actor
                    .as_ref()
                    .expect("session actor running")
                    .try_send(crate::actor::TypingNotification {
                        typing,
                        sender: metadata.sender,
                    });
                if let Err(e) = res {
                    log::error!("Could not send typing notification to SessionActor: {}", e);
                }
            }
            ContentBody::ReceiptMessage(receipt) => {
                log::info!("{:?} received a message.", metadata.sender);
                // XXX dispatch on receipt.type
                for &ts in &receipt.timestamp {
                    // Signal uses timestamps in milliseconds, chrono has nanoseconds
                    if let Some((sess, msg)) = storage.mark_message_received(
                        metadata.sender.uuid,
                        millis_to_naive_chrono(ts),
                        None,
                    ) {
                        self.inner
                            .pinned()
                            .borrow_mut()
                            .messageReceipt(sess.id, msg.id)
                    } else {
                        log::warn!("Could not mark {} as received!", ts);
                    }
                }
            }
            ContentBody::CallMessage(_call) => {
                log::info!("{:?} is calling.", metadata.sender);
            }
        }
    }

    fn attachment_log(&self) -> std::fs::File {
        std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(self.storage.as_ref().unwrap().path().join(format!(
                "attachments-{}.log",
                self.start_time.format("%Y-%m-%d_%H-%M")
            )))
            .expect("open attachment log")
    }
}

impl Actor for ClientActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.inner.pinned().borrow_mut().actor = Some(ctx.address());
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.inner.pinned().borrow_mut().actor = Some(ctx.address());

        self.inner.pinned().borrow_mut().connected = false;
        self.inner.pinned().borrow().connectedChanged();
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct FetchAttachment {
    session_id: i32,
    message_id: i32,
    dest: PathBuf,
    ptr: AttachmentPointer,
}

impl Handler<FetchAttachment> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;

    /// Downloads the attachment in the background and registers it in the database.
    /// Saves the given attachment into a random-generated path. Saves the path in the database.
    ///
    /// This was a Message method in Go
    fn handle(
        &mut self,
        fetch: FetchAttachment,
        ctx: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let FetchAttachment {
            session_id,
            message_id,
            dest,
            ptr,
        } = fetch;

        let client_addr = ctx.address();

        let mut service = self.unauthenticated_service();
        let mut storage = self.storage.clone().unwrap();

        // Sailfish and/or Rust needs "image/jpg" and some others need coaching
        // before taking a wild guess
        let mut ext = match ptr.content_type() {
            "text/plain" => "txt",
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/jpg" => "jpg",
            "text/x-signal-plain" => "txt",
            "application/x-signal-view-once" => "bin",
            other => mime_guess::get_mime_extensions_str(other)
                .expect("Could not find mime")
                .first()
                .unwrap(),
        };

        let ptr2 = ptr.clone();
        Box::pin(
            async move {
                use futures::io::AsyncReadExt;
                use libsignal_service::attachment_cipher::*;

                let mut stream = loop {
                    let r = service.get_attachment(&ptr).await;
                    match r {
                        Ok(stream) => break stream,
                        Err(ServiceError::Timeout { .. }) => {
                            log::warn!("get_attachment timed out, retrying")
                        }
                        Err(e) => return Err(e.into()),
                    }
                };
                log::info!("Downloading attachment");

                // We need the whole file for the crypto to check out 😢
                let actual_len = ptr.size.unwrap();
                let mut ciphertext = Vec::with_capacity(actual_len as usize);
                let stream_len = stream
                    .read_to_end(&mut ciphertext)
                    .await
                    .expect("streamed attachment") as u32;

                let key_material = ptr.key();
                assert_eq!(
                    key_material.len(),
                    64,
                    "key material for attachments is ought to be 64 bytes"
                );
                let mut key = [0u8; 64];
                key.copy_from_slice(key_material);
                decrypt_in_place(key, &mut ciphertext).expect("attachment decryption");

                // Signal puts exponentially increasing padding at the end
                // to prevent some distinguishing attacks, so it has to be truncated.
                if stream_len > actual_len {
                    log::info!(
                        "The attachment contains {} bytes of padding",
                        (stream_len - actual_len)
                    );
                    log::info!("Truncating from {} to {} bytes", stream_len, actual_len);
                    ciphertext.truncate(actual_len as usize);
                }

                // Signal Desktop sometimes sends a JPEG image with .png extension,
                // so double check the received .png image, and rename it if necessary.
                if ext == "png" {
                    log::trace!("Checking for JPEG with .png extension...");
                    let classifier = MimeClassifier::new();
                    let computed_type = classifier.classify(
                        LoadContext::Image,
                        NoSniffFlag::Off,
                        ApacheBugFlag::Off,
                        &None,
                        &ciphertext as &[u8],
                    );
                    if computed_type == mime::IMAGE_JPEG {
                        log::info!("Received JPEG file with .png suffix, renaming to .jpg");
                        ext = "jpg";
                    }
                }

                let attachment_path = storage.save_attachment(&dest, ext, &ciphertext).await?;

                storage.register_attachment(
                    message_id,
                    ptr,
                    attachment_path.to_str().expect("attachment path utf-8"),
                );
                client_addr
                    .send(AttachmentDownloaded {
                        session_id,
                        message_id,
                    })
                    .await?;
                Ok(())
            }
            .into_actor(self)
            .map(move |r: Result<(), anyhow::Error>, act, _ctx| {
                // Synchronise on the actor, to log the error to attachment.log
                if let Err(e) = r {
                    let e = format!(
                        "Error fetching attachment for message with ID `{}` {:?}: {:?}",
                        message_id, ptr2, e
                    );
                    log::error!("{} in handle()", e);
                    let mut log = act.attachment_log();
                    if let Err(e) = writeln!(log, "{}", e) {
                        log::error!("Could not write error to error log: {}", e);
                    }
                }
            }),
        )
    }
}

impl Handler<QueueMessage> for ClientActor {
    type Result = ();

    fn handle(&mut self, msg: QueueMessage, ctx: &mut Self::Context) -> Self::Result {
        log::trace!("MessageActor::handle({:?})", msg);
        let storage = self.storage.as_mut().unwrap();

        let has_attachment = !msg.attachment.is_empty();
        let self_recipient = storage
            .fetch_self_recipient()
            .expect("self recipient set when sending");
        let session = storage
            .fetch_session_by_id(msg.session_id)
            .expect("existing session when sending");

        let quote = if msg.quote >= 0 {
            Some(
                storage
                    .fetch_message_by_id(msg.quote)
                    .expect("existing quote id"),
            )
        } else {
            None
        };

        let (msg, _session) = storage.process_message(
            crate::store::NewMessage {
                session_id: Some(msg.session_id),
                source_e164: self_recipient.e164,
                source_uuid: self_recipient.uuid,
                text: msg.message,
                timestamp: chrono::Utc::now().naive_utc(),
                has_attachment,
                mime_type: if has_attachment {
                    Some(
                        mime_guess::from_path(&msg.attachment)
                            .first_or_octet_stream()
                            .essence_str()
                            .into(),
                    )
                } else {
                    None
                },
                attachment: if has_attachment {
                    Some(msg.attachment)
                } else {
                    None
                },
                flags: 0,
                outgoing: true,
                received: false,
                sent: false,
                is_read: true,
                is_unidentified: false,
                quote_timestamp: quote.map(|msg| msg.server_timestamp.timestamp_millis() as u64),
            },
            Some(session),
        );

        ctx.notify(SendMessage(msg.id));
    }
}

impl Handler<SendMessage> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;

    // Equiv of worker/send.go
    fn handle(&mut self, SendMessage(mid): SendMessage, ctx: &mut Self::Context) -> Self::Result {
        log::info!("ClientActor::SendMessage({:?})", mid);
        let mut sender = self.message_sender();
        let storage = self.storage.as_mut().unwrap();
        let msg = storage.fetch_augmented_message(mid).unwrap();
        let session = storage.fetch_session_by_id(msg.session_id).unwrap();
        let session_id = session.id;

        if msg.sent_timestamp.is_some() {
            log::warn!("Message already sent, refusing to retransmit.");
            return Box::pin(async {}.into_actor(self).map(|_, _, _| ()));
        }

        let self_recipient = storage.fetch_self_recipient();
        log::trace!("Sending for session: {:?}", session);
        log::trace!("Sending message: {:?}", msg.inner);

        let storage = storage.clone();
        let addr = ctx.address();
        Box::pin(
            async move {
                if let orm::SessionType::GroupV1(_group) = &session.r#type {
                    // FIXME
                    log::error!("Cannot send to Group V1 anymore.");
                }
                let group_v2 = if let orm::SessionType::GroupV2(group) = &session.r#type {
                    let master_key = hex::decode(&group.master_key).expect("hex group id in db");
                    Some(GroupContextV2 {
                        master_key: Some(master_key),
                        revision: Some(group.revision as u32),
                        group_change: None,
                    })
                } else {
                    None
                };

                let timestamp = msg.server_timestamp.timestamp_millis() as u64;

                let quote = msg
                    .quote_id
                    .and_then(|quote_id| storage.fetch_augmented_message(quote_id))
                    .map(|quoted_message| {
                        if !quoted_message.attachments > 0 {
                            log::warn!("Quoting attachments is incomplete.  Here be dragons.");
                        }
                        let quote_sender = quoted_message
                            .sender_recipient_id
                            .and_then(|x| storage.fetch_recipient_by_id(x));

                        Quote {
                            id: Some(quoted_message.server_timestamp.timestamp_millis() as u64),
                            author_uuid: quote_sender.as_ref().and_then(|r| r.uuid.clone()),
                            text: quoted_message.text.clone(),

                            ..Default::default()
                        }
                    });

                let mut content = DataMessage {
                    body: msg.text.clone(),
                    flags: if msg.flags != 0 {
                        Some(msg.flags as _)
                    } else {
                        None
                    },
                    timestamp: Some(timestamp),
                    // XXX: depends on the features in the message!
                    required_protocol_version: Some(0),
                    group_v2,

                    profile_key: self_recipient.and_then(|r| r.profile_key),
                    quote,
                    ..Default::default()
                };

                let attachments = storage.fetch_attachments_for_message(msg.id);

                for attachment in &attachments {
                    let attachment_path = attachment
                        .attachment_path
                        .clone() // Clone for the spawn_blocking below
                        .expect("attachment path when uploading");
                    let contents =
                        tokio::task::spawn_blocking(move || std::fs::read(attachment_path))
                            .await
                            .context("threadpool")?
                            .context("reading attachment")?;
                    let attachment_path = attachment.attachment_path.as_deref().unwrap();
                    let spec = AttachmentSpec {
                        content_type: match mime_guess::from_path(attachment_path).first() {
                            Some(mime) => mime.essence_str().into(),
                            None => String::from("application/octet-stream"),
                        },
                        length: contents.len(),
                        file_name: Path::new(attachment_path)
                            .file_name()
                            .map(|f| f.to_string_lossy().into_owned()),
                        preview: None,
                        voice_note: Some(attachment.is_voice_note),
                        borderless: Some(attachment.is_borderless),
                        width: attachment.width.map(|x| x as u32),
                        height: attachment.height.map(|x| x as u32),
                        caption: None,
                        blur_hash: None,
                    };
                    let ptr = match sender.upload_attachment(spec, contents).await {
                        Ok(v) => v,
                        Err(e) => {
                            anyhow::bail!("Failed to upload attachment: {}", e);
                        }
                    };
                    content.attachments.push(ptr);
                }

                let res = addr
                    .send(DeliverMessage {
                        content,
                        online: false,
                        timestamp,
                        session_id,
                    })
                    .await?;

                match res {
                    Ok(()) => {
                        storage.dequeue_message(mid, chrono::Utc::now().naive_utc());
                        Ok((session.id, mid, msg.inner.text))
                    }
                    Err(e) => {
                        storage.fail_message(mid);

                        match &e.downcast_ref() {
                            Some(MessageSenderError::ProofRequired { token, options }) => {
                                // Note: 'recaptcha' can refer to reCAPTCHA or hCaptcha
                                let recaptcha = String::from("recaptcha");

                                if options.contains(&recaptcha) {
                                    addr.send(ProofRequired {
                                        token: token.to_owned(),
                                        r#type: recaptcha,
                                    })
                                    .await
                                    .expect("deliver captcha required");
                                } else {
                                    log::warn!("Rate limit proof requested, but type 'recaptcha' wasn't available!");
                                }
                            },
                            Some(MessageSenderError::NotFound { uuid }) => {
                                let uuid_s = uuid.to_string();
                                log::warn!("Recipient not found, removing device sessions {}", uuid_s);
                                let mut num = storage.delete_all_sessions(&ServiceAddress { uuid: *uuid }).await?;
                                log::trace!("Removed {} device session(s)", num);
                                num = storage.mark_recipient_registered(&uuid_s, false);
                                log::trace!("Marked {} recipient(s) as unregistered", num);
                                anyhow::bail!(MessageSenderError::NotFound { uuid: uuid.to_owned() });
                            },
                            _ => (),
                        };

                        Err(e)
                    }
                }
            }
            .into_actor(self)
            .map(move |res, act, _ctx| {
                match res {
                    Ok((sid, mid, message)) => {
                        act.inner.pinned().borrow().messageSent(
                            sid,
                            mid,
                            message.unwrap_or_default().into(),
                        );
                    }
                    Err(e) => {
                        log::error!("Sending message: {}", e);
                        act.inner.pinned().borrow().messageNotSent(session_id, mid);
                        if let Some(MessageSenderError::NotFound { uuid: _ }) = e.downcast_ref() {
                            // Handles session-is-not-a-group ok
                            act.inner
                                .pinned()
                                .borrow()
                                .refresh_group_v2(session_id as _);
                        }
                    }
                };
            }),
        )
    }
}

impl Handler<EndSession> for ClientActor {
    type Result = ();

    fn handle(&mut self, EndSession(id): EndSession, ctx: &mut Self::Context) -> Self::Result {
        log::trace!("ClientActor::EndSession(recipient_id = {})", id);

        let storage = self.storage.as_mut().unwrap();
        let recipient = storage
            .fetch_recipient_by_id(id)
            .expect("existing recipient id");

        let (msg, _session) = storage.process_message(
            crate::store::NewMessage {
                session_id: None,
                source_e164: recipient.e164,
                source_uuid: recipient.uuid,
                text: "[Whisperfish] Reset secure session".into(),
                timestamp: chrono::Utc::now().naive_utc(),
                has_attachment: false,
                mime_type: None,
                attachment: None,
                flags: DataMessageFlags::EndSession.into(),
                outgoing: true,
                received: false,
                sent: false,
                is_read: true,
                is_unidentified: false,
                quote_timestamp: None,
            },
            None,
        );
        ctx.notify(SendMessage(msg.id));
    }
}

impl Handler<SendTypingNotification> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(
        &mut self,
        SendTypingNotification {
            session_id,
            is_start,
        }: SendTypingNotification,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        log::info!("ClientActor::SendTypingNotification({:?})", session_id);
        let storage = self.storage.as_mut().unwrap();
        let addr = ctx.address();

        let session = storage.fetch_session_by_id(session_id).unwrap();
        assert_eq!(session_id, session.id);

        log::trace!("Sending typing notification for session: {:?}", session);

        // Since we don't want to stress database needlessly,
        // cache the sent TypingMessage timestamps and try to
        // match delivery receipts against it when they arrive.

        if self.typing_message_timestamps.len() > (TM_CACHE_CAPACITY * TM_MAX_RATE) as _ {
            // slots / slots_per_minute = minutes
            const DURATION: u64 = (TM_CACHE_TRESHOLD * 60.0 * 1000.0) as _;
            let limit = (Utc::now().timestamp_millis() as u64) - DURATION;

            let len_before = self.typing_message_timestamps.len();
            self.typing_message_timestamps.retain(|t| *t > limit);
            log::trace!(
                "Removed {} cached TypingMessage timestamps",
                len_before - self.typing_message_timestamps.len()
            );
        }

        let timestamp = Utc::now().timestamp_millis() as u64;
        self.typing_message_timestamps.insert(timestamp);

        Box::pin(
            async move {
                let group_id = match &session.r#type {
                    orm::SessionType::DirectMessage(_) => None,
                    orm::SessionType::GroupV1(group) => {
                        Some(hex::decode(&group.id).expect("valid hex identifiers in db"))
                    }
                    orm::SessionType::GroupV2(group) => {
                        Some(hex::decode(&group.id).expect("valid hex identifiers in db"))
                    }
                };

                let content = TypingMessage {
                    timestamp: Some(timestamp),
                    action: Some(if is_start {
                        Action::Started
                    } else {
                        Action::Stopped
                    } as _),
                    group_id,
                };

                addr.send(DeliverMessage {
                    content,
                    online: true,
                    timestamp,
                    session_id,
                })
                .await?
                .map(|()| session.id)
            }
            .into_actor(self)
            .map(move |res, _act, _ctx| {
                match res {
                    Ok(sid) => {
                        log::trace!("Successfully sent typing notification for session {}", sid);
                    }
                    Err(e) => {
                        log::error!("Delivering typing notification: {}", e);
                    }
                };
            }),
        )
    }
}

impl<T: Into<ContentBody>> Handler<DeliverMessage<T>> for ClientActor {
    type Result = ResponseFuture<Result<(), anyhow::Error>>;

    fn handle(&mut self, msg: DeliverMessage<T>, _ctx: &mut Self::Context) -> Self::Result {
        let DeliverMessage {
            content,
            timestamp,
            online,
            session_id,
        } = msg;
        let content = content.into();

        log::trace!("Transmitting {:?} with timestamp {}", content, timestamp);

        let storage = self.storage.clone().unwrap();
        let session = storage.fetch_session_by_id(session_id).unwrap();
        let mut sender = self.message_sender();
        let local_addr = self.local_addr.unwrap();

        Box::pin(async move {
            match &session.r#type {
                orm::SessionType::GroupV1(_group) => {
                    // FIXME
                    log::error!("Cannot send to Group V1 anymore.");
                }
                orm::SessionType::GroupV2(group) => {
                    let members = storage.fetch_group_members_by_group_v2_id(&group.id);
                    let members = members
                        .iter()
                        .filter_map(|(_member, recipient)| {
                            let member = recipient.to_service_address();

                            if !recipient.is_registered || Some(local_addr) == member {
                                None
                            } else {
                                if member.is_none() {
                                    log::warn!(
                                        "No known UUID for {}; will not deliver this message.",
                                        recipient.e164_or_uuid()
                                    );
                                }
                                member
                            }
                        })
                        .collect::<Vec<_>>();
                    // Clone + async closure means we can use an immutable borrow.
                    let results = sender
                        .send_message_to_group(&members, None, content, timestamp, online)
                        .await;
                    for result in results {
                        if let Err(e) = result {
                            anyhow::bail!(e)
                        }
                    }
                }
                orm::SessionType::DirectMessage(recipient) => {
                    let svc = recipient.to_service_address();

                    if let Some(svc) = svc {
                        if !recipient.is_registered {
                            anyhow::bail!("Unregistered recipient {}", svc.uuid.to_string());
                        }

                        if let Err(e) = sender
                            .send_message(&svc, None, content.clone(), timestamp, online)
                            .await
                        {
                            anyhow::bail!(e);
                        }
                    } else {
                        anyhow::bail!("Recipient id {} has no UUID", recipient.id);
                    }
                }
            }
            Ok(())
        })
    }
}

impl Handler<AttachmentDownloaded> for ClientActor {
    type Result = ();

    fn handle(
        &mut self,
        AttachmentDownloaded {
            session_id: sid,
            message_id: mid,
        }: AttachmentDownloaded,
        _ctx: &mut Self::Context,
    ) {
        log::info!("Attachment downloaded for message {:?}", mid);
        self.inner.pinned().borrow().attachmentDownloaded(sid, mid);
    }
}

impl Handler<StorageReady> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, storageready: StorageReady, _ctx: &mut Self::Context) -> Self::Result {
        self.storage = Some(storageready.storage.clone());
        let tel = self.config.get_tel_clone();
        let uuid = self.config.get_uuid_clone();
        let device_id = self.config.get_device_id();

        storageready.storage.mark_pending_messages_failed();

        let storage_for_password = storageready.storage;
        let request_password = async move {
            // Web socket
            let phonenumber = phonenumber::parse(None, tel).unwrap();
            let uuid = if !uuid.is_empty() {
                match uuid::Uuid::parse_str(&uuid) {
                    Ok(uuid) => Some(uuid),
                    Err(e) => {
                        log::error!("Could not parse uuid {}. Try removing the uuid field in config.yaml and restart Whisperfish. {}", &uuid, e);
                        None
                    }
                }
            } else {
                None
            };

            log::info!("Phone number: {}", phonenumber);
            log::info!("UUID: {:?}", uuid);
            log::info!("DeviceId: {}", device_id);

            let password = storage_for_password.signal_password().await.unwrap();
            let signaling_key = Some(storage_for_password.signaling_key().await.unwrap());

            (uuid, phonenumber, device_id, password, signaling_key)
        };
        let service_cfg = self.service_cfg();

        Box::pin(request_password.into_actor(self).map(
            move |(uuid, phonenumber, device_id, password, signaling_key), act, ctx| {
                // Store credentials
                let credentials = ServiceCredentials {
                    uuid,
                    phonenumber,
                    password: Some(password),
                    signaling_key,
                    device_id: Some(device_id.into()),
                };
                act.credentials = Some(credentials);
                // end store credentials

                // Signal service context
                let storage = act.storage.clone().unwrap();
                // XXX What about the whoami migration?
                let uuid = uuid.expect("local uuid to initialize service cipher");
                let cipher = ServiceCipher::new(
                    storage.clone(),
                    storage.clone(),
                    storage.clone(),
                    storage.clone(),
                    storage,
                    rand::thread_rng(),
                    service_cfg.unidentified_sender_trust_root,
                    uuid,
                    device_id.into(),
                );
                // end signal service context
                act.cipher = Some(cipher);
                act.local_addr = Some(ServiceAddress { uuid });

                Self::queue_migrations(ctx);

                ctx.notify(Restart);

                ctx.notify(RefreshPreKeys);
            },
        ))
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct Restart;

impl Handler<Restart> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _: Restart, _ctx: &mut Self::Context) -> Self::Result {
        let service = self.authenticated_service();
        let credentials = self.credentials.clone().unwrap();
        let migrations_ready = self.migration_state.ready();

        self.inner.pinned().borrow_mut().connected = false;
        self.inner.pinned().borrow().connectedChanged();
        Box::pin(
            async move {
                migrations_ready.await;
                let mut receiver = MessageReceiver::new(service.clone());

                receiver
                    .create_message_pipe(credentials)
                    .await
                    .map(|pipe| (pipe.ws(), pipe))
            }
            .into_actor(self)
            .map(move |pipe, act, ctx| match pipe {
                Ok((ws, pipe)) => {
                    ctx.add_stream(pipe.stream());

                    ctx.set_mailbox_capacity(1);
                    act.inner.pinned().borrow_mut().connected = true;
                    act.ws = Some(ws);
                    act.inner.pinned().borrow().connectedChanged();

                    // If profile stream was running, restart.
                    if let Some(handle) = act.outdated_profile_stream_handle.take() {
                        ctx.cancel_future(handle);
                    }
                    act.outdated_profile_stream_handle = Some(
                        ctx.add_stream(OutdatedProfileStream::new(act.storage.clone().unwrap())),
                    );
                }
                Err(e) => {
                    log::error!("Error starting stream: {}", e);
                    log::info!("Retrying in 10");
                    let addr = ctx.address();
                    actix::spawn(async move {
                        actix::clock::sleep(Duration::from_secs(10)).await;
                        addr.send(Restart).await.expect("retry restart");
                    });
                }
            }),
        )
    }
}

/// Queue a force-refresh of a profile fetch
#[derive(Message)]
#[rtype(result = "()")]
pub enum RefreshProfile {
    BySession(i32),
    ByRecipientId(i32),
}

impl Handler<RefreshProfile> for ClientActor {
    type Result = ();

    fn handle(&mut self, profile: RefreshProfile, _ctx: &mut Self::Context) {
        let storage = self.storage.as_ref().unwrap();
        let recipient = match profile {
            RefreshProfile::BySession(session_id) => {
                match storage.fetch_session_by_id(session_id).map(|x| x.r#type) {
                    Some(orm::SessionType::DirectMessage(recipient)) => recipient,
                    None => {
                        log::error!("No session with id {}", session_id);
                        return;
                    }
                    _ => {
                        log::error!("Can only refresh profiles for DirectMessage sessions.");
                        return;
                    }
                }
            }
            RefreshProfile::ByRecipientId(id) => match storage.fetch_recipient_by_id(id) {
                Some(r) => r,
                None => {
                    log::error!("No recipient with id {}", id);
                    return;
                }
            },
        };
        let recipient_uuid = match recipient.uuid {
            Some(uuid) => uuid.parse().expect("valid uuid in db"),
            None => {
                log::error!(
                    "Recipient without uuid; not refreshing profile: {:?}",
                    recipient
                );
                return;
            }
        };
        storage.mark_profile_outdated(recipient_uuid);
        // Polling the actor will poll the OutdatedProfileStream, which should immediately fire the
        // necessary events.  This is hacky, we should in fact wake the stream somehow to ensure
        // correct behaviour.
    }
}

impl StreamHandler<Result<Envelope, ServiceError>> for ClientActor {
    fn handle(&mut self, msg: Result<Envelope, ServiceError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                // XXX: we might want to dispatch on this error.
                log::error!("MessagePipe pushed an error: {:?}", e);
                return;
            }
        };

        let mut cipher = self.cipher.clone().expect("cipher initialized");

        if msg.is_receipt() {
            self.process_receipt(&msg);
        }

        if !(msg.is_prekey_signal_message()
            || msg.is_signal_message()
            || msg.is_unidentified_sender()
            || msg.is_receipt())
        {
            log::warn!("Unknown envelope type {:?}", msg.r#type());
        }

        let storage = self.storage.clone().expect("initialized storage");

        ctx.spawn(
            async move {
                let content = loop {
                    match cipher.open_envelope(msg.clone()).await {
                        Ok(Some(content)) => break content,
                        Ok(None) => {
                            log::warn!("Empty envelope");
                            return None;
                        }
                        Err(ServiceError::SignalProtocolError(
                            SignalProtocolError::UntrustedIdentity(addr),
                        )) => {
                            // This branch is the only one that loops, and it *should not* loop
                            // more than once.
                            log::warn!("Untrusted identity for {}; replacing identity and inserting a warning.", addr);
                            let msg = crate::store::NewMessage {
                                session_id: None,
                                source_e164: None,
                                source_uuid: Some(addr.name().into()),
                                text: "[Whisperfish] The identity key for this contact has changed.  Please verify your safety number.".into(),
                                timestamp: chrono::Utc::now().naive_utc(),
                                sent: false,
                                received: true,
                                is_read: false,
                                flags: 0,
                                attachment: None,
                                mime_type: None,
                                has_attachment: false,
                                outgoing: false,
                                is_unidentified: false,
                                quote_timestamp: None,
                            };
                            storage.process_message(msg, None);
                            let removed = storage.delete_identity_key(&addr);
                            if ! removed {
                                log::error!("Could not remove identity key for {}.  Please file a bug.", addr);
                                return None;
                            }
                        }
                        Err(e) => {
                            log::error!("Error opening envelope: {:?}", e);
                            return None;
                        }
                    }
                };

                log::trace!("Opened envelope: {:?}", content);

                Some(content)
            }
            .into_actor(self)
            .map(|content, act, ctx| {
                if let Some(content) = content {
                    act.process_envelope(content, ctx);
                }
            }),
        );
    }

    /// Called when the WebSocket somehow has disconnected.
    fn finished(&mut self, ctx: &mut Self::Context) {
        log::debug!("Attempting reconnect");

        self.inner.pinned().borrow_mut().connected = false;
        self.inner.pinned().borrow().connectedChanged();

        ctx.notify(Restart);
    }
}

#[derive(Message)]
#[rtype(result = "Result<VerificationCodeResponse, anyhow::Error>")]
pub struct Register {
    pub phonenumber: PhoneNumber,
    pub password: String,
    pub use_voice: bool,
    pub captcha: Option<String>,
}

impl Handler<Register> for ClientActor {
    type Result = ResponseActFuture<Self, Result<VerificationCodeResponse, anyhow::Error>>;

    fn handle(&mut self, reg: Register, _ctx: &mut Self::Context) -> Self::Result {
        let Register {
            phonenumber,
            password,
            use_voice,
            captcha,
        } = reg;

        let mut push_service = self.authenticated_service_with_credentials(ServiceCredentials {
            uuid: None,
            phonenumber: phonenumber.clone(),
            password: Some(password.clone()),
            signaling_key: None,
            device_id: None, // !77
        });
        // XXX add profile key when #192 implemneted
        let registration_procedure = async move {
            let captcha = captcha
                .as_deref()
                .map(|captcha| captcha.trim())
                .and_then(|captcha| captcha.strip_prefix("signalcaptcha://"));

            let mut provisioning_manager = ProvisioningManager::<AwcPushService>::new(
                &mut push_service,
                phonenumber,
                password,
            );
            if use_voice {
                provisioning_manager
                    .request_voice_verification_code(captcha, None)
                    .await
                    .map(Into::into)
            } else {
                provisioning_manager
                    .request_sms_verification_code(captcha, None)
                    .await
                    .map(Into::into)
            }
        };

        Box::pin(
            registration_procedure
                .into_actor(self)
                .map(|result, _act, _ctx| Ok(result?)),
        )
    }
}

#[derive(Message)]
#[rtype(result = "Result<(u32, VerifyAccountResponse), anyhow::Error>")]
pub struct ConfirmRegistration {
    pub phonenumber: PhoneNumber,
    pub password: String,
    pub confirm_code: String,
    pub signaling_key: [u8; 52],
}

impl Handler<ConfirmRegistration> for ClientActor {
    type Result = ResponseActFuture<Self, Result<(u32, VerifyAccountResponse), anyhow::Error>>;

    fn handle(&mut self, confirm: ConfirmRegistration, _ctx: &mut Self::Context) -> Self::Result {
        use libsignal_service::provisioning::*;

        let ConfirmRegistration {
            phonenumber,
            password,
            confirm_code,
            signaling_key,
        } = confirm;

        let registration_id = generate_registration_id(&mut rand::thread_rng());
        log::trace!("registration_id: {}", registration_id);

        let mut push_service = self.authenticated_service_with_credentials(ServiceCredentials {
            uuid: None,
            phonenumber: phonenumber.clone(),
            password: Some(password.clone()),
            signaling_key: None,
            device_id: None, // !77
        });
        let confirmation_procedure = async move {
            let mut provisioning_manager = ProvisioningManager::<AwcPushService>::new(
                &mut push_service,
                phonenumber,
                password,
            );
            // XXX centralize the place where attributes are generated.
            let account_attrs = AccountAttributes {
                // XXX probably we should remove the signaling key.
                signaling_key: Some(signaling_key.to_vec()),
                registration_id,
                voice: false,
                video: false,
                fetches_messages: true,
                pin: None,
                registration_lock: None,
                unidentified_access_key: None,
                unrestricted_unidentified_access: false,
                discoverable_by_phone_number: true,
                capabilities: whisperfish_device_capabilities(),
                name: "Whisperfish".into(),
            };
            provisioning_manager
                .confirm_verification_code(confirm_code, account_attrs)
                .await
        };

        Box::pin(
            confirmation_procedure
                .into_actor(self)
                .map(move |result, _act, _ctx| Ok((registration_id, result?))),
        )
    }
}

#[derive(Message)]
#[rtype(result = "Result<RegisterLinkedResponse, anyhow::Error>")]
pub struct RegisterLinked {
    pub device_name: String,
    pub password: String,
    pub signaling_key: [u8; 52],
    pub tx_uri: futures::channel::oneshot::Sender<String>,
}

pub struct RegisterLinkedResponse {
    pub phone_number: PhoneNumber,
    pub registration_id: u32,
    pub device_id: DeviceId,
    pub uuid: String,
    pub identity_key_pair: libsignal_protocol::IdentityKeyPair,
    pub profile_key: Vec<u8>,
}

impl Handler<RegisterLinked> for ClientActor {
    type Result = ResponseActFuture<Self, Result<RegisterLinkedResponse, anyhow::Error>>;

    fn handle(&mut self, reg: RegisterLinked, _ctx: &mut Self::Context) -> Self::Result {
        use libsignal_service::provisioning::*;

        let push_service = self.unauthenticated_service();

        let mut provision_manager: LinkingManager<AwcPushService> =
            LinkingManager::new(push_service, reg.password.clone());

        let (tx, mut rx) = futures::channel::mpsc::channel(1);

        let mut tx_uri = Some(reg.tx_uri);
        let signaling_key = reg.signaling_key;

        let registration_procedure = async move {
            let (fut1, fut2) = future::join(
                provision_manager.provision_secondary_device(
                    &mut rand::thread_rng(),
                    signaling_key,
                    tx,
                ),
                async move {
                    let mut res = Result::<RegisterLinkedResponse, anyhow::Error>::Err(
                        anyhow::Error::msg("Registration timed out"),
                    );
                    while let Some(provisioning_step) = rx.next().await {
                        match provisioning_step {
                            SecondaryDeviceProvisioning::Url(url) => {
                                log::info!("generating qrcode from provisioning link: {}", &url);
                                tx_uri
                                    .take()
                                    .expect("that only one URI is emitted by provisioning code")
                                    .send(url.to_string())
                                    .expect("to forward provisioning URL to caller");
                            }
                            SecondaryDeviceProvisioning::NewDeviceRegistration {
                                phone_number,
                                device_id,
                                registration_id,
                                uuid,
                                private_key,
                                public_key,
                                profile_key,
                            } => {
                                let identity_key_pair = libsignal_protocol::IdentityKeyPair::new(
                                    libsignal_protocol::IdentityKey::new(public_key),
                                    private_key,
                                );

                                res = Result::<RegisterLinkedResponse, anyhow::Error>::Ok(
                                    RegisterLinkedResponse {
                                        phone_number,
                                        registration_id,
                                        device_id,
                                        uuid: uuid.to_string(),
                                        identity_key_pair,
                                        profile_key,
                                    },
                                );
                            }
                        }
                    }
                    res
                },
            )
            .await;

            fut1?;
            fut2
        };

        Box::pin(
            registration_procedure
                .into_actor(self)
                .map(move |result, _act, _ctx| result),
        )
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RefreshPreKeys;

/// Java's RefreshPreKeysJob
impl Handler<RefreshPreKeys> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _: RefreshPreKeys, _ctx: &mut Self::Context) -> Self::Result {
        log::trace!("handle(RefreshPreKeys)");

        let service = self.authenticated_service();
        // XXX add profile key when #192 implemneted
        let mut am = AccountManager::new(service, None);
        let storage = self.storage.clone().unwrap();

        let proc = async move {
            let (next_signed_pre_key_id, pre_keys_offset_id) = storage.next_pre_key_ids().await;

            am.update_pre_key_bundle(
                &storage.clone(),
                &mut storage.clone(),
                &mut storage.clone(),
                &mut rand::thread_rng(),
                next_signed_pre_key_id,
                pre_keys_offset_id,
                false,
            )
            .await
        };
        // XXX: store the last refresh time somewhere.

        Box::pin(proc.into_actor(self).map(move |result, _act, _ctx| {
            if let Err(e) = result {
                log::error!("Refresh pre keys failed: {}", e);
            } else {
                log::trace!("Successfully refreshed prekeys");
            }
        }))
    }
}

// methods called from Qt
impl ClientWorker {
    #[with_executor]
    pub fn compact_db(&self) {
        let actor = self.actor.clone().unwrap();
        actix::spawn(async move {
            if let Err(e) = actor.send(CompactDb(0)).await {
                log::error!("{:?} in compact_db()", e);
            }
        });
    }

    #[with_executor]
    pub fn delete_file(&self, file_name: String) {
        let result = remove_file(&file_name);
        match result {
            Ok(()) => {
                log::trace!("Deleted file {}", file_name);
            }
            Err(e) => {
                log::trace!("Could not delete file {}: {:?}", file_name, e);
            }
        };
    }

    #[with_executor]
    pub fn refresh_profile(&self, recipient_id: i32) {
        let actor = self.actor.clone().unwrap();
        actix::spawn(async move {
            if let Err(e) = actor
                .send(RefreshProfile::ByRecipientId(recipient_id))
                .await
            {
                log::error!("{:?}", e);
            }
        });
    }

    #[with_executor]
    pub fn upload_profile(
        &self,
        given_name: String,
        family_name: String,
        about: String,
        emoji: String,
    ) {
        let actor = self.actor.clone().unwrap();
        actix::spawn(async move {
            if let Err(e) = actor
                .send(UpdateProfile {
                    given_name,
                    family_name,
                    about,
                    emoji,
                })
                .await
            {
                log::error!("{:?}", e);
            }
        });
    }

    #[with_executor]
    pub fn submit_proof_captcha(&self, token: String, response: String) {
        let actor = self.actor.clone().unwrap();
        let schema = "signalcaptcha://";
        let response = if response.starts_with(schema) {
            response.strip_prefix("signalcaptcha://").unwrap().into()
        } else {
            response
        };
        actix::spawn(async move {
            if let Err(e) = actor
                .send(ProofResponse {
                    _type: "recaptcha".into(),
                    token,
                    response,
                })
                .await
            {
                log::error!("{:?}", e);
            }
        });
    }

    #[with_executor]
    fn send_typing_notification(&self, session_id: i32, is_start: bool) {
        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(SendTypingNotification {
                    session_id,
                    is_start,
                })
                .map(Result::unwrap),
        );
    }
}

impl Handler<CompactDb> for ClientActor {
    type Result = usize;

    fn handle(&mut self, _: CompactDb, _ctx: &mut Self::Context) -> Self::Result {
        log::trace!("handle(CompactDb)");
        let store = self.storage.clone().unwrap();
        store.compact_db()
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ProofRequired {
    token: String,
    r#type: String,
}

impl Handler<ProofRequired> for ClientActor {
    type Result = ();

    fn handle(&mut self, proof: ProofRequired, _ctx: &mut Self::Context) -> Self::Result {
        self.inner
            .pinned()
            .borrow()
            .proofRequested(proof.token.into(), proof.r#type.into());
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ProofResponse {
    _type: String,
    token: String,
    response: String,
}

impl Handler<ProofResponse> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, proof: ProofResponse, ctx: &mut Self::Context) -> Self::Result {
        log::trace!("handle(ProofResponse)");

        let storage = self.storage.clone().unwrap();
        let self_recipient = storage
            .fetch_self_recipient()
            .expect("self recipient in handle(ProofResponse)");
        let profile_key = self_recipient.profile_key.map(|bytes| {
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            ProfileKey::create(key)
        });

        let service = self.authenticated_service();
        let mut am = AccountManager::new(service, profile_key);

        let addr = ctx.address();

        let proc = async move {
            am.submit_recaptcha_challenge(&proof.token, &proof.response)
                .await
        };

        Box::pin(proc.into_actor(self).map(move |result, _act, _ctx| {
            actix::spawn(async move {
                if let Err(e) = result {
                    log::error!("Error sending signalcaptcha proof: {}", e);
                    addr.send(ProofAccepted { result: false }).await
                } else {
                    log::trace!("Successfully sent signalcaptcha proof");
                    addr.send(ProofAccepted { result: true }).await
                }
            });
        }))
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ProofAccepted {
    result: bool,
}

impl Handler<ProofAccepted> for ClientActor {
    type Result = ();

    fn handle(&mut self, accepted: ProofAccepted, _ctx: &mut Self::Context) {
        self.inner
            .pinned()
            .borrow_mut()
            .proofCaptchaResult(accepted.result);
    }
}
