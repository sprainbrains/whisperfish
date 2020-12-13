use actix::prelude::*;

use super::*;

/// Migration to ensure our own UUID is known.
///
/// Installs before Whisperfish 0.6 do not have their own UUID present in settings.
mod whoami;
use whoami::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Migrations;

impl Handler<Migrations> for ClientActor {
    type Result = ();
    fn handle(&mut self, _: Migrations, ctx: &mut Self::Context) {
        ctx.notify(WhoAmI);
    }
}
