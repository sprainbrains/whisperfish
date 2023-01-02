#![allow(non_snake_case)]

use crate::model::*;
use crate::store::observer::EventObserving;
use crate::store::{orm, Storage};
use qmeta_async::with_executor;
use qmetaobject::prelude::*;

/// QML-constructable object that interacts with a single recipient.
#[derive(Default)]
pub struct RecipientImpl {
    recipient_id: Option<i32>,
    recipient: Option<orm::Recipient>,
}

crate::observing_model! {
    pub struct Recipient(RecipientImpl) {
        recipientId: i32; READ get_recipient_id WRITE set_recipient_id,
    }
}

impl EventObserving for RecipientImpl {
    fn observe(&mut self, storage: Storage, _event: crate::store::observer::Event) {
        if let Some(_id) = self.recipient_id {
            self.init(storage);
        }
    }

    fn interests() -> Vec<crate::store::observer::Interest> {
        vec![crate::store::observer::Interest::All]
    }
}

impl RecipientImpl {
    fn get_recipient_id(&self) -> i32 {
        self.recipient_id.unwrap_or(-1)
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
