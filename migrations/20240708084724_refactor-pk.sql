-- Add migration script here
CREATE TABLE poet_registration_new (
    id CHAR(32) NOT NULL,
    round_id VARCHAR NOT NULL,
    num_unit INT NOT NULL,
    PRIMARY KEY (id, round_id)
) WITHOUT ROWID;

INSERT INTO
    poet_registration_new (id, round_id, num_unit)
SELECT
    id,
    round_id,
    num_unit
FROM
    poet_registration;

DROP TABLE poet_registration;

ALTER TABLE
    poet_registration_new RENAME TO poet_registration;