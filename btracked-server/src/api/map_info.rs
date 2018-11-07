use actix_web::*;
use futures::Future;

use {AppState, db::map_info::*};

pub fn get_collision_image((path, state): (Path<String>, State<AppState>))
    -> FutureResponse<HttpResponse>
{
    let map_key = path.clone();
    state.db.send(GetMapInfo { map_key })
        .from_err()
        .and_then(|data| match data {
            Ok((_, collision_map)) => {
                Ok(HttpResponse::Ok().content_type("image/png").body(collision_map))
            },
            Err(err) => Ok({
                error!("{:?}", err);
                HttpResponse::InternalServerError().into()
            })
        }).responder()
}

pub fn get_coverage_image((_path, _state): (Path<String>, State<AppState>))
    -> FutureResponse<HttpResponse>
{
    unimplemented!()
}
