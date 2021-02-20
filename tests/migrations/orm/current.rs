use std::time::Duration;

use chrono::prelude::*;

use super::schemas::current::*;

#[derive(Queryable, Insertable, Debug, Clone)]
pub struct GroupV1 {
    pub id: String,
    pub name: String,
}

#[derive(Queryable, Insertable, Debug, Clone)]
pub struct GroupV1Member {
    pub group_v1_id: String,
    pub recipient_id: i32,
    pub member_since: Option<NaiveDateTime>,
}

#[derive(Queryable, Debug, Clone)]
pub struct Message {
    pub id: i32,
    pub session_id: i32,

    pub text: Option<String>,
    pub sender_recipient_id: Option<i32>,

    pub received_timestamp: Option<NaiveDateTime>,
    pub sent_timestamp: Option<NaiveDateTime>,
    pub server_timestamp: Option<NaiveDateTime>,
    pub is_read: bool,
    pub is_outbound: bool,
    pub flags: i32,
    pub expires_in: Option<i32>,
    pub expiry_started: Option<NaiveDateTime>,
    pub schedule_send_time: Option<NaiveDateTime>,
    pub is_bookmarked: bool,
    pub use_unidentified: bool,
    pub is_remote_deleted: bool,
}

#[derive(Queryable, Debug, Clone)]
pub struct Recipient {
    pub id: i32,
    pub e164: Option<String>,
    pub uuid: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub blocked: bool,

    pub profile_key: Option<Vec<u8>>,
    pub profile_key_credential: Option<Vec<u8>>,

    pub profile_given_name: Option<String>,
    pub profile_family_name: Option<String>,
    pub profile_joined_name: Option<String>,
    pub signal_profile_avatar: Option<String>,
    pub profile_sharing: bool,

    pub last_profile_fetch: Option<NaiveDateTime>,
    pub unidentified_access_mode: bool,

    pub storage_service_id: Option<Vec<u8>>,
    pub storage_proto: Option<Vec<u8>>,

    pub capabilities: i32,
    pub last_gv1_migrate_reminder: Option<NaiveDateTime>,
    pub last_session_reset: Option<NaiveDateTime>,
}

#[derive(Queryable, Debug, Clone)]
pub struct DbSession {
    pub id: i32,

    pub direct_message_recipient_id: Option<i32>,
    pub group_v1_id: Option<String>,

    pub is_archived: bool,
    pub is_pinned: bool,

    pub is_silent: bool,
    pub is_muted: bool,

    pub draft: Option<String>,

    pub expiring_message_timeout: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: i32,

    pub is_archived: bool,
    pub is_pinned: bool,

    pub is_silent: bool,
    pub is_muted: bool,

    pub expiring_message_timeout: Option<Duration>,

    pub draft: Option<String>,
    pub r#type: SessionType,
}

impl Session {
    pub fn is_dm(&self) -> bool {
        self.r#type.is_dm()
    }

    pub fn is_group_v1(&self) -> bool {
        self.r#type.is_group_v1()
    }

    pub fn unwrap_dm(&self) -> &Recipient {
        self.r#type.unwrap_dm()
    }

    pub fn unwrap_group_v1(&self) -> &GroupV1 {
        self.r#type.unwrap_group_v1()
    }
}

impl From<(DbSession, Option<Recipient>, Option<GroupV1>)> for Session {
    fn from(
        (session, recipient, groupv1): (DbSession, Option<Recipient>, Option<GroupV1>),
    ) -> Session {
        assert_eq!(
            session.direct_message_recipient_id.is_some(),
            recipient.is_some(),
            "direct session requires recipient"
        );
        assert_eq!(
            session.group_v1_id.is_some(),
            groupv1.is_some(),
            "groupv1 session requires groupv1"
        );

        let t = match (recipient, groupv1) {
            (Some(recipient), None) => SessionType::DirectMessage(recipient),
            (None, Some(gv1)) => SessionType::GroupV1(gv1),
            _ => unreachable!("case handled above"),
        };

        let DbSession {
            id,

            direct_message_recipient_id: _,
            group_v1_id: _,

            is_archived,
            is_pinned,

            is_silent,
            is_muted,

            draft,

            expiring_message_timeout,
        } = session;
        Session {
            id,

            is_archived,
            is_pinned,

            is_silent,
            is_muted,

            draft,

            expiring_message_timeout: expiring_message_timeout
                .map(|i| i as u64)
                .map(Duration::from_secs),

            r#type: t,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SessionType {
    DirectMessage(Recipient),
    GroupV1(GroupV1),
}

impl SessionType {
    pub fn is_dm(&self) -> bool {
        matches!(self, Self::DirectMessage(_))
    }

    pub fn is_group_v1(&self) -> bool {
        matches!(self, Self::GroupV1(_))
    }

    pub fn unwrap_dm(&self) -> &Recipient {
        assert!(self.is_dm(), "unwrap panicked at unwrap_dm()");
        match self {
            Self::DirectMessage(r) => r,
            _ => unreachable!(),
        }
    }

    pub fn unwrap_group_v1(&self) -> &GroupV1 {
        assert!(self.is_group_v1(), "unwrap panicked at unwrap_group_v1()");
        match self {
            Self::GroupV1(g) => g,
            _ => unreachable!(),
        }
    }
}
