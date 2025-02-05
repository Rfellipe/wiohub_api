use super::models::{Data, Device};
use super::utils::{group_by_ten_minutes, process_data_with_fill, DeviceControllerQueries};
use super::{BsonRejection, MongoRejection};
use bson::oid::ObjectId;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use mongodb::{Collection, Database};
use serde::Serialize;
use std::collections::HashMap;
use std::convert::Infallible;

#[derive(Debug, Serialize)]
struct Foo {
    field1: String,
    field2: i32,
    field3: bool,
}

#[derive(Debug, Serialize)]
struct GroupedData {
    device_id: ObjectId,
    data: Data,
}

pub async fn device_controller(
    opts: DeviceControllerQueries,
    db: Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    let start_dt: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339(&opts.start).expect("Failed to parse start string");
    let end_dt: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339(&opts.end).expect("Failed to parse end string");

    let start_utc: DateTime<Utc> = start_dt.with_timezone(&Utc);
    let end_utc: DateTime<Utc> = end_dt.with_timezone(&Utc);

    // let target_offset = FixedOffset::east(3 * 3600);

    // let start_target = start_utc.with_timezone(&target_offset);
    // let end_target = end_utc.with_timezone(&target_offset);

    // let start_res = start_target.to_rfc3339();
    // let end_res = end_target.to_rfc3339();

    let client_id = ObjectId::parse_str("6707040fe35f054bd65e5d73")
        .map_err(|e| warp::reject::custom(BsonRejection(e)))?;

    let device_coll: Collection<Device> = db.collection("Device");
    let data_coll: Collection<Data> = db.collection("Data");
    // Fetch active device IDs
    let devices_id = device_coll
        .find(
            doc! {
                "clientId": client_id,
                "status": "active",
            },
            None,
        )
        .await
        .map_err(|e| warp::reject::custom(MongoRejection(e)))?
        .try_collect::<Vec<_>>()
        .await
        .map_err(|e| warp::reject::custom(MongoRejection(e)))?
        .into_iter()
        .map(|doc| doc.id)
        .collect::<Vec<_>>();

    // Fetch data for the devices within the time range
    let all_data = data_coll
        .find(
            doc! {
                "deviceId": { "$in": devices_id },
                "timestamp": {
                    "$gte": start_utc,
                    "$lte": end_utc,
                },
            },
            FindOptions::builder()
                .sort(doc! { "timestamp": 1 })
                .projection(doc! { "deviceId": 1, "timestamp": 1, "sensorType": 1, "value": 1 })
                .build(),
        )
        .await
        .map_err(|e| warp::reject::custom(MongoRejection(e)))?
        .try_collect::<Vec<_>>()
        .await
        .map_err(|e| warp::reject::custom(MongoRejection(e)))?;

    let grouped_data = group_by_ten_minutes(all_data);
    let processed_data = process_data_with_fill(grouped_data, start_utc, end_utc);

    Ok(warp::reply::json(&processed_data))
}

pub async fn hello_handler(
    s: String,
    _db: mongodb::Database,
) -> Result<impl warp::Reply, Infallible> {
    println!("update_todo: id={}", s);

    Ok(warp::http::StatusCode::OK)
}
