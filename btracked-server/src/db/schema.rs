table! {
    collision_data (id) {
        id -> Integer,
        map_id -> Integer,
        data -> Binary,
    }
}

table! {
    config (id) {
        id -> Integer,
        key -> Text,
        #[sql_name = "type"]
        type_ -> Text,
        description -> Text,
        value -> Text,
    }
}

table! {
    map_config (id) {
        id -> Integer,
        map_key -> Text,
        description -> Text,
        config -> Text,
    }
}

joinable!(collision_data -> map_config (map_id));

allow_tables_to_appear_in_same_query!(
    collision_data,
    config,
    map_config,
);
