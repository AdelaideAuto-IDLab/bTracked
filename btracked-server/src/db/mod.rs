pub mod models;
pub mod schema;

pub mod map_config;
pub mod map_info;
pub mod config;

use std::env;

use actix::prelude::*;

use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    sqlite::SqliteConnection
};

pub type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;
pub type _Connection = PooledConnection<ConnectionManager<SqliteConnection>>;

pub fn init_pool() -> SqlitePool {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "db.sqlite".into());
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::new(manager).expect("Failed to initialize database pool")
}

pub struct DbExecutor(pub Pool<ConnectionManager<SqliteConnection>>);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}
