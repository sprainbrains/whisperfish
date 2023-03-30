use super::ClientActor;
use actix::prelude::*;
use libsignal_service::prelude::*;

// XXX In principle, these can be persisted, and don't need to be fetched on every start.
#[derive(Default)]
pub struct UnidentifiedCertificates {
    complete: Option<protocol::SenderCertificate>,
    uuid_only: Option<protocol::SenderCertificate>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RotateUnidentifiedCertificates;

impl Handler<RotateUnidentifiedCertificates> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(
        &mut self,
        _: RotateUnidentifiedCertificates,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        todo!()
    }
}
