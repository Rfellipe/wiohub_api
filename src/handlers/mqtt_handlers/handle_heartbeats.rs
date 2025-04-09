use chrono::Utc;
use serde::Deserialize;
use mongodb::{bson::doc, Collection, Database};

use crate::models::Device;

#[derive(Debug, Deserialize, Clone)]
struct DeviceReq {
    uuid: String
}

// #[derive(Debug, Deserialize, Clone)]
// struct Sensors {
//     thd: String,
//     sts: String,
//     err: Option<String>
// }

#[derive(Debug, Deserialize, Clone)]
struct DeviceThreadsReq {
    uuid: String,
    // sensors: Vec<Sensors>
}

pub async fn read_device_heartbeat(message: &str, db: Database) -> Result<(), String> {
    let device_serial: DeviceReq = serde_json::from_str(&message).unwrap();

    let device_coll: Collection<Device> = db.collection("Device");
    let query = doc! { "serial": device_serial.uuid };
    let update = doc! { "$set": doc! { "lastConnection": Utc::now() } };
    match device_coll.update_one( query, update, None).await {
        Ok(device) => println!("Device updated: {:#?}", device),
        Err(err) => return Err(format!("Error updating device: {}", err))
    }

    Ok(())
}

pub async fn read_device_threads_heartbeat(message: &str, db: Database) -> Result<(), String> {
    let device_threads_info: DeviceThreadsReq = serde_json::from_str(&message).unwrap();        

    let device_coll: Collection<Device> = db.collection("Device");
    let query = doc! { "serial": device_threads_info.uuid };
    let update = doc! { "$set": doc! { "sensorsStatus": message } };
    match device_coll.update_one( query, update, None).await {
        Ok(device) => println!("Device updated: {:#?}", device),
        Err(err) => return Err(format!("Error updating device: {}", err))
    }

    Ok(())
}
