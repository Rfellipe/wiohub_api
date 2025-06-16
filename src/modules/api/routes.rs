use crate::{
    modules::api::{
        filters::{with_auth, with_db_access_manager},
        handlers::devices_data_handler,
    },
    shared::db::PgPool,
};
use serde::{Deserialize, Serialize};
use utoipa::IntoParams;
use warp::Filter;

/// Params for /api/devices/data route
/// Ex: /api/devices/data?start=""end=""
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct DeviceControllerQueries {
    pub start: String,
    pub end: String,
}

pub fn devices_route(
    pool: PgPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("devices" / "data")
        .and(warp::get())
        .and(with_auth())
        .and(warp::query::<DeviceControllerQueries>())
        .and(with_db_access_manager(pool))
        .and_then(devices_data_handler)
}
