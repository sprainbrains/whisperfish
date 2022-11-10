/// Migration to get rid of file based session and identity data.
// XXX maybe the session-to-db migration should move into the store module.
pub mod session_to_db;

/// Migration to move files in `storage/sessions` and `storage/identity` to their
/// UUID-based counterparts.
mod e164_to_uuid;
/// Migrations related to groupv2
mod groupv2;
/// Migration to remove R@ reactions and dump them in the correct table.
mod parse_reactions;
/// Migration to ensure our own UUID is known.
///
/// Installs before Whisperfish 0.6 do not have their own UUID present in settings.
mod whoami;

use self::e164_to_uuid::*;
use self::groupv2::*;
use self::parse_reactions::*;
use self::session_to_db::*;
use self::whoami::*;
use super::*;
use actix::prelude::*;

impl ClientActor {
    pub(super) fn queue_migrations(ctx: &mut <Self as Actor>::Context) {
        ctx.notify(WhoAmI);
        ctx.notify(MoveSessionsToDatabase);
        ctx.notify(E164ToUuid);
        ctx.notify(ComputeGroupV2ExpectedIds);
        ctx.notify(RefreshOwnProfile);
        ctx.notify(ParseOldReaction);
    }
}
