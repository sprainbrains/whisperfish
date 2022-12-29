#![allow(non_snake_case)]

use super::*;
use futures::prelude::*;
use qmeta_async::with_executor;

#[derive(QObject, Default)]
pub struct MessageMethods {
    base: qt_base_class!(trait QObject),
    pub actor: Option<Addr<MessageActor>>,
    pub client_actor: Option<Addr<ClientActor>>,

    // XXX move into Session
    fingerprint: Option<String>,

    numericFingerprint: qt_property!(QString; NOTIFY peerIdentityChanged READ fingerprint),
    peerName: qt_property!(QString; NOTIFY peerChanged),
    peerTel: qt_property!(QString; NOTIFY peerChanged),
    peerUuid: qt_property!(QString; NOTIFY peerChanged),
    peerHasAvatar: qt_property!(bool; NOTIFY peerChanged),
    aboutEmoji: qt_property!(QString; NOTIFY peerChanged),
    aboutText: qt_property!(QString; NOTIFY peerChanged),

    groupMembers: qt_property!(QString; NOTIFY groupMembersChanged),
    groupMemberNames: qt_property!(QString; NOTIFY groupMembersChanged),
    groupMemberUuids: qt_property!(QString; NOTIFY groupMembersChanged),
    groupId: qt_property!(QString; NOTIFY groupChanged),
    group: qt_property!(bool; NOTIFY groupChanged),
    groupV1: qt_property!(bool; NOTIFY groupChanged),
    groupV2: qt_property!(bool; NOTIFY groupChanged),
    groupDescription: qt_property!(QString; NOTIFY peerChanged),

    peerIdentityChanged: qt_signal!(),
    peerChanged: qt_signal!(),
    groupMembersChanged: qt_signal!(),
    sessionIdChanged: qt_signal!(),
    groupChanged: qt_signal!(),

    createMessage: qt_method!(
        fn(
            &self,
            session_id: i32,
            message: QString,
            attachment: QString,
            quote: i32,
            add: bool,
        ) -> i32
    ),

    sendMessage: qt_method!(fn(&self, mid: i32)),
    endSession: qt_method!(fn(&self, e164: QString)),

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
    ) -> i32 {
        let message = message.to_string();
        let attachment = attachment.to_string();

        actix::spawn(
            self.actor
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

        // TODO: QML should *not* synchronously wait for a session ID to be returned.
        -1
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

    /// Called when a message should be queued to be sent to OWS
    #[with_executor]
    fn endSession(&mut self, e164: QString) {
        actix::spawn(
            self.actor
                .as_mut()
                .unwrap()
                .send(EndSession(e164.into()))
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

    #[with_executor]
    fn fingerprint(&self) -> QString {
        self.fingerprint
            .as_deref()
            .unwrap_or("no fingerprint")
            .into()
    }
}
