use crate::errors::{AppError, ErrorType};
use crate::handlers::auth_handlers::security::{decode_jwt, JWT_SECRET};
use crate::models::Device;
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonBody {
    pub name: String,
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
    #[serde(rename = "partNumber")]
    pub part_number: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub location: String,
    pub lat: String,
    pub long: String,
}

pub async fn device(
    authorization: String,
    body: JsonBody,
    db: Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    let user_info =
        decode_jwt(authorization, &JWT_SECRET, db.clone()).await?;

    let device_coll: Collection<Device> = db.collection("Device");

    let location_id = ObjectId::parse_str(body.location).unwrap();
    let client_id = ObjectId::parse_str(user_info.client_id.unwrap()).unwrap();

    let point = r#"{"latitude":"0","longitude":"0"}"#;
    let new_device = doc! {
        "name": body.name,
        "type": body.device_type,
        "point": point,
        "serial": body.serial_number,
        "locationId": location_id,
        "clientId": client_id
    };

    let _  = device_coll.clone_with_type()
        .insert_one(new_device.clone(), None)
        .await
        .map_err(|e| {
            let err_str = format!("Mongo Error: {:#?}", e);
            let err = AppError {
                message: err_str,
                err_type: ErrorType::MongoError 
            };
            warp::reject::custom(err)
        })?;

    Ok(warp::reply::reply())
}
