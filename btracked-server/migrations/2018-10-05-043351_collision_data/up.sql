CREATE TABLE collision_data (
    id INTEGER PRIMARY KEY NOT NULL,
    map_id INTEGER NOT NULL UNIQUE,
    data BLOB NOT NULL,

    FOREIGN KEY(map_id) REFERENCES map_config(id) ON DELETE CASCADE
);

CREATE TRIGGER remove_cached_collision_data UPDATE OF config ON map_config
    BEGIN
        DELETE FROM collision_data WHERE map_id = old.id;
    END;