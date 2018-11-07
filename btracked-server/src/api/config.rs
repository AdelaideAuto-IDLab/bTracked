use actix_web::*;
use serde_json;

use {AppState, db::{config::*, models::NewConfig}};

pub mod signal_model {
    use super::*;
    use futures::{Future, future};
    use tracking::SignalConfig;

    #[derive(Deserialize)]
    pub struct NewSignalModel {
        pub key: String,
        pub description: String,
        pub value: String,
    }

    pub fn list(state: State<AppState>) -> FutureResponse<HttpResponse> {
        map_response!(state.db.send(ListConfigs { type_: "signal_model".into() }))
    }

    pub fn insert_or_update((body, state): (Json<NewSignalModel>, State<AppState>))
        -> FutureResponse<HttpResponse>
    {
        let config: NewSignalModel = body.into_inner();
        if let Err(e) = serde_json::from_str::<SignalConfig>(&config.value) {
            let error = json!({ "error": e.to_string() });
            return future::result(Ok(HttpResponse::BadRequest().json(error))).responder();
        }

        let msg = NewConfig {
            key: config.key,
            type_: "signal_model".into(),
            description: config.description,
            value: config.value
        };
        map_response!(state.db.send(msg))
    }

    pub fn get_value((path, state): (Path<String>, State<AppState>))
        -> FutureResponse<HttpResponse>
    {
        let key = path.clone();
        map_response!(state.db.send(ConfigValue { key, type_: "signal_model".into() }))
    }
}


pub mod filter_config {
    use super::*;
    use futures::{Future, future};
    use tracking::FilterConfig;

    #[derive(Deserialize)]
    pub struct NewFilterModel {
        pub key: String,
        pub description: String,
        pub value: String,
    }

    pub fn list(state: State<AppState>) -> FutureResponse<HttpResponse> {
        map_response!(state.db.send(ListConfigs { type_: "filter_config".into() }))
    }

    pub fn insert_or_update((body, state): (Json<NewFilterModel>, State<AppState>))
        -> FutureResponse<HttpResponse>
    {
        let config: NewFilterModel = body.into_inner();

        if let Err(e) = serde_json::from_str::<FilterConfig>(&config.value) {
            let error = json!({ "error": e.to_string() });
            return future::result(Ok(HttpResponse::BadRequest().json(error))).responder();
        }

        let msg = NewConfig {
            key: config.key,
            type_: "filter_config".into(),
            description: config.description,
            value: config.value
        };
        map_response!(state.db.send(msg))
    }

    pub fn get_value((path, state): (Path<String>, State<AppState>))
        -> FutureResponse<HttpResponse>
    {
        let key = path.clone();
        map_response!(state.db.send(ConfigValue { key, type_: "filter_config".into() }))
    }
}
