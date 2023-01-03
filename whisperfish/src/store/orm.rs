use super::schema::*;
use chrono::prelude::*;
use std::time::Duration;

#[derive(Queryable, Insertable, Debug, Clone)]
pub struct GroupV1 {
    pub id: String,
    pub name: String,
    pub expected_v2_id: Option<String>,
}

#[derive(Queryable, Insertable, Debug, Clone)]
pub struct GroupV1Member {
    pub group_v1_id: String,
    pub recipient_id: i32,
    pub member_since: Option<NaiveDateTime>,
}

#[derive(Queryable, Insertable, Debug, Clone)]
pub struct GroupV2 {
    pub id: String,
    pub name: String,

    pub master_key: String,
    pub revision: i32,

    pub invite_link_password: Option<Vec<u8>>,

    pub access_required_for_attributes: i32,
    pub access_required_for_members: i32,
    pub access_required_for_add_from_invite_link: i32,

    pub avatar: Option<String>,
    pub description: Option<String>,
}

#[derive(Queryable, Insertable, Debug, Clone)]
pub struct GroupV2Member {
    pub group_v2_id: String,
    pub recipient_id: i32,
    pub member_since: NaiveDateTime,
    pub joined_at_revision: i32,
    pub role: i32,
}

#[derive(Queryable, Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: i32,
    pub session_id: i32,

    pub text: Option<String>,
    pub sender_recipient_id: Option<i32>,

    pub received_timestamp: Option<NaiveDateTime>,
    pub sent_timestamp: Option<NaiveDateTime>,
    pub server_timestamp: NaiveDateTime,
    pub is_read: bool,
    pub is_outbound: bool,
    pub flags: i32,
    pub expires_in: Option<i32>,
    pub expiry_started: Option<NaiveDateTime>,
    pub schedule_send_time: Option<NaiveDateTime>,
    pub is_bookmarked: bool,
    pub use_unidentified: bool,
    pub is_remote_deleted: bool,

    pub sending_has_failed: bool,

    pub quote_id: Option<i32>,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            id: Default::default(),
            session_id: Default::default(),
            text: Default::default(),
            sender_recipient_id: Default::default(),
            received_timestamp: Default::default(),
            sent_timestamp: Default::default(),
            server_timestamp: NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
            is_read: Default::default(),
            is_outbound: Default::default(),
            flags: Default::default(),
            expires_in: Default::default(),
            expiry_started: Default::default(),
            schedule_send_time: Default::default(),
            is_bookmarked: Default::default(),
            use_unidentified: Default::default(),
            is_remote_deleted: Default::default(),
            sending_has_failed: Default::default(),
            quote_id: Default::default(),
        }
    }
}

#[derive(Queryable, Identifiable, Debug, Clone)]
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

    pub about: Option<String>,
    pub about_emoji: Option<String>,
}

#[derive(Queryable, Identifiable, Insertable, Debug, Clone)]
#[diesel(primary_key(address, device_id))]
pub struct SessionRecord {
    pub address: String,
    pub device_id: i32,
    pub record: Vec<u8>,
}

#[derive(Queryable, Identifiable, Insertable, Debug, Clone)]
#[diesel(primary_key(address))]
pub struct IdentityRecord {
    pub address: String,
    pub record: Vec<u8>,
}

#[derive(Queryable, Identifiable, Insertable, Debug, Clone)]
pub struct SignedPrekey {
    pub id: i32,
    pub record: Vec<u8>,
}

#[derive(Queryable, Identifiable, Insertable, Debug, Clone)]
pub struct Prekey {
    pub id: i32,
    pub record: Vec<u8>,
}

#[derive(Queryable, Identifiable, Insertable, Debug, Clone)]
#[diesel(primary_key(address, device, distribution_id))]
pub struct SenderKeyRecord {
    pub address: String,
    pub device: i32,
    pub distribution_id: String,
    pub record: Vec<u8>,
    pub created_at: NaiveDateTime,
}

impl Recipient {
    pub fn profile_key(&self) -> Option<[u8; 32]> {
        if let Some(pk) = self.profile_key.as_ref() {
            if pk.len() != 32 {
                log::warn!("Profile key is {} bytes", pk.len());
                None
            } else {
                let mut key = [0u8; 32];
                key.copy_from_slice(pk);
                Some(key)
            }
        } else {
            None
        }
    }

    pub fn to_service_address(&self) -> libsignal_service::ServiceAddress {
        libsignal_service::ServiceAddress {
            phonenumber: self
                .e164
                .as_ref()
                .map(|e164| phonenumber::parse(None, e164).expect("only valid phone number in db")),
            relay: None,
            uuid: self
                .uuid
                .as_ref()
                .map(|uuid| uuid::Uuid::parse_str(uuid).expect("only valid UUIDs in db")),
        }
    }

    pub fn uuid(&self) -> &str {
        self.uuid.as_deref().or(Some("")).expect("uuid")
    }

    pub fn e164_or_uuid(&self) -> &str {
        self.e164
            .as_deref()
            .or(self.uuid.as_deref())
            .expect("either uuid or e164")
    }

    pub fn name(&self) -> &str {
        self.profile_joined_name
            .as_deref()
            .or_else(|| Some(self.e164_or_uuid()))
            .expect("either joined name, uuid or e164")
    }
}

#[derive(Queryable, Debug, Clone)]
pub struct DbSession {
    pub id: i32,

    pub direct_message_recipient_id: Option<i32>,
    pub group_v1_id: Option<String>,
    pub group_v2_id: Option<String>,

    pub is_archived: bool,
    pub is_pinned: bool,

    pub is_silent: bool,
    pub is_muted: bool,

    pub draft: Option<String>,

    pub expiring_message_timeout: Option<i32>,
}

#[derive(Queryable, Debug, Clone)]
pub struct Attachment {
    pub id: i32,
    pub json: Option<String>,
    pub message_id: i32,
    pub content_type: String,
    pub name: Option<String>,
    pub content_disposition: Option<String>,
    pub content_location: Option<String>,
    pub attachment_path: Option<String>,
    pub is_pending_upload: bool,
    pub transfer_file_path: Option<String>,
    pub size: Option<i32>,
    pub file_name: Option<String>,
    pub unique_id: Option<String>,
    pub digest: Option<String>,
    pub is_voice_note: bool,
    pub is_borderless: bool,
    pub is_quote: bool,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub sticker_pack_id: Option<String>,
    pub sticker_pack_key: Option<Vec<u8>>,
    pub sticker_id: Option<i32>,
    pub sticker_emoji: Option<String>,
    pub data_hash: Option<Vec<u8>>,
    pub visual_hash: Option<String>,
    pub transform_properties: Option<String>,
    pub transfer_file: Option<String>,
    pub display_order: i32,
    pub upload_timestamp: NaiveDateTime,
    pub cdn_number: Option<i32>,
    pub caption: Option<String>,
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

#[derive(Queryable, Debug, Clone)]
pub struct Reaction {
    pub reaction_id: i32,
    pub message_id: i32,
    pub author: i32,
    pub emoji: String,
    pub sent_time: NaiveDateTime,
    pub received_time: NaiveDateTime,
}

#[derive(Queryable, Debug, Clone)]
pub struct Receipt {
    pub message_id: i32,
    pub recipient_id: i32,
    pub delivered: Option<NaiveDateTime>,
    pub read: Option<NaiveDateTime>,
    pub viewed: Option<NaiveDateTime>,
}

impl Session {
    pub fn is_dm(&self) -> bool {
        self.r#type.is_dm()
    }

    pub fn is_group(&self) -> bool {
        self.r#type.is_group_v1() || self.r#type.is_group_v2()
    }

    pub fn is_group_v1(&self) -> bool {
        self.r#type.is_group_v1()
    }

    pub fn is_group_v2(&self) -> bool {
        self.r#type.is_group_v2()
    }

    pub fn unwrap_dm(&self) -> &Recipient {
        self.r#type.unwrap_dm()
    }

    pub fn unwrap_group_v1(&self) -> &GroupV1 {
        self.r#type.unwrap_group_v1()
    }

    pub fn unwrap_group_v2(&self) -> &GroupV2 {
        self.r#type.unwrap_group_v2()
    }
}

impl
    From<(
        DbSession,
        Option<Recipient>,
        Option<GroupV1>,
        Option<GroupV2>,
    )> for Session
{
    fn from(
        (session, recipient, groupv1, groupv2): (
            DbSession,
            Option<Recipient>,
            Option<GroupV1>,
            Option<GroupV2>,
        ),
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
        assert_eq!(
            session.group_v2_id.is_some(),
            groupv2.is_some(),
            "groupv2 session requires groupv2"
        );

        let t = match (recipient, groupv1, groupv2) {
            (Some(recipient), None, None) => SessionType::DirectMessage(recipient),
            (None, Some(gv1), None) => SessionType::GroupV1(gv1),
            (None, None, Some(gv2)) => SessionType::GroupV2(gv2),
            _ => unreachable!("case handled above"),
        };

        let DbSession {
            id,

            direct_message_recipient_id: _,
            group_v1_id: _,
            group_v2_id: _,

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
#[allow(clippy::large_enum_variant)]
pub enum SessionType {
    // XXX clippy suggests to put Recipient, 322 bytes, on the heap.
    DirectMessage(Recipient),
    GroupV1(GroupV1),
    GroupV2(GroupV2),
}

impl SessionType {
    pub fn is_dm(&self) -> bool {
        matches!(self, Self::DirectMessage(_))
    }

    pub fn is_group_v1(&self) -> bool {
        matches!(self, Self::GroupV1(_))
    }

    pub fn is_group_v2(&self) -> bool {
        matches!(self, Self::GroupV2(_))
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

    pub fn unwrap_group_v2(&self) -> &GroupV2 {
        assert!(self.is_group_v2(), "unwrap panicked at unwrap_group_v2()");
        match self {
            Self::GroupV2(g) => g,
            _ => unreachable!(),
        }
    }
}

// Some extras

/// [`Message`] augmented with its sender, attachments, reactions and receipts.
#[derive(Clone, Default)]
pub struct AugmentedMessage {
    pub inner: Message,
    pub sender: Option<Recipient>,
    pub attachments: Vec<Attachment>,
    pub reactions: Vec<(Reaction, Recipient)>,
    pub receipts: Vec<(Receipt, Recipient)>,
    // Constraint: don't make this nested more than one level deep.
    pub quoted_message: Option<Box<AugmentedMessage>>,
}

impl std::ops::Deref for AugmentedMessage {
    type Target = Message;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AugmentedMessage {
    pub fn name(&self) -> &str {
        if let Some(sender) = &self.sender {
            sender.name()
        } else {
            ""
        }
    }

    pub fn sent(&self) -> bool {
        self.inner.sent_timestamp.is_some()
    }

    pub fn delivered(&self) -> u32 {
        self.receipts
            .iter()
            .filter(|(r, _)| r.delivered.is_some())
            .count() as _
    }

    pub fn read(&self) -> u32 {
        self.receipts
            .iter()
            .filter(|(r, _)| r.read.is_some())
            .count() as _
    }

    pub fn viewed(&self) -> u32 {
        self.receipts
            .iter()
            .filter(|(r, _)| r.viewed.is_some())
            .count() as _
    }

    pub fn queued(&self) -> bool {
        self.is_outbound && self.sent_timestamp.is_none() && !self.sending_has_failed
    }

    pub fn attachments(&self) -> u32 {
        self.attachments.len() as _
    }

    pub fn reactions(&self) -> String {
        use itertools::Itertools;
        self.reactions
            .iter()
            .map(|(reaction, _recipient)| &reaction.emoji)
            .join(" ")
    }

    pub fn reactions_full(&self) -> String {
        use itertools::Itertools;
        self.reactions
            .iter()
            .map(|(reaction, recipient)| format!("{} - {}", &reaction.emoji, &recipient.name()))
            .join("\n")
    }
}
