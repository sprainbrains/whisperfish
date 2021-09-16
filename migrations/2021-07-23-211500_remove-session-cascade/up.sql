-- Adds "ON DELETE CASCADE" to message_id constrait
-- on two tables: reactions, receipts.

-- Created by Matti Viljanen (direc85) 27.3.2021

-- Create new tables


DROP TABLE IF EXISTS new_reactions;
DROP TABLE IF EXISTS new_receipts;

CREATE TABLE new_reactions (
    reaction_id INTEGER PRIMARY KEY NOT NULL,

    message_id INTEGER NOT NULL,
    author INTEGER NOT NULL,

    emoji TEXT NOT NULL,
    sent_time TIMESTAMP NOT NULL,
    received_time TIMESTAMP NOT NULL,

    -- In Signal, only one emoji per author is allowed
    UNIQUE (author, message_id),

    FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE,
    FOREIGN KEY(author) REFERENCES recipients(id)
);

CREATE INDEX IF NOT EXISTS reaction_message ON reactions(message_id);
CREATE INDEX IF NOT EXISTS reaction_author ON reactions(author);



CREATE TABLE new_receipts (
    message_id INTEGER NOT NULL,
    recipient_id INTEGER NOT NULL,

    delivered TIMESTAMP,
    read TIMESTAMP,
    viewed TIMESTAMP,

    PRIMARY KEY (message_id, recipient_id),
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
    FOREIGN KEY (recipient_id) REFERENCES recipients(id)
);

CREATE INDEX IF NOT EXISTS receipt_message ON receipts(message_id);



-- Copy data to new tables

PRAGMA defer_foreign_keys = ON;

INSERT INTO new_reactions (
    message_id,
    author,
    emoji,
    sent_time,
    received_time
)
SELECT
    message_id,
    author,
    emoji,
    sent_time,
    received_time
FROM reactions;

INSERT INTO new_receipts (
    message_id,
    recipient_id,
    delivered,
    read,
    viewed
)
SELECT
    message_id,
    recipient_id,
    delivered,
    read,
    viewed
FROM new_receipts;



-- Drop the original tables

DROP TABLE reactions;
DROP TABLE receipts;



-- Rename new tables

ALTER TABLE new_reactions RENAME TO reactions;
ALTER TABLE new_receipts RENAME TO receipts;
