use actix::prelude::*;
use actix_web::*;

use diesel::{self, prelude::*, result::{DatabaseErrorKind, Error::DatabaseError}};

use serde_json;

use db::{DbExecutor, models::{MapConfig, NewMapConfig, MapConfigMetadata}};

impl Message for NewMapConfig {
    type Result = Result<MapConfig, Error>;
}

impl Handler<NewMapConfig> for DbExecutor {
    type Result = Result<MapConfig, Error>;

    fn handle(&mut self, msg: NewMapConfig, _: &mut Self::Context) -> Self::Result {
        use db::schema::map_config::dsl::*;

        let conn = &self.0.get().expect("Failed to get DB connection");
        let result = conn.transaction(|| {
            let target = map_config.filter(map_key.eq(&msg.map_key));

            let result = diesel::insert_into(map_config)
                .values(&msg)
                .execute(conn);

            match result {
                Ok(_) => {},
                Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                    diesel::update(target)
                        .set(&msg)
                        .execute(conn)?;
                },
                Err(e) => return Err(e),
            }

            target.first(conn)
        });

        result.map_err(error::ErrorInternalServerError)
    }
}

pub struct ListMapConfigs {}

impl Message for ListMapConfigs {
    type Result = Result<Vec<MapConfigMetadata>, Error>;
}

impl Handler<ListMapConfigs> for DbExecutor {
    type Result = Result<Vec<MapConfigMetadata>, Error>;

    fn handle(&mut self, _: ListMapConfigs, _: &mut Self::Context) -> Self::Result {
        use db::schema::map_config::dsl::*;

        let conn = &self.0.get().expect("Failed to get DB connection");
        map_config
            .select((map_key, description))
            .load(conn)
            .map_err(error::ErrorInternalServerError)
    }
}


pub struct MapConfigValue {
    pub map_key: String,
}

impl Message for MapConfigValue {
    type Result = Result<serde_json::Value, Error>;
}

impl Handler<MapConfigValue> for DbExecutor {
    type Result = Result<serde_json::Value, Error>;

    fn handle(&mut self, msg: MapConfigValue, _: &mut Self::Context) -> Self::Result {
        use db::schema::map_config::dsl::*;

        let conn = &self.0.get().expect("Failed to get DB connection");
        let value = map_config
            .filter(map_key.eq(msg.map_key))
            .select(config)
            .first::<String>(conn)
            .map_err(error::ErrorInternalServerError)?;

        serde_json::from_str(&value).map_err(error::ErrorInternalServerError)
    }
}
