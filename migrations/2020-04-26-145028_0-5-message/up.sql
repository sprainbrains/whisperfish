CREATE TABLE IF NOT EXISTS message (
    id INTEGER PRIMARY KEY,
    session_id INTEGER,
    source TEXT,
    message STRING,
    timestamp INTEGER,
    sent INTEGER DEFAULT 0,
    received INTEGER DEFAULT 0,
    flags INTEGER DEFAULT 0,
    attachment TEXT,
    mime_type STRING,
    has_attachment INTEGER DEFAULT 0,
    outgoing INTEGER DEFAULT 0
);
