-- These tables are *almost* directly linked to recipients,
-- but the implied trust relations and logic are quite impossible to model in SQL.
-- libsignal-client handles 99% of that for us.

CREATE TABLE session_records (
    address TEXT NOT NULL,
    device_id INTEGER NOT NULL,
    record BLOB NOT NULL,

    PRIMARY KEY(address, device_id)
);

CREATE TABLE identity_records (
    address TEXT NOT NULL,
    record BLOB NOT NULL,

    -- TODO: Signal adds a lot more fields here that I don't yet care about.

    PRIMARY KEY(address)
);
