use actix::prelude::*;
use actix_web::*;

use diesel::{self, prelude::*, result::{DatabaseErrorKind, Error::DatabaseError}};

use serde_json;

use db::{DbExecutor, models::{Config, NewConfig, ConfigMetadata}};

impl Message for NewConfig {
    type Result = Result<Config, Error>;
}

impl Handler<NewConfig> for DbExecutor {
    type Result = Result<Config, Error>;

    fn handle(&mut self, msg: NewConfig, _: &mut Self::Context) -> Self::Result {
        use db::schema::config::dsl::*;

        let conn = &self.0.get().expect("Failed to get DB connection");
        let result = conn.transaction(|| {
            let target = config.filter(key.eq(&msg.key)).filter(type_.eq(&msg.type_));
            let result = diesel::insert_into(config).values(&msg).execute(conn);

            match result {
                Ok(_) => {},
                Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                    diesel::update(target).set(&msg).execute(conn)?;
                },
                Err(e) => return Err(e),
            }

            target.first(conn)
        });

        result.map_err(error::ErrorInternalServerError)
    }
}

pub struct ListConfigs {
    pub type_: String,
}

impl Message for ListConfigs {
    type Result = Result<Vec<ConfigMetadata>, Error>;
}

impl Handler<ListConfigs> for DbExecutor {
    type Result = Result<Vec<ConfigMetadata>, Error>;

    fn handle(&mut self, msg: ListConfigs, _: &mut Self::Context) -> Self::Result {
        use db::schema::config::dsl::*;

        let conn = &self.0.get().expect("Failed to get DB connection");
        config
            .filter(type_.eq(msg.type_))
            .select((key, type_, description))
            .load(conn)
            .map_err(error::ErrorInternalServerError)
    }
}

pub struct ConfigValue {
    pub key: String,
    pub type_: String,
}

impl Message for ConfigValue {
    type Result = Result<serde_json::Value, Error>;
}

impl Handler<ConfigValue> for DbExecutor {
    type Result = Result<serde_json::Value, Error>;

    fn handle(&mut self, msg: ConfigValue, _: &mut Self::Context) -> Self::Result {
        use db::schema::config::dsl::*;

        let conn = &self.0.get().expect("Failed to get DB connection");
        let result = config
            .filter(key.eq(msg.key))
            .filter(type_.eq(msg.type_))
            .select(value)
            .first::<String>(conn)
            .map_err(error::ErrorInternalServerError)?;

        serde_json::from_str(&result).map_err(error::ErrorInternalServerError)
    }
}
