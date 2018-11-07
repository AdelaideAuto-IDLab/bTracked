use super::schema::{map_config, collision_data, config};

#[derive(Identifiable, Queryable, Serialize)]
#[table_name="map_config"]
pub struct MapConfig {
    pub id: i32,
    pub map_key: String,
    pub description: String,
    pub config: String,
}

#[derive(Queryable, Serialize)]
pub struct MapConfigMetadata {
    pub map_key: String,
    pub description: String,
}

#[derive(Insertable, AsChangeset, Deserialize)]
#[table_name="map_config"]
pub struct NewMapConfig {
    pub map_key: String,
    pub description: String,
    pub config: String,
}

#[derive(Identifiable, Queryable, Associations, Serialize)]
#[belongs_to(MapConfig, foreign_key = "map_id")]
#[table_name="collision_data"]
pub struct CollisionData {
    pub id: i32,
    pub map_id: i32,
    pub data: Vec<u8>,
}

#[derive(Insertable, AsChangeset, Deserialize)]
#[table_name="collision_data"]
pub struct NewCollisionData {
    pub map_id: i32,
    pub data: Vec<u8>,
}

#[derive(Identifiable, Queryable, Serialize)]
#[table_name="config"]
pub struct Config {
    pub id: i32,
    pub key: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub description: String,
    pub value: String,
}

#[derive(Queryable, Serialize)]
pub struct ConfigMetadata {
    pub key: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub description: String,
}

#[derive(Insertable, AsChangeset, Deserialize)]
#[table_name="config"]
pub struct NewConfig {
    pub key: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub description: String,
    pub value: String,
}
