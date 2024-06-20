-- Add migration script here
CREATE TABLE IF NOT EXISTS initial_post (
    id CHAR(80) PRIMARY KEY,
    status INTEGER NOT NULL,
    data TEXT,
    ts TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS registerations (
    id CHAR(80) PRIMARY KEY,
    status INTEGER NOT NULL,
    data TEXT,
    ts TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS atxs (
    id CHAR(80) PRIMARY KEY,
    status INTEGER NOT NULL,
    data TEXT,
    ts TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
) WITHOUT ROWID;