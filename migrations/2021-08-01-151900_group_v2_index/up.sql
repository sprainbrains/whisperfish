-- This is only used in group migrations,
-- but as there are multiple versions of GroupV2's
-- already, it's worth having.
CREATE INDEX group_v2_member_recipient_id ON group_v2_members(recipient_id DESC);

CREATE INDEX receipt_recipient_id ON receipts(recipient_id);
CREATE INDEX receipt_message_id ON receipts(message_id);
CREATE INDEX group_v1_members_v1_id ON group_v1_members(group_v1_id);
CREATE INDEX group_v2_members_v2_id ON group_v2_members(group_v2_id);
CREATE INDEX session_group_v1_id ON sessions(group_v1_id);
CREATE INDEX session_group_v2_id ON sessions(group_v2_id);
CREATE INDEX session_dm_recipient_id ON sessions(direct_message_recipient_id DESC);
CREATE INDEX attachment_message_id ON attachments(message_id);
