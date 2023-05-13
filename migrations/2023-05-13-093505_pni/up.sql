ALTER TABLE recipients
    ADD COLUMN pni VARCHAR(36);
ALTER TABLE recipients
    ADD COLUMN needs_pni_signature BOOLEAN DEFAULT FALSE NOT NULL;

CREATE INDEX recipient_pni ON recipients(pni);
