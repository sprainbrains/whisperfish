use actix::prelude::*;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub enum ConnectionChange {
    Connected,
    Disconnected,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RequestNotifications(pub Recipient<ConnectionChange>);

pub struct ConnectionManager {
    listeners: Vec<Recipient<ConnectionChange>>,
}

impl Actor for ConnectionManager {
    type Context = Context<Self>;
}

impl Default for ConnectionManager {
    fn default() -> Self {
        ConnectionManager { listeners: vec![] }
    }
}

impl Supervised for ConnectionManager {}
impl ArbiterService for ConnectionManager {}

impl Handler<RequestNotifications> for ConnectionManager {
    type Result = ();

    fn handle(
        &mut self,
        RequestNotifications(recipient): RequestNotifications,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.listeners.push(recipient);
    }
}
