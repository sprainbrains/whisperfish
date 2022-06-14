use actix::prelude::*;

use super::*;

/// Migration to ensure our own UUID is known.
///
/// Installs before Whisperfish 0.6 do not have their own UUID present in settings.
mod whoami;
use whoami::*;

/// Migration to move files in `storage/sessions` and `storage/identity` to their
/// UUID-based counterparts.
mod e164_to_uuid;
use e164_to_uuid::*;

/// Migrations related to groupv2
mod groupv2;
use groupv2::*;

/// Migration to remove R@ reactions and dump them in the correct table.
mod parse_reactions;
use parse_reactions::*;

/// Migration to get rid of file based session and identity data.
// XXX maybe the session-to-db migration should move into the store module.
pub mod session_to_db;
use session_to_db::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Migrations;

impl Handler<Migrations> for ClientActor {
    type Result = ();
    fn handle(&mut self, _: Migrations, ctx: &mut Self::Context) {
        ctx.notify(WhoAmI);
        ctx.notify(E164ToUuid);
        ctx.notify(ComputeGroupV2ExpectedIds);
        ctx.notify(GenerateEmptyProfileIfNeeded);
        ctx.notify(ParseOldReaction);
        ctx.notify(MoveSessionsToDatabase);
    }
}
