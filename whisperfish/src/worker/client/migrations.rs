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
use std::sync::Arc;
use tokio::sync::{Notify, RwLock};

#[derive(Clone)]
pub(super) struct MigrationCondVar {
    state: Arc<RwLock<MigrationState>>,
    notify: Arc<Notify>,
}

impl MigrationCondVar {
    pub fn new() -> Self {
        MigrationCondVar {
            state: Arc::new(RwLock::new(MigrationState::new())),
            notify: Arc::new(Notify::new()),
        }
    }
}

pub(super) struct MigrationState {
    pub whoami: bool,
    pub protocol_store_in_db: bool,
    pub sessions_have_uuid: bool,
    pub gv2_expected_ids: bool,
    pub self_profile_ready: bool,
    pub reactions_ready: bool,
}

impl MigrationState {
    fn new() -> MigrationState {
        MigrationState {
            whoami: false,
            protocol_store_in_db: false,
            sessions_have_uuid: false,
            gv2_expected_ids: false,
            self_profile_ready: false,
            reactions_ready: false,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.whoami
            && self.protocol_store_in_db
            && self.sessions_have_uuid
            && self.gv2_expected_ids
            && self.self_profile_ready
            && self.reactions_ready
    }
}

macro_rules! method_for_condition {
    ($method:ident : $state:ident -> $cond:expr) => {
        pub fn $method(&self) -> impl Future<Output = ()> + 'static {
            let notify = self.notify.clone();
            let state = self.state.clone();

            async move {
                while {
                    let $state = state.read().await;
                    $cond
                } {
                    notify.notified().await;
                }
            }
        }
    };
    ($name:ident) => {
        method_for_condition!($name : state -> state.$name);
    }
}

macro_rules! notify_method_for_var {
    ($method:ident -> $var:ident) => {
        pub fn $method(&self) {
            let notify = self.notify.clone();
            let state = self.state.clone();
            actix::spawn(async move {
                state.write().await.$var = true;
                notify.notify_waiters();
            });
        }
    };
}

impl MigrationCondVar {
    method_for_condition!(ready : state -> state.is_ready());
    method_for_condition!(self_uuid_is_known : state -> state.whoami);
    method_for_condition!(protocol_store_in_db);

    notify_method_for_var!(notify_whoami -> whoami);
    notify_method_for_var!(notify_protocol_store_in_db -> protocol_store_in_db);
    notify_method_for_var!(notify_e164_to_uuid -> sessions_have_uuid);
    notify_method_for_var!(notify_groupv2_expected_ids -> gv2_expected_ids);
    notify_method_for_var!(notify_self_profile_ready -> self_profile_ready);
    notify_method_for_var!(notify_reactions_ready -> reactions_ready);
}

impl ClientActor {
    pub(super) fn queue_migrations(ctx: &mut <Self as Actor>::Context) {
        ctx.notify(WhoAmI);
        ctx.notify(MoveSessionsToDatabase);
        ctx.notify(E164ToUuid);
        ctx.notify(ComputeGroupV2ExpectedIds);
        ctx.notify(RefreshOwnProfile { force: false });
        ctx.notify(ParseOldReaction);
    }
}
