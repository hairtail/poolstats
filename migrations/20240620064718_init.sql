-- Add migration script here
CREATE TABLE IF NOT EXISTS poet_registration (
    id CHAR(32) NOT NULL,
    round_id VARCHAR NOT NULL,
    num_unit INT NOT NULL,
    PRIMARY KEY (id)
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS atxs (
    id CHAR(32),
    epoch INT NOT NULL,
    num_unit INT NOT NULL,
    effective_num_units INT NOT NULL,
    coinbase CHAR(24),
    atx_id CHAR(32) PRIMARY KEY
) WITHOUT ROWID;