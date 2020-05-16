CREATE TABLE message (
    id INTEGER PRIMARY KEY,
    session_id INTEGER,
    source TEXT,
-- message used to say STRING
    message TEXT,
    timestamp INTEGER,
    sent INTEGER DEFAULT 0,
    received INTEGER DEFAULT 0,
    flags INTEGER DEFAULT 0,
    attachment TEXT,
-- mime_type used to say STRING
    mime_type TEXT,
    has_attachment INTEGER DEFAULT 0,
    outgoing INTEGER DEFAULT 0
);
