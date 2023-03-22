#![allow(non_snake_case)]

use crate::model::*;
use crate::schema;
use crate::store::observer::EventObserving;
use crate::store::observer::Interest;
use crate::store::Storage;
use phonenumber::PhoneNumber;
use qmetaobject::prelude::*;

/// QML-constructable object that queries a session based on e164 or uuid, and creates it if
/// necessary.
#[derive(Default, QObject)]
pub struct SessionImpl {
    base: qt_base_class!(trait QObject),
    session_id: Option<i32>,
    uuid: Option<uuid::Uuid>,
    e164: Option<phonenumber::PhoneNumber>,
}

crate::observing_model! {
    pub struct Session(SessionImpl) {
        sessionId: i32; READ get_session_id,
        uuid: QString; READ get_uuid WRITE set_uuid,
        e164: QString; READ get_e164 WRITE set_e164,
        ready: bool; READ get_ready,
        invalid: bool; READ get_invalid,
    }
}

impl EventObserving for SessionImpl {
    type Context = ModelContext<Self>;

    fn observe(&mut self, ctx: Self::Context, event: crate::store::observer::Event) {
        let storage = ctx.storage();

        // If something changed
        self.fetch(storage);
    }

    fn interests(&self) -> Vec<Interest> {
        vec![Interest::whole_table(schema::sessions::table)]
    }
}

impl SessionImpl {
    fn get_session_id(&self) -> i32 {
        self.session_id.unwrap_or(-1)
    }

    fn get_ready(&self) -> bool {
        self.session_id.is_some()
    }

    fn get_invalid(&self) -> bool {
        self.e164.is_none() && self.uuid.is_none()
    }

    fn fetch(&mut self, storage: Storage) {}

    fn get_uuid(&self) -> QString {
        self.uuid
            .as_ref()
            .map(uuid::Uuid::to_string)
            .unwrap_or_default()
            .into()
    }

    fn set_uuid(&mut self, ctx: Option<ModelContext<Self>>, uuid: QString) {
        self.uuid = uuid::Uuid::parse_str(&uuid.to_string())
            // inspect_err https://github.com/rust-lang/rust/pull/91346 Rust 1.59
            .map_err(|e| {
                log::error!("Parsing uuid: {}", e);
                e
            })
            .ok();
        self.e164 = None;
        if let Some(ctx) = ctx {
            self.fetch(ctx.storage());
        }
    }

    fn set_e164(&mut self, ctx: Option<ModelContext<Self>>, e164: QString) {
        self.e164 = phonenumber::parse(None, e164.to_string())
            // inspect_err https://github.com/rust-lang/rust/pull/91346 Rust 1.59
            .map_err(|e| {
                log::error!("Parsing phone number: {}", e);
                e
            })
            .ok();
        self.uuid = None;
        if let Some(ctx) = ctx {
            self.fetch(ctx.storage());
        }
    }

    fn get_e164(&self) -> QString {
        self.e164
            .as_ref()
            .map(PhoneNumber::to_string)
            .unwrap_or_default()
            .into()
    }

    fn init(&mut self, ctx: ModelContext<Self>) {
        if let Some(id) = self.session_id {
            self.fetch(ctx.storage());
        }
    }
}
