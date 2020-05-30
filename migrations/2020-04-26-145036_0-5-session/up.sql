CREATE TABLE session (
    id INTEGER PRIMARY KEY,
    source TEXT,
    message STRING,
    timestamp INTEGER,
    sent INTEGER DEFAULT 0,
    received INTEGER DEFAULT 0,
    unread INTEGER DEFAULT 0,
    is_group INTEGER DEFAULT 0,
    group_members TEXT,
    group_id TEXT,
    group_name TEXT,
    has_attachment INTEGER DEFAULT 0
);
