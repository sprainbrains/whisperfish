CREATE TABLE session (
    id INTEGER PRIMARY KEY,
-- source used to be STRING
    source TEXT,
-- message used to be STRING
    message TEXT,
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
