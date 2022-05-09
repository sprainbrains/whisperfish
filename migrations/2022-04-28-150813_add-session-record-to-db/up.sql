ALTER TABLE recipients
    ADD COLUMN identity BLOB;

CREATE TABLE session_records (
    recipient_id INTEGER NOT NULL,
    device_id INTEGER NOT NULL,
    record BLOB NOT NULL,

    FOREIGN KEY(recipient_id) REFERENCES recipients(id) ON DELETE CASCADE,
    PRIMARY KEY(recipient_id, device_id)
)
