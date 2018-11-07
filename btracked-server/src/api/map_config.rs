use actix_web::*;
use futures::Future;

use {AppState, db::{map_config::*, models::NewMapConfig}};

pub fn list(state: State<AppState>) -> FutureResponse<HttpResponse> {
    map_response!(state.db.send(ListMapConfigs {}))
}

pub fn insert_or_update((body, state): (Json<NewMapConfig>, State<AppState>))
    -> FutureResponse<HttpResponse>
{
    let config: NewMapConfig = body.into_inner();
    map_response!(state.db.send(config))
}

pub fn get_value((path, state): (Path<String>, State<AppState>))
    -> FutureResponse<HttpResponse>
{
    let map_key = path.clone();
    map_response!(state.db.send(MapConfigValue { map_key }))
}
