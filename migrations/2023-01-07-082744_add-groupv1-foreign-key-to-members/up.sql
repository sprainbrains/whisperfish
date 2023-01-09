-- This adds a group_v1_members.group_v1_id foreign key.
ALTER TABLE group_v1_members RENAME TO group_v1_members_old;

CREATE TABLE group_v1_members (
    group_v1_id VARCHAR(32) NOT NULL,
    recipient_id INTEGER NOT NULL,
    member_since TIMESTAMP, -- not sure whether we'll use this

    -- artificial primary key
    FOREIGN KEY(group_v1_id) REFERENCES group_v1s(id),
    FOREIGN KEY(recipient_id) REFERENCES recipients(id),
    PRIMARY KEY(group_v1_id, recipient_id)
);

INSERT INTO group_v1_members (
    group_v1_id, recipient_id, member_since
)
SELECT
    group_v1_id, recipient_id, member_since
FROM group_v1_members_old;

DROP TABLE group_v1_members_old;
