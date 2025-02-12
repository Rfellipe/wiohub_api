use crate::errors::{AuthError, MongoRejection};
use crate::handlers::auth_handlers::security::{decode_jwt, JWT_SECRET};
use crate::models::{Client, Device};
use crate::utils::utils_models::{ApiDeviceDataResponse, CustomMessage, DeviceStatusQueries};
use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use mongodb::{Collection, Database};

#[utoipa::path(
        get,
        path = "devices/status",
        params(DeviceStatusQueries),
        responses(
            (status = 200, description = "Devices datas received", body = [ApiDeviceDataResponse]),
            (status = 400, description = "Dates parse error", body = [CustomMessage]),
            (status = 403, description = "Authorization token invalid or expired", body = String),
            (status = 500, description = "Internal Server Error", body = String),
        )
    )
]
pub async fn device_status_handler(
    authorization: String,
    opts: DeviceStatusQueries,
    db: Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    let user_info =
        decode_jwt(authorization, &JWT_SECRET).map_err(|_e| warp::reject::custom(AuthError))?;

    let device_coll: Collection<Device> = db.collection("Device");

    if let Some(opts) = opts.serial {
        let device_status = device_coll
            .find_one(
                doc! {
                    "serial": opts
                },
                None,
            )
            .await
            .map_err(|e| warp::reject::custom(MongoRejection(e)))?
            .ok_or_else(|| warp::reject::reject())?;

        Ok(warp::reply::json(&device_status))
    } else {
        let find_options = FindOneOptions::builder()
            .projection(doc! {
                "id": 1
            })
            .build();
        let client_coll: Collection<Client> = db.collection("Client");
        let client = client_coll
            .find_one(
                doc! {
                    "tenantId": user_info.tenant
                },
                find_options,
            )
            .await
            .map_err(|e| warp::reject::custom(MongoRejection(e)))?
            .ok_or_else(|| warp::reject::reject())?;

        let devices = device_coll
            .find(
                doc! {
                    "clientId": client.id,
                },
                None,
            )
            .await
            .map_err(|e| warp::reject::custom(MongoRejection(e)))?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| warp::reject::custom(MongoRejection(e)))?
            .into_iter()
            .map(|doc| doc)
            .collect::<Vec<_>>();

        Ok(warp::reply::json(&devices))
    }
}
