use crate::gui::StorageReady;
use crate::model::session::SessionModel;
use crate::sfos::SailfishApp;
use crate::store::{Session, Storage};

use actix::prelude::*;
use diesel::prelude::*;
use qmetaobject::*;

#[derive(Message)]
#[rtype(result = "()")]
struct SessionsLoaded(Vec<Session>);

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

    /// Panics when storage is not yet set.
    fn reload(&self, session_actor: Addr<SessionActor>) -> impl Future<Output = ()> {
        let db = self.storage.clone().unwrap().db;

        async move {
            let sessions = actix_threadpool::run(move || -> Result<_, failure::Error> {
                let db = db
                    .lock()
                    .map_err(|_| failure::format_err!("Database mutex is poisoned."))?;
                use crate::schema::session::dsl::*;
                Ok(session.order_by(timestamp.desc()).load(&*db)?)
            })
            .await
            .unwrap();
            // XXX handle error

            session_actor.send(SessionsLoaded(sessions)).await.unwrap();
            // XXX handle error
        }
    }
}

impl Actor for SessionActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.inner.pinned().borrow_mut().actor = Some(ctx.address());
    }
}

impl Handler<SessionsLoaded> for SessionActor {
    type Result = ();

    fn handle(
        &mut self,
        SessionsLoaded(sessions): SessionsLoaded,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        use std::ops::DerefMut;

        let inner = self.inner.pinned();
        let mut inner = inner.borrow_mut();

        inner.handle_sessions_loaded(sessions);
    }
}

impl Handler<StorageReady> for SessionActor {
    type Result = ();

    fn handle(
        &mut self,
        StorageReady(storage, _config): StorageReady,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        self.storage = Some(storage);
        log::trace!("SessionActor has a registered storage");

        Arbiter::spawn(self.reload(ctx.address()));
    }
}
