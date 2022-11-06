CREATE TABLE sender_key_records (
    address TEXT NOT NULL,
    device INTEGER NOT NULL,
    distribution_id TEXT NOT NULL,
    record BLOB NOT NULL,
    created_at TIMESTAMP NOT NULL,

    PRIMARY KEY(address, device, distribution_id),
    UNIQUE(address, device, distribution_id) ON CONFLICT REPLACE
);
