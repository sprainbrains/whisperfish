-- This is the database format that supports most Signal features in December 2020.
-- Big part of the structure is taken from Signal Android:
-- app/src/main/java/org/thoughtcrime/securesms/database/*.java

-- Original copyright statement for those files:
--  Copyright (C) 2011 Whisper Systems
--  Copyright (C) 2013 - 2017 Open Whisper Systems
--
--  This program is free software: you can redistribute it and/or modify
--  it under the terms of the GNU General Public License as published by
--  the Free Software Foundation, either version 3 of the License, or
--  (at your option) any later version.
--
--  This program is distributed in the hope that it will be useful,
--  but WITHOUT ANY WARRANTY; without even the implied warranty of
--  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
--  GNU General Public License for more details.
--
--  You should have received a copy of the GNU General Public License
--  along with this program.  If not, see <http://www.gnu.org/licenses/>.

----
-- 1. Rename X to X_old

ALTER TABLE sentq RENAME TO old_sentq;
ALTER TABLE message RENAME TO old_message;
ALTER TABLE session RENAME TO old_session;

-----
-- 2. Create the new structures

-- `recipients` contains the registered persons that we can talk to. Signal Android
-- interprets this table differently; they also consider a "group" as a recipient,
-- while we store groups as *separate* entities, and instead abstract over these
-- using the `session` table.
CREATE TABLE recipients (
    id INTEGER PRIMARY KEY NOT NULL,

    -- Recipient identification with Signal
    e164 VARCHAR(25) UNIQUE,
    uuid VARCHAR(36) UNIQUE,
    username TEXT UNIQUE,
    email TEXT UNIQUE,

    is_blocked BOOLEAN DEFAULT FALSE NOT NULL,

    -- Signal profile
    profile_key BLOB, -- Signal Android stores these as base64
    profile_key_credential BLOB,
    profile_given_name TEXT,
    profile_family_name TEXT,
    profile_joined_name TEXT,
    signal_profile_avatar TEXT, -- This is a pointer to the avatar, not the real thing.
    profile_sharing_enabled BOOLEAN DEFAULT FALSE NOT NULL,
    last_profile_fetch TIMESTAMP,

    unidentified_access_mode TINYINT DEFAULT 0 NOT NULL, -- 0 is UNKNOWN

    storage_service_id BLOB,
    storage_proto BLOB, -- This is set when an account update contains unknown fields

    capabilities INTEGER DEFAULT 0 NOT NULL, -- These are flags

    last_gv1_migrate_reminder TIMESTAMP,

    last_session_reset TIMESTAMP,

    -- Either e164 or uuid should be entered in recipients
    CHECK(NOT(e164 == NULL AND uuid == NULL))
);

-- Create index on UUID and e164 and other identifiers
CREATE INDEX recipient_e164 ON recipients(e164);
CREATE INDEX recipient_uuid ON recipients(uuid);
CREATE INDEX recipient_username ON recipients(username);
CREATE INDEX recipient_email ON recipients(email);

CREATE INDEX recipient_last_session_reset ON recipients(last_session_reset DESC);

-- The `v1_group` table contains the spontaneous V1 groups.
CREATE TABLE group_v1s (
    id VARCHAR(32) PRIMARY KEY NOT NULL, -- This is hex encoded. Sqlite has no HEX-decode.
    name TEXT NOT NULL
    -- Yes. Group V1 is that simple.
);

CREATE TABLE group_v1_members (
    group_v1_id VARCHAR(32) NOT NULL,
    recipient_id INTEGER NOT NULL,
    member_since TIMESTAMP, -- not sure whether we'll use this

    -- artificial primary key
    FOREIGN KEY(recipient_id) REFERENCES recipients(id), -- on delete RESTRICT because we shouldn't delete a group member because we don't like the receiver.
    PRIMARY KEY(group_v1_id, recipient_id)
);

-- The `sessions` table is a superclass of groupv1/groupv2/1:1 messages
-- When GroupV2 gets implemented, this table will be replaces once again, because
-- the constraints cannot be altered.
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY NOT NULL,

    -- Exactly one of these two (later three with groupv2) should be filed
    direct_message_recipient_id INTEGER,
    group_v1_id VARCHAR(32),

    is_archived BOOLEAN DEFAULT FALSE NOT NULL,
    is_pinned BOOLEAN DEFAULT FALSE NOT NULL,

    -- silent: notification without sound or vibration
    is_silent BOOLEAN DEFAULT FALSE NOT NULL,
    -- muted: no notification at all
    is_muted BOOLEAN DEFAULT FALSE NOT NULL,

    draft TEXT,

    expiring_message_timeout INTEGER,

    -- Deleting recipients should be separate from deleting sessions. ON DELETE RESTRICT
    FOREIGN KEY(direct_message_recipient_id) REFERENCES recipients(id),
    FOREIGN KEY(group_v1_id) REFERENCES group_v1s(id),

    -- Either a session is dm, gv1 or gv2
    CHECK (NOT(direct_message_recipient_id == NULL AND group_v1_id == NULL))
);

-- The actual messages
CREATE TABLE messages (
    id INTEGER PRIMARY KEY NOT NULL,
    session_id INTEGER NOT NULL,
    text TEXT,

    -- for group messages, this refers to the sender.
    sender_recipient_id INTEGER,

    received_timestamp TIMESTAMP,
    sent_timestamp TIMESTAMP,
    server_timestamp TIMESTAMP NOT NULL,

    -- This `is_read` flag indicates that the local user read the incoming message.
    is_read BOOLEAN DEFAULT FALSE NOT NULL,
    is_outbound BOOLEAN NOT NULL,
    flags INTEGER NOT NULL,

    -- expiring messages
    -- NOT NULL means that the message gets deleted at `expires_in + expiry_started`.
    expires_in INTEGER,
    expiry_started TIMESTAMP,

    -- scheduled messages
    schedule_send_time TIMESTAMP,

    is_bookmarked BOOLEAN DEFAULT FALSE NOT NULL,

    -- misc flags
    use_unidentified BOOLEAN DEFAULT FALSE NOT NULL,
    is_remote_deleted BOOLEAN DEFAULT FALSE NOT NULL,

    FOREIGN KEY(sender_recipient_id) REFERENCES recipients(id) ON DELETE CASCADE,
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE TRIGGER validate_group_message_has_sender
  BEFORE INSERT ON messages
BEGIN
  SELECT
    RAISE (ABORT, 'sender of inbound group message is not set')
  WHERE EXISTS (
    SELECT
      group_v1_id IS NOT NULL AS is_group,
      NOT NEW.is_outbound AS is_inbound
    FROM sessions
    WHERE sessions.id = NEW.session_id
      AND is_group
      AND is_inbound
      AND NEW.sender_recipient_id IS NULL
  );
END;

-- Index the timestamps of message
CREATE INDEX message_received ON messages(received_timestamp);
CREATE INDEX message_sent ON messages(sent_timestamp);
CREATE INDEX message_server ON messages(server_timestamp);

CREATE TABLE attachments (
    id INTEGER PRIMARY KEY NOT NULL,
    json TEXT,
    message_id INTEGER NOT NULL,
    content_type TEXT DEFAULT "" NOT NULL,
    name TEXT,
    content_disposition TEXT,
    content_location TEXT,
    attachment_path TEXT,
    is_pending_upload BOOLEAN DEFAULT FALSE NOT NULL,
    transfer_file_path TEXT,
    size INTEGER,
    file_name TEXT,
    unique_id TEXT,
    digest TEXT,
    is_voice_note BOOLEAN NOT NULL,
    is_borderless BOOLEAN NOT NULL,
    is_quote BOOLEAN NOT NULL,

    width INTEGER,
    height INTEGER,

    sticker_pack_id TEXT DEFAULT NULL,
    sticker_pack_key BLOB DEFAULT NULL,
    sticker_id INTEGER DEFAULT NULL,
    sticker_emoji TEXT DEFAULT NULL,

    data_hash BLOB,
    visual_hash TEXT,
    transform_properties TEXT,

    -- This is the encrypted file, used for resumable uploads (#107)
    transfer_file TEXT,
    display_order INTEGER DEFAULT 0 NOT NULL,
    -- default is timestamp of this migration.
    upload_timestamp TIMESTAMP DEFAULT "2021-02-14T18:05:49Z" NOT NULL,
    cdn_number INTEGER DEFAULT 0,

    FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE,
    FOREIGN KEY(sticker_pack_id, sticker_id) REFERENCES stickers(pack_id, sticker_id) ON DELETE CASCADE
);

CREATE TABLE stickers (
    pack_id TEXT,
    sticker_id INTEGER NOT NULL,
    -- Cover is the ID of the sticker of this pack to be used as "cover".
    cover_sticker_id INTEGER NOT NULL,

    key BLOB NOT NULL,

    title TEXT NOT NULL,
    author TEXT NOT NULL,

    pack_order INTEGER NOT NULL,
    emoji TEXT NOT NULL,
    content_type TEXT,
    last_used TIMESTAMP NOT NULL,
    installed TIMESTAMP NOT NULL,
    file_path TEXT NOT NULL,
    file_length INTEGER NOT NULL,
    file_random BLOB NOT NULL,

    PRIMARY KEY(pack_id, sticker_id),
    FOREIGN KEY(pack_id, cover_sticker_id) REFERENCES stickers(pack_id, sticker_id) ON DELETE CASCADE,
    UNIQUE(pack_id, sticker_id, cover_sticker_id)
);


CREATE TABLE reactions (
    reaction_id INTEGER PRIMARY KEY NOT NULL,

    message_id INTEGER NOT NULL,
    author INTEGER NOT NULL,

    emoji TEXT NOT NULL,
    sent_time TIMESTAMP NOT NULL,
    received_time TIMESTAMP NOT NULL,

    -- In Signal, only one emoji per author is allowed
    UNIQUE (author, message_id),

    FOREIGN KEY(message_id) REFERENCES messages(id),
    FOREIGN KEY(author) REFERENCES recipients(id)
);

CREATE INDEX reaction_message ON reactions(message_id);
CREATE INDEX reaction_author ON reactions(author);

CREATE TABLE receipts (
    message_id INTEGER NOT NULL,
    recipient_id INTEGER NOT NULL,

    delivered TIMESTAMP,
    read TIMESTAMP,
    viewed TIMESTAMP,

    PRIMARY KEY (message_id, recipient_id),
    FOREIGN KEY (message_id) REFERENCES messages(id),
    FOREIGN KEY (recipient_id) REFERENCES recipients(id)
);

---
-- 3. Copy over the data

-- Create a view for the group members.
-- The TEMPORARY view is automatically destroyed
-- at the end of the connection.
CREATE TEMPORARY VIEW old_group_members AS
WITH split(group_id, word, str) AS (
    SELECT
        group_id, '', group_members||','
        FROM old_session
        WHERE is_group != 0
            AND group_members IS NOT NULL
            AND group_members != ""
    UNION ALL
    SELECT
        group_id,
        substr(str, 0, instr(str, ',')),
        substr(str, instr(str, ',')+1)
    FROM split
    WHERE str!=''
)
SELECT
    group_id,
    word
AS group_member_e164
FROM split
WHERE word != '';

-- We have two sources of recipients.
-- - 1:1/direct messages, i.e. the `source` field of `old_session`.
INSERT INTO recipients (
    e164
)
SELECT
    source
FROM old_session
WHERE source IS NOT NULL and source != "";

-- - group messages, i.e. the `group_members` field of `old_session`.
INSERT OR IGNORE INTO recipients (
    e164
) SELECT DISTINCT (
    group_member_e164
) FROM old_group_members;

-- 🫂  Create the groups 🫂
INSERT INTO group_v1s (
    id,
    name
)
SELECT group_id, group_name
FROM old_session
WHERE is_group;

INSERT INTO group_v1_members (
    group_v1_id,
    recipient_id
)
SELECT group_id, recipients.id
FROM old_group_members
LEFT JOIN recipients ON old_group_members.group_member_e164 == recipients.e164;

-- For sessions, too, we have two sources.
-- They both come from old_session. 🐎 Hold your horses 🐎
-- - 🐎 1:1 sessions have to be tied to the recipient table
INSERT INTO sessions (
    direct_message_recipient_id
)
SELECT recipients.id
FROM old_session
-- left join will put NULL for recipient,
-- which will fail the foreign key
LEFT JOIN recipients ON recipients.e164 == old_session.source
WHERE NOT old_session.is_group;
-- - 🐎 groups have to be tied to the group_v1s table
INSERT INTO sessions (
    group_v1_id
)
SELECT group_id
FROM old_session
WHERE old_session.is_group;

-- ✉️  And finally the messages ✉️
-- This is also split in groups and 1:1,
-- because of the lack of SQL-skilzz
INSERT INTO messages (
    session_id,
    text,

    received_timestamp,
    sent_timestamp,
    server_timestamp,

    is_read,
    is_outbound,
    flags
)
SELECT
    sessions.id,
    old_message.message,

    -- received timestamp
    CASE WHEN old_message.outgoing == 0
        THEN old_message.timestamp
        ELSE NULL
    END,
    -- sent timestamp
    CASE WHEN old_message.outgoing
        THEN old_message.timestamp
        ELSE NULL
    END,
    -- server timestamp
    old_message.timestamp,

    old_message.received,
    old_message.outgoing,
    old_message.flags
FROM old_message
-- Left join again ensures that we fail more foreign keys
LEFT JOIN old_session ON old_session.id == old_message.session_id
LEFT JOIN recipients ON recipients.e164 == old_session.source
LEFT JOIN sessions ON sessions.direct_message_recipient_id == recipients.id
WHERE NOT old_session.is_group;

INSERT INTO messages (
    session_id,
    text,

    sender_recipient_id,

    received_timestamp,
    sent_timestamp,
    server_timestamp,

    is_read,
    is_outbound,
    flags
)
SELECT
    sessions.id,
    old_message.message,

    -- sender id
    CASE WHEN old_message.outgoing == 0
        THEN recipients.id
        ELSE NULL
    END,

    -- received timestamp
    CASE WHEN old_message.outgoing == 0
        THEN old_message.timestamp
        ELSE NULL
    END,
    -- sent timestamp
    CASE WHEN old_message.outgoing AND old_sentq.timestamp IS NULL
        THEN old_message.timestamp
        ELSE NULL
    END,
    -- server timestamp
    old_message.timestamp,

    old_message.received,
    old_message.outgoing,
    old_message.flags
FROM old_message
-- Use a LEFT JOIN , because when old_message.source = NULL or empty string, we still want the message.
LEFT JOIN recipients ON recipients.e164 == old_message.source
LEFT JOIN old_session ON old_session.id == old_message.session_id
LEFT JOIN sessions ON sessions.group_v1_id == old_session.group_id
LEFT JOIN old_sentq ON old_message.id == old_sentq.message_id
WHERE  old_session.is_group;

INSERT INTO attachments (
    message_id,
    attachment_path,
    content_type,

    is_voice_note,
    is_borderless,
    is_quote
)
SELECT
    messages.id,
    old_message.attachment,
    CASE WHEN old_message.mime_type IS NULL
        THEN ""
        ELSE old_message.mime_type
    END,

    0,
    0,
    0
FROM messages, old_message
WHERE messages.server_timestamp == old_message.timestamp
    AND old_message.has_attachment;

-- For outbound 1:1 messages, we have read-receipts.
-- We don't have their date, but we just set it on UNIX epoch as marker.
INSERT OR IGNORE INTO receipts (
    message_id,
    recipient_id,

    delivered -- we called this "received"
    -- read receipts and view receipts weren't handled before.
)
SELECT
    messages.id,
    recipients.id,

    CASE WHEN old_message.received
        THEN '1970-01-01T00:00:00'
        ELSE NULL
    END

FROM sessions, messages, recipients, old_message
WHERE messages.server_timestamp == old_message.timestamp
    AND messages.session_id == sessions.id
    AND recipients.id == sessions.direct_message_recipient_id
    AND messages.is_outbound;

-- We don't care about group messages, since this is information we did not retain. Sorry!

-- 4. Drop the old tables

DROP TABLE old_sentq;
DROP TABLE old_message;
DROP TABLE old_session;
