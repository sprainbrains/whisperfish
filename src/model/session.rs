use std::collections::HashMap;

use crate::sfos::SailfishApp;
use crate::store::{Storage, StorageReady};

use actix::prelude::*;
use qmetaobject::*;

#[derive(QObject, Default)]
pub struct SessionModel {
    base: qt_base_class!(trait QAbstractListModel),
    actor: Option<Addr<SessionActor>>,
}

impl QAbstractListModel for SessionModel {
    fn row_count(&self) -> i32 {
        0
    }

    fn data(&self, _index: QModelIndex, _role: i32) -> QVariant {
        unimplemented!()
    }

    fn role_names(&self) -> HashMap<i32, QByteArray> {
        unimplemented!()
    }
}

pub struct SessionActor {
    inner: QObjectBox<SessionModel>,
    storage: Option<Storage>,
}

impl SessionActor {
    pub fn new(app: &mut SailfishApp) -> Self {
        let inner = QObjectBox::new(SessionModel::default());
        app.set_object_property("SessionModel".into(), inner.pinned());

        Self {
            inner,
            storage: None,
        }
    }
}

impl Handler<StorageReady> for SessionActor {
    type Result = ();

    fn handle(
        &mut self,
        StorageReady(storage): StorageReady,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.storage = Some(storage);
        log::trace!("SessionActor has a registered storage");
    }
}

impl Actor for SessionActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.inner.pinned().borrow_mut().actor = Some(ctx.address());
    }
}
