use bson::oid::ObjectId;
use bson::Document;
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::{options::FindOneOptions, Collection, Database};
use serde::Deserialize;

use crate::models::{Client, Device, Filter};

#[derive(Debug, Deserialize, Clone)]
struct Version {
    pub version: String,
}

#[derive(Debug, Deserialize, Clone)]
struct DeviceInfo {
    #[serde(rename = "tenantId")]
    pub tenant_id: String,
    pub uuid: String,
    pub mac: String,
    pub version: String,
    pub software: Option<String>,
    pub firmware: Version,
}

pub async fn handle_device_registration(msg: &str, db: Database) -> Result<(), String> {
    let parsed_msg: DeviceInfo = serde_json::from_str(msg).unwrap();

    let device_coll: Collection<Device> = db.collection("Device");
    let find_opts = FindOneOptions::builder()
        .projection(doc! {
            "id": 1
        })
        .build();
    match device_coll
        .clone_with_type::<Device>()
        .find_one(doc! {"serial": &parsed_msg.uuid}, find_opts.clone())
        .await
    {
        Ok(Some(_)) => {
            // Device alredy exists
            return Err(format!("Device alredy exists"));
        }
        Ok(None) => {
            // Handle device registration
            let client_coll: Collection<Client> = db.collection("Client");
            match client_coll
                .find_one(doc! { "tenantId": parsed_msg.tenant_id}, find_opts)
                .await
            {
                Ok(Some(client)) => {
                    let point = r#"{"latitude":"0","longitude":"0"}"#;
                    let new_device_info = doc! {
                        "serial": &parsed_msg.uuid,
                        "macAddres": &parsed_msg.mac,
                        "clientId": client.id,
                        "point": point,
                        "hardwareVersion": &parsed_msg.version,
                        "osVersion": &parsed_msg.software,
                        "kernelVersion": &parsed_msg.firmware.version,
                        "transmissionInterval": 10,
                        "mode": "logger",
                        "lastConnection": Utc::now(),
                        "name": format!("data-logger-{}", parsed_msg.mac),
                        "type": "weatherStation",
                    };

                    let new_device = device_coll
                        .clone_with_type()
                        .insert_one(new_device_info, None)
                        .await;

                    match new_device {
                        Ok(device_id) => {
                            let filters_coll: Collection<Filter> = db.collection("Filter");
                            let default_filters =
                                base_filters(device_id.inserted_id.as_object_id().unwrap());
                            let _ = filters_coll
                                .clone_with_type()
                                .insert_many(default_filters, None)
                                .await;
                        }
                        Err(e) => return Err(format!("MongoDb error when creating device: {}", e)),
                    }

                    Ok(())
                }
                Ok(None) => {
                    // Handle no client
                    return Err(format!("No client found"));
                }
                Err(err) => {
                    // Handle mongo error
                    return Err(format!("MongoDb error when finding client: {}", err));
                }
            }
        }
        Err(err) => {
            // Handle mongo error
            return Err(format!("MongoDb error when finding device: {}", err));
        }
    }
}

fn base_filters(device_id: ObjectId) -> Vec<Document> {
    vec![
        doc! {
          "sensorType": "temperature",
          "unit": "ºC",
          "createdAt": Utc::now(),
          "updatedAt": Utc::now(),
          "deviceId": device_id,
          "minValue": 0,
          "maxValue": 10
        },
        doc! {
          "sensorType": "humidity",
          "unit": "%",
          "createdAt": Utc::now(),
          "updatedAt": Utc::now(),
          "deviceId": device_id,
          "minValue": 0,
          "maxValue": 10
        },
        doc! {
          "sensorType": "pressure",
          "unit": "Pa",
          "createdAt": Utc::now(),
          "updatedAt": Utc::now(),
          "deviceId": device_id,
          "minValue": 0,
          "maxValue": 10
        },
        doc! {
          "sensorType": "wind_speed",
          "unit": "m/s",
          "createdAt": Utc::now(),
          "updatedAt": Utc::now(),
          "deviceId": device_id,
          "minValue": 0,
          "maxValue": 10
        },
        doc! {
          "sensorType": "wind_direction",
          "unit": "º",
          "createdAt": Utc::now(),
          "updatedAt": Utc::now(),
          "deviceId": device_id,
          "minValue": 0,
          "maxValue": 10
        },
        doc! {
          "sensorType": "rainfall",
          "unit": "mm",
          "createdAt": Utc::now(),
          "updatedAt": Utc::now(),
          "deviceId": device_id,
          "minValue": 0,
          "maxValue": 10
        },
        doc! {
          "sensorType": "solar_radiantion",
          "unit": "ºC",
          "createdAt": Utc::now(),
          "updatedAt": Utc::now(),
          "deviceId": device_id,
          "minValue": 0,
          "maxValue": 10
        },
        doc! {
          "sensorType": "uv_index",
          "unit": "%",
          "createdAt": Utc::now(),
          "updatedAt": Utc::now(),
          "deviceId": device_id,
          "minValue": 0,
          "maxValue": 10
        },
        doc! {
          "sensorType": "co2",
          "unit": "ppm",
          "createdAt": Utc::now(),
          "updatedAt": Utc::now(),
          "deviceId": device_id,
          "minValue": 0,
          "maxValue": 10
        },
        doc! {
          "sensorType": "pm2_5",
          "unit": "µg/m³",
          "createdAt": Utc::now(),
          "updatedAt": Utc::now(),
          "deviceId": device_id,
          "minValue": 0,
          "maxValue": 10
        },
        doc! {
          "sensorType": "pm10",
          "unit": "µg/m³",
          "createdAt": Utc::now(),
          "updatedAt": Utc::now(),
          "deviceId": device_id,
          "minValue": 0,
          "maxValue": 10
        },
    ]
}
