CREATE TABLE map_config (
    id INTEGER PRIMARY KEY NOT NULL,
    map_key VARCHAR(50) NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    config TEXT NOT NULL
);