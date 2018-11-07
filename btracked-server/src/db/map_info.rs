use actix::prelude::*;
use actix_web::*;

use diesel::{self, prelude::*};
use serde_json;
use tracking::GeometryConfig;

use {
    db::{
        DbExecutor,
        models::{MapConfig, CollisionData, NewCollisionData},
        schema::{map_config, collision_data}
    },
    map
};

pub struct GetMapInfo {
    pub map_key: String,
}

impl Message for GetMapInfo {
    type Result = Result<(GeometryConfig, Vec<u8>), Error>;
}

impl Handler<GetMapInfo> for DbExecutor {
    type Result = Result<(GeometryConfig, Vec<u8>), Error>;

    fn handle(&mut self, msg: GetMapInfo, _: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().expect("Failed to get DB connection");
        let map: MapConfig = map_config::table.filter(map_config::map_key.eq(msg.map_key))
            .first(conn)
            .optional()
            .map_err(error::ErrorInternalServerError)?
            .ok_or_else(|| error::ErrorNotFound("Map not found"))?;

        let geometry_config = serde_json::from_str(&map.config)?;

        let record: Option<CollisionData> = CollisionData::belonging_to(&map)
            .first(conn)
            .optional()
            .map_err(error::ErrorInternalServerError)?;

        if let Some(record) = record {
            return Ok((geometry_config, record.data));
        }

        let data = map::generate_collision_map(&geometry_config)
            .map_err(error::ErrorInternalServerError)?;
        let record = NewCollisionData {
            map_id: map.id,
            data: data.clone()
        };
        diesel::insert_into(collision_data::table)
            .values(&record)
            .execute(conn)
            .map_err(error::ErrorInternalServerError)?;

        Ok((geometry_config, data))
    }
}
