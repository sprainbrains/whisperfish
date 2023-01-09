use crate::gui::StorageReady;
use crate::platform::QmlApp;
use crate::store::Storage;
use crate::worker::ClientActor;
use actix::prelude::*;
use qmetaobject::prelude::*;

mod methods;
use methods::*;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct FetchSession {
    pub id: i32,
    pub mark_read: bool,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct UpdateSession {
    pub id: i32,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct FetchMessage(pub i32);

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct FetchAllMessages(pub i32);

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct DeleteMessage(pub i32);

#[derive(Message)]
#[rtype(result = "()")]
/// Send a ne
pub struct EndSession(pub String);

pub struct MessageActor {
    inner: QObjectBox<MessageMethods>,
    storage: Option<Storage>,
}

pub fn pad_fingerprint(fp: &mut String) {
    if fp.len() == 60 {
        // twelve groups, eleven spaces.
        for i in 1..12 {
            fp.insert(6 * i - 1, ' ');
        }
    }
}

impl MessageActor {
    pub fn new(app: &mut QmlApp, client: Addr<ClientActor>) -> Self {
        let inner = QObjectBox::new(MessageMethods::default());
        app.set_object_property("MessageModel".into(), inner.pinned());
        inner.pinned().borrow_mut().client_actor = Some(client);

        Self {
            inner,
            storage: None,
        }
    }
}

impl Actor for MessageActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.inner.pinned().borrow_mut().actor = Some(ctx.address());
    }
}

impl Handler<StorageReady> for MessageActor {
    type Result = ();

    fn handle(&mut self, storageready: StorageReady, _ctx: &mut Self::Context) -> Self::Result {
        self.storage = Some(storageready.storage);
        log::trace!("MessageActor has a registered storage");
    }
}

impl Handler<DeleteMessage> for MessageActor {
    type Result = ();

    fn handle(
        &mut self,
        DeleteMessage(id): DeleteMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let _del_rows = self.storage.as_ref().unwrap().delete_message(id);
        // TODO: maybe show some error when this is None or Some(x) if x != 1
    }
}

impl Handler<EndSession> for MessageActor {
    type Result = ();

    fn handle(&mut self, EndSession(e164): EndSession, _ctx: &mut Self::Context) -> Self::Result {
        use libsignal_service::content::DataMessageFlags;
        log::trace!("MessageActor::EndSession({})", e164);

        let storage = self.storage.as_mut().unwrap();

        let (_msg, _session) = storage.process_message(
            crate::store::NewMessage {
                session_id: None,
                source_e164: Some(e164),
                source_uuid: None,
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pad_fingerprint_smoke() {
        let mut s = "892064087450853131489552767731995657884565179277972848560834".to_string();
        pad_fingerprint(&mut s);
        assert_eq!(
            s,
            "89206 40874 50853 13148 95527 67731 99565 78845 65179 27797 28485 60834"
        );
    }
}
