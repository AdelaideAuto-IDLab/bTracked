CREATE TABLE config (
    id INTEGER PRIMARY KEY NOT NULL,
    `key` VARCHAR(50) NOT NULL,
    `type` VARCHAR(50) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    `value` TEXT NOT NULL,

    CONSTRAINT type_key UNIQUE (`key`, `type`)
);