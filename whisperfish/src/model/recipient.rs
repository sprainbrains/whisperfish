#![allow(non_snake_case)]

use crate::model::*;
use crate::store::observer::{EventObserving, Interest};
use crate::store::orm;
use actix::{ActorContext, Handler};
use futures::TryFutureExt;
use libsignal_service::prelude::protocol::SessionStoreExt;
use qmeta_async::with_executor;
use qmetaobject::prelude::*;
use std::collections::HashMap;

/// QML-constructable object that interacts with a single recipient.
#[derive(Default, QObject)]
pub struct RecipientImpl {
    base: qt_base_class!(trait QObject),
    recipient_id: Option<i32>,
    recipient: Option<RecipientWithFingerprint>,
}

crate::observing_model! {
    pub struct Recipient(RecipientImpl) {
        recipientId: i32; READ get_recipient_id WRITE set_recipient_id,
        valid: bool; READ get_valid,
    } WITH OPTIONAL PROPERTIES FROM recipient WITH ROLE RecipientWithFingerprintRoles {
        id Id,
        directMessageSessionId DirectMessageSessionId,
        uuid Uuid,
        // These two are aliases
        e164 E164,
        phoneNumber PhoneNumber,
        username Username,
        email Email,

        sessionFingerprint SessionFingerprint,

        blocked Blocked,

        name JoinedName,
        familyName FamilyName,
        givenName GivenName,

        about About,
        emoji Emoji,

        unidentifiedAccessMode UnidentifiedAccessMode,
        profileSharing ProfileSharing,

        isRegistered IsRegistered,
    }
}

impl EventObserving for RecipientImpl {
    type Context = ModelContext<Self>;

    fn observe(&mut self, ctx: Self::Context, _event: crate::store::observer::Event) {
        if self.recipient_id.is_some() {
            self.init(ctx);
        }
    }

    fn interests(&self) -> Vec<Interest> {
        self.recipient
            .iter()
            .flat_map(|r| r.inner.interests())
            .collect()
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
struct FingerprintComputed {
    recipient_id: i32,
    fingerprint: String,
}

impl Handler<FingerprintComputed> for ObservingModelActor<RecipientImpl> {
    type Result = ();

    fn handle(
        &mut self,
        FingerprintComputed {
            recipient_id,
            fingerprint,
        }: FingerprintComputed,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        match self.model.upgrade() {
            Some(model) => {
                let model = model.pinned();
                let mut model = model.borrow_mut();
                if let Some(recipient) = &mut model.recipient {
                    if recipient.id != recipient_id {
                        log::trace!("Different recipient_id requested, dropping fingerprint");
                    } else {
                        recipient.fingerprint = Some(fingerprint);
                        // TODO: trigger something changed
                    }
                }
            }
            None => {
                // In principle, the actor should have gotten stopped when the model got dropped,
                // because the actor's only strong reference is contained in the ObservingModel.
                log::debug!("Model got dropped, stopping actor execution.");
                // XXX What is the difference between stop and terminate?
                ctx.stop();
            }
        }
    }
}

impl RecipientImpl {
    fn get_recipient_id(&self) -> i32 {
        self.recipient_id.unwrap_or(-1)
    }

    fn get_valid(&self) -> bool {
        self.recipient_id.is_some() && self.recipient.is_some()
    }

    #[with_executor]
    fn set_recipient_id(&mut self, ctx: Option<ModelContext<Self>>, id: i32) {
        self.recipient_id = Some(id);
        if let Some(ctx) = ctx {
            self.init(ctx);
        }
    }

    fn init(&mut self, ctx: ModelContext<Self>) {
        let storage = ctx.storage();
        if let Some(id) = self.recipient_id {
            let recipient = if id >= 0 {
                let recipient = storage.fetch_recipient_by_id(id).map(|inner| {
                    let direct_message_recipient_id = storage
                        .fetch_session_by_recipient_id(inner.id)
                        .map(|session| session.id)
                        .unwrap_or(-1);
                    RecipientWithFingerprint {
                        inner,
                        direct_message_recipient_id,
                        fingerprint: None,
                    }
                });
                // If a recipient was found, attempt to compute the fingeprint
                if let Some(r) = &recipient {
                    if let Some(recipient_svc) = r.to_service_address() {
                        let compute_fingerprint = async move {
                            let local = storage
                                .fetch_self_recipient()
                                .expect("self recipient present in db");
                            let local_svc =
                                local.to_service_address().expect("self-recipient has UUID");
                            let fingerprint = storage
                                .compute_safety_number(&local_svc, &recipient_svc, None)
                                .await?;
                            ctx.addr()
                                .send(FingerprintComputed {
                                    recipient_id: id,
                                    fingerprint,
                                })
                                .await?;

                            Result::<_, anyhow::Error>::Ok(())
                        }
                        .map_ok_or_else(|e| log::error!("Computing fingeprint: {}", e), |_| ());
                        actix::spawn(compute_fingerprint);
                    }
                }
                recipient
            } else {
                None
            };
            self.recipient = recipient;
            // XXX trigger Qt signal for this?
        }
    }
}

#[derive(QObject, Default)]
pub struct RecipientListModel {
    base: qt_base_class!(trait QAbstractListModel),
    content: Vec<orm::Recipient>,
}

pub struct RecipientWithFingerprint {
    inner: orm::Recipient,
    direct_message_recipient_id: i32,
    fingerprint: Option<String>,
}

impl std::ops::Deref for RecipientWithFingerprint {
    type Target = orm::Recipient;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl RecipientListModel {}

define_model_roles! {
    pub(super) enum RecipientWithFingerprintRoles for RecipientWithFingerprint {
        Id(id): "id",
        DirectMessageSessionId(direct_message_recipient_id): "directMessageSessionId",
        Uuid(uuid via qstring_from_option): "uuid",
        // These two are aliases
        E164(e164 via qstring_from_option): "e164",
        PhoneNumber(e164 via qstring_from_option): "phoneNumber",
        Username(username via qstring_from_option): "username",
        Email(email via qstring_from_option): "email",
        IsRegistered(is_registered): "isRegistered",

        Blocked(blocked): "blocked",

        JoinedName(profile_joined_name via qstring_from_option): "name",
        FamilyName(profile_family_name via qstring_from_option): "familyName",
        GivenName(profile_given_name via qstring_from_option): "givenName",

        About(about via qstring_from_option): "about",
        Emoji(about_emoji via qstring_from_option): "emoji",

        UnidentifiedAccessMode(unidentified_access_mode): "unidentifiedAccessMode",
        ProfileSharing(profile_sharing): "profileSharing",

        SessionFingerprint(fingerprint via qstring_from_option): "sessionFingerprint",
    }
}

define_model_roles! {
    pub(super) enum RecipientRoles for orm::Recipient {
        Id(id): "id",
        Uuid(uuid via qstring_from_option): "uuid",
        // These two are aliases
        E164(e164 via qstring_from_option): "e164",
        PhoneNumber(e164 via qstring_from_option): "phoneNumber",
        Username(username via qstring_from_option): "username",
        Email(email via qstring_from_option): "email",

        Blocked(blocked): "blocked",

        JoinedName(profile_joined_name via qstring_from_option): "name",
        FamilyName(profile_family_name via qstring_from_option): "familyName",
        GivenName(profile_given_name via qstring_from_option): "givenName",

        About(about via qstring_from_option): "about",
        Emoji(about_emoji via qstring_from_option): "emoji",

        UnidentifiedAccessMode(unidentified_access_mode): "unidentifiedAccessMode",
        ProfileSharing(profile_sharing): "profileSharing",

        IsRegistered(is_registered): "isRegistered",
    }
}

impl QAbstractListModel for RecipientListModel {
    fn row_count(&self) -> i32 {
        self.content.len() as _
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let role = RecipientRoles::from(role);
        role.get(&self.content[index.row() as usize])
    }

    fn role_names(&self) -> HashMap<i32, QByteArray> {
        RecipientRoles::role_names()
    }
}
