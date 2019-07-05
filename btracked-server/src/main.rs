extern crate actix;
extern crate actix_web;
#[macro_use]
extern crate crossbeam_channel;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate env_logger;
extern crate futures;
extern crate image;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate nalgebra as na;
extern crate nalgebra_glm as glm;
extern crate palette;
extern crate parking_lot;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tracking;

#[macro_use]
pub(crate) mod util;

mod api;
mod db;
mod map;
mod tracking_manager;
mod types;
mod update_listener;

use std::env;

use actix::prelude::*;
use actix_web::{http, middleware, server, App, fs};

fn configure_logger() {
    let mut builder = env_logger::Builder::new();
    builder.target(env_logger::Target::Stdout);

    match env::var("LOG") {
        Ok(var) => builder.parse_filters(&var),
        Err(_) => builder.parse_filters("warn"),
    };

    builder.init();
}

pub struct AppState {
    pub db: Addr<db::DbExecutor>,
}

fn main() {
    dotenv::dotenv().ok();
    configure_logger();

    info!("Started bTracked Server");

    let sys = actix::System::new("bTracked Server");

    let db_pool = db::init_pool();
    let db_addr = SyncArbiter::start(8, move || db::DbExecutor(db_pool.clone()));

    server::new(move || {
        App::with_state(AppState { db: db_addr.clone() })
            .middleware(middleware::Logger::default())
            .resource("/api/map", |r| {
                r.method(http::Method::GET).with(api::map_config::list);
                r.method(http::Method::POST).with(api::map_config::insert_or_update);
            })
            .resource("/api/map/{map_key}/config", |r| {
                r.method(http::Method::GET).with(api::map_config::get_value);
            })
            .resource("/api/map/{map_key}/collision", |r| {
                r.method(http::Method::GET).with(api::map_info::get_collision_image);
            })
            .resource("/api/map/{map_key}/coverage", |r| {
                r.method(http::Method::GET).with(api::map_info::get_coverage_image);
            })
            .resource("/api/signal_model", |r| {
                r.method(http::Method::GET).with(api::config::signal_model::list);
                r.method(http::Method::POST).with(api::config::signal_model::insert_or_update);
            })
            .resource("/api/signal_model/{model_key}/value", |r| {
                r.method(http::Method::GET).with(api::config::signal_model::get_value);
            })
            .resource("/api/filter_config", |r| {
                r.method(http::Method::GET).with(api::config::filter_config::list);
                r.method(http::Method::POST).with(api::config::filter_config::insert_or_update);
            })
            .resource("/api/filter_config/{model_key}/value", |r| {
                r.method(http::Method::GET).with(api::config::filter_config::get_value);
            })
            .resource("/api/instance", |r| {
                r.method(http::Method::GET).with(api::instance::list);
            })
            .resource("/api/instance/{instance_name}", |r| {
                r.method(http::Method::DELETE).with(api::instance::stop);
                r.method(http::Method::GET).with(api::instance::get_info);
            })
            .resource("/api/instance/{instance_name}/start", |r| {
                r.method(http::Method::POST).with(api::instance::start);
            })
            .resource("/api/instance/{instance_name}/measurement", |r| {
                r.method(http::Method::POST).with(api::instance::new_measurement);
            })
            .resource("/api/instance/{instance_name}/beacon_measurement", |r| {
                r.method(http::Method::POST).with(api::instance::new_beacon_measurement);
            })
            .resource("/api/instance/{instance_name}/snapshot/{num_particles}", |r| {
                r.method(http::Method::GET).with(api::instance::get_snapshot);
            })
            .resource("/api/instance/{instance_name}/estimate", |r| {
                r.method(http::Method::GET).with(api::instance::get_estimate);
            })
            .resource("/api/instance/{instance_name}/sim/{sim_name}", |r| {
                r.method(http::Method::GET).with(api::sim::state);
                r.method(http::Method::POST).with(api::sim::start);
                r.method(http::Method::DELETE).with(api::sim::stop);
            })
            .resource("/api/instance/{instance_name}/sim/{sim_name}/goto", |r| {
                r.method(http::Method::POST).with(api::sim::goto);
            })
            .resource("/ws/listener", |r| r.route().f(api::listener::listener))
            .handler("/", fs::StaticFiles::new("./resources").unwrap().index_file("index.html"))
            .default_resource(|r| r.h(http::NormalizePath::default()))
    })
        .bind(env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into()))
        .unwrap()
        .start();

    let _ = sys.run();
}
