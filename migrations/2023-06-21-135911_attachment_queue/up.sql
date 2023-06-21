-- Your SQL goes here
ALTER TABLE attachments
    ADD COLUMN pointer BLOB DEFAULT NULL;
