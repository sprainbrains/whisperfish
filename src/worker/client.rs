use std::fs::remove_file;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use actix::prelude::*;
use anyhow::Context;
use chrono::prelude::*;
use futures::prelude::*;
use libsignal_service::proto::typing_message::Action;
use libsignal_service::websocket::SignalWebSocket;
use phonenumber::PhoneNumber;
use qmeta_async::with_executor;
use qmetaobject::prelude::*;

use crate::actor::{LoadAllSessions, SessionActor};
use crate::gui::StorageReady;
use crate::model::DeviceModel;
use crate::platform::QmlApp;
use crate::store::{orm, Storage};

use libsignal_service::configuration::SignalServers;
use libsignal_service::content::sync_message::Request as SyncRequest;
use libsignal_service::content::DataMessageFlags;
use libsignal_service::content::{
    sync_message, AttachmentPointer, ContentBody, DataMessage, GroupContext, GroupContextV2,
    GroupType, Metadata, TypingMessage,
};
use libsignal_service::prelude::protocol::*;
use libsignal_service::prelude::*;
use libsignal_service::push_service::{DeviceId, DEFAULT_DEVICE_ID};
use libsignal_service::sender::AttachmentSpec;
use libsignal_service::AccountManager;
use libsignal_service_actix::prelude::*;

use libsignal_service::provisioning::ProvisioningManager;
pub use libsignal_service::provisioning::{VerificationCodeResponse, VerifyAccountResponse};
pub use libsignal_service::push_service::DeviceInfo;

// XXX maybe the session-to-db migration should move into the store module.
pub mod migrations;

mod linked_devices;
pub use linked_devices::*;

mod profile;
pub use profile::*;

mod profile_upload;
pub use profile_upload::*;

mod groupv2;
pub use groupv2::*;

use crate::millis_to_naive_chrono;

use mime_classifier::{ApacheBugFlag, LoadContext, MimeClassifier, NoSniffFlag};

use super::profile_refresh::OutdatedProfileStream;

#[derive(Message)]
#[rtype(result = "()")]
/// Enqueue a message on socket by MID
pub struct SendMessage(pub i32);

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
        senderIdentifier: QString,
        message: QString,
        isGroup: bool
    ),
    promptResetPeerIdentity: qt_signal!(),
    messageSent: qt_signal!(sid: i32, mid: i32, message: QString),
    messageNotSent: qt_signal!(sid: i32, mid: i32),

    send_typing_notification: qt_method!(fn(&self, id: i32, is_start: bool)),

    connected: qt_property!(bool; NOTIFY connectedChanged),
    connectedChanged: qt_signal!(),

    actor: Option<Addr<ClientActor>>,
    session_actor: Option<Addr<SessionActor>>,
    device_model: Option<QObjectBox<DeviceModel>>,

    // Linked device management
    link_device: qt_method!(fn(&self, tsurl: String)),
    unlink_device: qt_method!(fn(&self, id: i64)),
    reload_linked_devices: qt_method!(fn(&self)),
    compress_db: qt_method!(fn(&self)),

    refresh_group_v2: qt_method!(fn(&self, session_id: usize)),

    delete_file: qt_method!(fn(&self, file_name: String)),

    refresh_profile: qt_method!(fn(&self, session_id: i32)),
}

/// ClientActor keeps track of the connection state.
pub struct ClientActor {
    inner: QObjectBox<ClientWorker>,

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
            rand::rngs::ThreadRng,
        >,
    >,
    config: std::sync::Arc<crate::config::SignalConfig>,

    start_time: DateTime<Local>,

    outdated_profile_stream_handle: Option<SpawnHandle>,
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

        Ok(Self {
            inner,
            credentials: None,
            local_addr: None,
            storage: None,
            cipher: None,
            ws: None,
            config,

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
        rand::rngs::ThreadRng,
    > {
        let storage = self.storage.clone().unwrap();
        let service = self.authenticated_service();
        MessageSender::new(
            service,
            self.cipher.clone().unwrap(),
            rand::thread_rng(),
            storage.clone(),
            storage,
            self.local_addr.clone().unwrap(),
            self.config.get_device_id(),
        )
    }

    fn service_cfg(&self) -> ServiceConfiguration {
        // XXX: read the configuration files!
        SignalServers::Production.into()
    }

    /// Process incoming message from Signal
    ///
    /// This was `MessageHandler` in Go.
    ///
    /// TODO: consider putting this as an actor `Handle<>` implementation instead.
    pub fn handle_message(
        &mut self,
        ctx: &mut <Self as Actor>::Context,
        source_e164: Option<String>,
        source_uuid: Option<String>,
        msg: DataMessage,
        is_sync_sent: bool,
        timestamp: u64,
    ) {
        let settings = crate::config::Settings::default();

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
            let source_e164 = source_e164.clone();
            let source_uuid = source_uuid.clone();
            actix::spawn(async move {
                if let Some(e164) = source_e164.as_ref() {
                    if let Err(e) = storage.delete_all_sessions(e164).await {
                        log::error!(
                            "End session (e164) requested, but could not end session: {:?}",
                            e
                        );
                    }
                }
                if let Some(uuid) = source_uuid.as_ref() {
                    if let Err(e) = storage.delete_all_sessions(uuid).await {
                        log::error!(
                            "End session (uuid) requested, but could not end session: {:?}",
                            e
                        );
                    }
                }
            });
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
            let config = self.config.clone();
            if let Some((message, session)) = storage.process_reaction(
                &sender_recipient
                    .clone()
                    .or_else(|| storage.fetch_self_recipient(&config))
                    .expect("sender or self-sent"),
                &msg,
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
            Some("".into())
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
            return;
        };

        let mut new_message = crate::store::NewMessage {
            source_e164,
            source_uuid,
            text,
            flags: msg.flags() as i32,
            outgoing: is_sync_sent,
            sent: is_sync_sent,
            timestamp: millis_to_naive_chrono(if is_sync_sent && timestamp > 0 {
                timestamp
            } else {
                msg.timestamp()
            } as u64),
            has_attachment: !msg.attachments.is_empty(),
            mime_type: None,  // Attachments are further handled asynchronously
            received: false,  // This is set true by a receipt handler
            session_id: None, // Canary value checked later
            attachment: None,
            is_read: is_sync_sent,
        };

        let group = if let Some(group) = msg.group.as_ref() {
            match group.r#type() {
                GroupType::Update => {
                    new_message.text = String::from("Group was updated");
                }
                GroupType::Quit => {
                    new_message.text = String::from("Member left group");
                }
                t => log::warn!("Unhandled group type {:?}", t),
            }

            Some(
                storage.fetch_or_insert_session_by_group_v1(&crate::store::GroupV1 {
                    id: group.id().to_vec(),
                    name: group.name().to_string(),
                    members: group.members_e164.clone(),
                }),
            )
        } else if let Some(group) = msg.group_v2.as_ref() {
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
                ctx.notify(RequestGroupV2Info(store_v2.clone()));
            } else if !storage.group_v2_exists(&store_v2) {
                log::info!(
                    "We don't know this group. We'll request it's structure from the server."
                );
                ctx.notify(RequestGroupV2Info(store_v2.clone()));
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

        if settings.get_bool("save_attachments") && !settings.get_bool("incognito") {
            for attachment in msg.attachments {
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
                    ptr: attachment,
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
                orm::SessionType::DirectMessage(recipient) => recipient.e164_or_uuid(),
            };

            self.inner.pinned().borrow_mut().notifyMessage(
                session.id,
                message.id,
                session_name.into(),
                sender_recipient
                    .map(|x| x.e164_or_uuid().into())
                    .unwrap_or_else(|| "".into()),
                message.text.as_deref().unwrap_or("").into(),
                session.is_group(),
            );
        }
    }

    fn handle_sync_request(&mut self, meta: Metadata, req: SyncRequest) {
        use sync_message::request::Type;
        log::trace!("Processing sync request {:?}", req.r#type());

        let local_addr = self.local_addr.clone().unwrap();
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
                        let db = storage.db
                            .lock();
                        recipients.load(&*db)?
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
            };

            Ok::<_, anyhow::Error>(())
        }.map(|v| if let Err(e) = v {log::error!("{:?}", e)}));
    }

    fn process_receipt(&mut self, msg: &Envelope) {
        log::info!("Received receipt: {}", msg.timestamp());

        let storage = self.storage.as_mut().expect("storage initialized");

        let ts = msg.timestamp();
        let source = msg.source_address();

        let ts = millis_to_naive_chrono(ts);
        log::trace!(
            "Marking message from {} at {} ({}) as received.",
            source,
            ts,
            msg.timestamp()
        );
        if let Some((sess, msg)) = storage.mark_message_received(
            source.e164().as_deref(),
            source.uuid.as_ref().map(uuid::Uuid::to_string).as_deref(),
            ts,
            None,
        ) {
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
        let storage = self.storage.as_mut().expect("storage initialized");

        match body {
            ContentBody::DataMessage(message) => {
                let e164 = metadata.sender.e164();
                let uuid = metadata.sender.uuid.map(|uuid| uuid.to_string());
                self.handle_message(ctx, e164, uuid, message, false, metadata.timestamp)
            }
            ContentBody::SynchronizeMessage(message) => {
                if let Some(sent) = message.sent {
                    log::trace!("Sync sent message");
                    // These are messages sent through a paired device.

                    let message = sent.message.expect("sync sent with message");
                    self.handle_message(
                        ctx,
                        // Empty string mainly when groups,
                        // but maybe needs a check. TODO
                        sent.destination_e164,
                        sent.destination_uuid,
                        message,
                        true,
                        0,
                    );
                } else if let Some(request) = message.request {
                    log::trace!("Sync request message");
                    self.handle_sync_request(metadata, request);
                } else if !message.read.is_empty() {
                    log::trace!("Sync read message");
                    for read in &message.read {
                        // XXX: this should probably not be based on ts alone.
                        let ts = read.timestamp();
                        let source = read.sender_e164();
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
                } else {
                    log::warn!("Sync message without known sync type");
                }
            }
            ContentBody::TypingMessage(typing) => {
                log::info!("{} is typing.", metadata.sender);
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
                log::info!("{} received a message.", metadata.sender);
                // XXX dispatch on receipt.type
                for &ts in &receipt.timestamp {
                    // Signal uses timestamps in milliseconds, chrono has nanoseconds
                    if let Some((sess, msg)) = storage.mark_message_received(
                        metadata.sender.e164().as_deref(),
                        metadata
                            .sender
                            .uuid
                            .as_ref()
                            .map(uuid::Uuid::to_string)
                            .as_deref(),
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
                log::info!("{} is calling.", metadata.sender);
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
                    log::error!("{}", e);
                    let mut log = act.attachment_log();
                    if let Err(e) = writeln!(log, "{}", e) {
                        log::error!("Could not write error to error log: {}", e);
                    }
                }
            }),
        )
    }
}

impl Handler<SendMessage> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;

    // Equiv of worker/send.go
    fn handle(&mut self, SendMessage(mid): SendMessage, _ctx: &mut Self::Context) -> Self::Result {
        log::info!("ClientActor::SendMessage({:?})", mid);
        let mut sender = self.message_sender();
        let storage = self.storage.as_mut().unwrap();
        let msg = storage.fetch_message_by_id(mid).unwrap();

        let session = storage.fetch_session_by_id(msg.session_id).unwrap();
        let session_id = session.id;

        if msg.sent_timestamp.is_some() {
            log::warn!("Message already sent, refusing to retransmit.");
            return Box::pin(async {}.into_actor(self).map(|_, _, _| ()));
        }

        let self_recipient = storage.fetch_self_recipient(&self.config);
        log::trace!("Sending for session: {:?}", session);
        log::trace!("Sending message: {:?}", msg);

        let local_addr = self.local_addr.clone().unwrap();
        let storage = storage.clone();
        Box::pin(
            async move {
                let group = if let orm::SessionType::GroupV1(group) = &session.r#type {
                    Some(GroupContext {
                        id: Some(hex::decode(&group.id).expect("hex encoded group id")),
                        r#type: Some(GroupType::Deliver.into()),
                        ..Default::default()
                    })
                } else {
                    None
                };
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

                // XXX online status goes in that bool
                let online = false;
                let timestamp = msg.server_timestamp.timestamp_millis() as u64;
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
                    group,
                    group_v2,

                    profile_key: self_recipient.and_then(|r| r.profile_key),
                    ..Default::default()
                };

                let attachments = storage.fetch_attachments_for_message(msg.id);

                for attachment in &attachments {
                    let attachment_path = attachment
                        .attachment_path
                        .clone()
                        .expect("attachment path when uploading");
                    let contents =
                        tokio::task::spawn_blocking(move || std::fs::read(&attachment_path))
                            .await
                            .context("threadpool")?
                            .context("reading attachment")?;
                    let attachment_path = attachment.attachment_path.as_ref().unwrap();
                    let spec = AttachmentSpec {
                        content_type: mime_guess::from_path(&attachment_path)
                            .first()
                            .unwrap()
                            .essence_str()
                            .into(),
                        length: contents.len(),
                        file_name: Path::new(&attachment_path)
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

                log::trace!("Transmitting {:?} with timestamp {}", content, timestamp);

                match &session.r#type {
                    orm::SessionType::GroupV1(group) => {
                        let members = storage.fetch_group_members_by_group_v1_id(&group.id);
                        let members = members
                            .iter()
                            .filter_map(|(_member, recipient)| {
                                let member = recipient.to_service_address();

                                if local_addr.matches(&member) {
                                    None
                                } else {
                                    Some(member)
                                }
                            })
                            .collect::<Vec<_>>();
                        // Clone + async closure means we can use an immutable borrow.
                        let results = sender
                            .send_message_to_group(&members, None, content, timestamp, online)
                            .await;
                        for result in results {
                            if let Err(e) = result {
                                anyhow::bail!("Error sending message: {}", e);
                            }
                        }
                    }
                    orm::SessionType::GroupV2(group) => {
                        let members = storage.fetch_group_members_by_group_v2_id(&group.id);
                        let members = members
                            .iter()
                            .filter_map(|(_member, recipient)| {
                                let member = recipient.to_service_address();

                                if local_addr.matches(&member) {
                                    None
                                } else {
                                    Some(member)
                                }
                            })
                            .collect::<Vec<_>>();
                        // Clone + async closure means we can use an immutable borrow.
                        let results = sender
                            .send_message_to_group(&members, None, content, timestamp, online)
                            .await;
                        for result in results {
                            if let Err(e) = result {
                                storage.fail_message(mid);
                                anyhow::bail!("Error sending message: {}", e);
                            }
                        }
                    }
                    orm::SessionType::DirectMessage(recipient) => {
                        let recipient = recipient.to_service_address();

                        if let Err(e) = sender
                            .send_message(&recipient, None, content.clone(), timestamp, online)
                            .await
                        {
                            storage.fail_message(mid);
                            anyhow::bail!("Error sending message: {}", e);
                        }
                    }
                }

                // Mark as sent
                storage.dequeue_message(mid, chrono::Utc::now().naive_utc());

                Ok((session.id, mid, msg.text))
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
                    }
                };
                actix::spawn(
                    act.inner
                        .pinned()
                        .borrow()
                        .session_actor
                        .clone()
                        .unwrap()
                        .send(LoadAllSessions)
                        .map(Result::unwrap),
                );
            }),
        )
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
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        log::info!("ClientActor::SendTypingNotification({:?})", session_id);
        let mut sender = self.message_sender();
        let storage = self.storage.as_mut().unwrap();

        let session = storage.fetch_session_by_id(session_id).unwrap();
        assert_eq!(session_id, session.id);

        log::trace!("Sending typing notification for session: {:?}", session);

        let local_addr = self.local_addr.clone().unwrap();
        let storage = storage.clone();
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

                let online = true;
                let timestamp = Utc::now().timestamp_millis() as u64;
                let content = TypingMessage {
                    timestamp: Some(timestamp),
                    action: Some(if is_start {
                        Action::Started
                    } else {
                        Action::Stopped
                    } as _),
                    group_id,
                };

                log::trace!("Transmitting {:?} with timestamp {}", content, timestamp);

                match &session.r#type {
                    orm::SessionType::GroupV1(group) => {
                        let members = storage.fetch_group_members_by_group_v1_id(&group.id);
                        let members = members
                            .iter()
                            .filter_map(|(_member, recipient)| {
                                let member = recipient.to_service_address();

                                if local_addr.matches(&member) {
                                    None
                                } else {
                                    Some(member)
                                }
                            })
                            .collect::<Vec<_>>();
                        // Clone + async closure means we can use an immutable borrow.
                        let results = sender
                            .send_message_to_group(&members, None, content, timestamp, online)
                            .await;
                        for result in results {
                            if let Err(e) = result {
                                anyhow::bail!("Error sending message: {}", e);
                            }
                        }
                    }
                    orm::SessionType::GroupV2(group) => {
                        let members = storage.fetch_group_members_by_group_v2_id(&group.id);
                        let members = members
                            .iter()
                            .filter_map(|(_member, recipient)| {
                                let member = recipient.to_service_address();

                                if local_addr.matches(&member) {
                                    None
                                } else {
                                    Some(member)
                                }
                            })
                            .collect::<Vec<_>>();
                        // Clone + async closure means we can use an immutable borrow.
                        let results = sender
                            .send_message_to_group(&members, None, content, timestamp, online)
                            .await;
                        for result in results {
                            if let Err(e) = result {
                                anyhow::bail!("Error sending message: {}", e);
                            }
                        }
                    }
                    orm::SessionType::DirectMessage(recipient) => {
                        let recipient = recipient.to_service_address();

                        if let Err(e) = sender
                            .send_message(&recipient, None, content.clone(), timestamp, online)
                            .await
                        {
                            anyhow::bail!("Error sending message: {}", e);
                        }
                    }
                }

                Ok(session.id)
            }
            .into_actor(self)
            .map(move |res, _act, _ctx| {
                match res {
                    Ok(sid) => {
                        log::trace!("Successfully sent typing notification for session {}", sid);
                    }
                    Err(e) => {
                        log::error!("Sending typing notification: {}", e);
                    }
                };
            }),
        )
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
                    phonenumber: phonenumber.clone(),
                    password: Some(password),
                    signaling_key,
                    device_id: Some(device_id.into()),
                };
                act.credentials = Some(credentials);
                // end store credentials

                // Signal service context
                let local_addr = ServiceAddress {
                    uuid,
                    phonenumber: Some(phonenumber),
                    relay: None,
                };
                let storage = act.storage.clone().unwrap();
                let cipher = ServiceCipher::new(
                    storage.clone(),
                    storage.clone(),
                    storage.clone(),
                    storage,
                    rand::thread_rng(),
                    service_cfg.unidentified_sender_trust_root,
                    uuid.expect("local uuid to initialize service cipher"),
                    device_id.into(),
                );
                // end signal service context
                act.cipher = Some(cipher);
                act.local_addr = Some(local_addr);

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

        self.inner.pinned().borrow_mut().connected = false;
        self.inner.pinned().borrow().connectedChanged();
        Box::pin(
            async move {
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
                    ctx.add_stream(OutdatedProfileStream::new(
                        act.storage.clone().unwrap(),
                        act.config.clone(),
                    ));
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

        ctx.spawn(
            async move {
                let content = match cipher.open_envelope(msg).await {
                    Ok(Some(content)) => content,
                    Ok(None) => {
                        log::warn!("Empty envelope");
                        return None;
                    }
                    Err(e) => {
                        log::error!("Error opening envelope: {:?}", e);
                        return None;
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
    pub confirm_code: u32,
    pub signaling_key: [u8; 52],
}

impl Handler<ConfirmRegistration> for ClientActor {
    type Result = ResponseActFuture<Self, Result<(u32, VerifyAccountResponse), anyhow::Error>>;

    fn handle(&mut self, confirm: ConfirmRegistration, _ctx: &mut Self::Context) -> Self::Result {
        use libsignal_service::provisioning::*;
        use libsignal_service::push_service::{AccountAttributes, DeviceCapabilities};

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
                capabilities: DeviceCapabilities {
                    announcement_group: false,
                    gv2: true,
                    storage: false,
                    gv1_migration: true,
                    sender_key: false,
                    change_number: false,
                    gift_badges: false,
                    stories: false,
                },
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
    pub fn compress_db(&self) {
        let actor = self.actor.clone().unwrap();
        actix::spawn(async move {
            if let Err(e) = actor.send(CompactDb(0)).await {
                log::error!("{:?}", e);
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
    pub fn refresh_profile(&self, session_id: i32) {
        let actor = self.actor.clone().unwrap();
        actix::spawn(async move {
            if let Err(e) = actor.send(RefreshProfile::BySession(session_id)).await {
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
        let res = store.compress_db();
        log::trace!("  res = {}", res);
        res
    }
}
