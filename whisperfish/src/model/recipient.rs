#![allow(non_snake_case)]

use crate::store::observer::{EventObserving, Interest};
use crate::store::{orm, Storage};
use crate::{model::*, schema};
use qmeta_async::with_executor;
use qmetaobject::prelude::*;
use std::collections::HashMap;

/// QML-constructable object that interacts with a single recipient.
#[derive(Default)]
pub struct RecipientImpl {
    recipient_id: Option<i32>,
    recipient: Option<orm::Recipient>,
}

crate::observing_model! {
    pub struct Recipient(RecipientImpl) {
        recipientId: i32; READ get_recipient_id WRITE set_recipient_id,
        valid: bool; READ get_valid,
    } WITH OPTIONAL PROPERTIES FROM recipient WITH ROLE RecipientRoles {
        id Id,
        uuid Uuid,
        // These two are aliases
        e164 E164,
        phoneNumber PhoneNumber,
        username Username,
        email Email,

        blocked Blocked,

        name JoinedName,
        familyName FamilyName,
        givenName GivenName,

        about About,
        emoji Emoji,

        unidentifiedAccessMode UnidentifiedAccessModel,
        profileSharing ProfileSharing,
    }
}

impl EventObserving for RecipientImpl {
    fn observe(&mut self, storage: Storage, _event: crate::store::observer::Event) {
        if let Some(_id) = self.recipient_id {
            self.init(storage);
        }
    }

    fn interests(&self) -> Vec<Interest> {
        self.recipient_id
            .into_iter()
            .map(|id| Interest::row(schema::recipients::table, id))
            .collect()
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
    fn set_recipient_id(&mut self, storage: Option<Storage>, id: i32) {
        self.recipient_id = Some(id);
        if let Some(storage) = storage {
            self.init(storage);
        }
    }

    fn init(&mut self, storage: Storage) {
        if let Some(id) = self.recipient_id {
            self.recipient = storage.fetch_recipient_by_id(id);
        }
    }
}

#[derive(QObject, Default)]
pub struct RecipientListModel {
    base: qt_base_class!(trait QAbstractListModel),
    content: Vec<orm::Recipient>,
}

impl RecipientListModel {}

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

        UnidentifiedAccessModel(unidentified_access_mode): "unidentifiedAccessMode",
        ProfileSharing(profile_sharing): "profileSharing",
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
