#![allow(non_snake_case)]

use crate::worker::QueueMessage;

use super::*;
use futures::prelude::*;
use qmeta_async::with_executor;

#[derive(QObject, Default)]
pub struct MessageMethods {
    base: qt_base_class!(trait QObject),
    pub actor: Option<Addr<MessageActor>>,
    pub client_actor: Option<Addr<ClientActor>>,

    createMessage: qt_method!(
        fn(&self, session_id: i32, message: QString, attachment: QString, quote: i32, add: bool)
    ),

    sendMessage: qt_method!(fn(&self, mid: i32)),
    endSession: qt_method!(fn(&self, recipient_id: i32)),

    remove: qt_method!(
        fn(
            &self,
            id: i32, /* FIXME the implemented method takes an *index* but should take a message ID */
        )
    ),
}

impl MessageMethods {
    #[with_executor]
    fn createMessage(
        &mut self,
        session_id: i32,
        message: QString,
        attachment: QString,
        quote: i32,
        _add: bool,
    ) {
        let message = message.to_string();
        let attachment = attachment.to_string();

        actix::spawn(
            self.client_actor
                .as_ref()
                .unwrap()
                .send(QueueMessage {
                    session_id,
                    message,
                    attachment,
                    quote,
                })
                .map(Result::unwrap),
        );
    }

    /// Called when a message should be queued to be sent to OWS
    #[with_executor]
    fn sendMessage(&mut self, mid: i32) {
        actix::spawn(
            self.client_actor
                .as_mut()
                .unwrap()
                .send(crate::worker::SendMessage(mid))
                .map(Result::unwrap),
        );
    }

    #[with_executor]
    fn endSession(&mut self, id: i32) {
        actix::spawn(
            self.client_actor
                .as_mut()
                .unwrap()
                .send(crate::worker::EndSession(id))
                .map(Result::unwrap),
        );
    }

    /// Remove a message from the database.
    #[with_executor]
    pub fn remove(&self, id: i32) {
        actix::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(DeleteMessage(id))
                .map(Result::unwrap),
        );

        log::trace!("Dispatched DeleteMessage({})", id);
    }
}
