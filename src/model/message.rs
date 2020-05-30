use crate::sfos::SailfishApp;

use actix::prelude::*;
use qmetaobject::*;

#[derive(QObject, Default)]
struct MessageModel {
    base: qt_base_class!(trait QObject),
    actor: Option<Addr<MessageActor>>,
}

pub struct MessageActor {
    inner: QObjectBox<MessageModel>,
}

impl MessageActor {
    pub fn new(app: &mut SailfishApp) -> Self {
        let inner = QObjectBox::new(MessageModel::default());
        app.set_object_property("MessageModel".into(), inner.pinned());

        Self {
            inner,
        }
    }
}

impl Actor for MessageActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.inner.pinned().borrow_mut().actor = Some(ctx.address());
    }
}
