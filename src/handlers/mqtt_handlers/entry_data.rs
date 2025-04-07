use std::sync::Arc;

use crate::models::{Data, Device, Filter, Notification};
use crate::utils::device_data_model::DeviceMessage;
use crate::utils::utils_functions::{find_device_filter, find_workspace_with_device_id};
use crate::websocket_srv::{ClientsConnections, WsResult};

use bson::oid::ObjectId;
use mongodb::bson::doc;
use mongodb::bson::DateTime;
use mongodb::options::FindOneOptions;
use mongodb::{Collection, Database};
use tokio::sync::RwLock;
use log::info;

pub async fn handle_entry_data(db: Database, message: &str, ws_conns: Arc<RwLock<ClientsConnections>>) -> Result<(), String> {
    let sensor_data_result = serde_json::from_str::<DeviceMessage>(&message);
    let data_coll: Collection<Data> = db.collection("Data");
    let notification_coll: Collection<Notification> = db.collection("Collection");

    match sensor_data_result {
        Ok(sensor_data) => {
            let find_opts = FindOneOptions::builder()
                .projection(doc! {
                    "id": 1
                })
                .build();
            let device_coll: Collection<Device> = db.collection("Device");

            match device_coll
                .find_one(doc! {"serial": &sensor_data.device_id}, find_opts)
                .await
            {
                Ok(Some(device)) => {
                    let workspace = find_workspace_with_device_id(device.id.clone(), db.clone())
                        .await
                        .unwrap();

                    let locations = workspace[0].get_array("locations").unwrap();
                    let workspace_id = workspace[0].get_object_id("_id").unwrap();
                    let mut data_entries: Vec<Data> = Vec::new();
                    let mut location_ids: Vec<ObjectId> = Vec::new();
                    let mut ntfy_entries: Vec<Notification> = Vec::new();

                    match locations.is_empty() {
                        true => {
                            // publish_device_report(
                            //     client.clone(),
                            //     device.id.to_string().as_str(),
                            //     QoS::ExactlyOnce,
                            //     format!(
                            //         "No location found for the device: {}",
                            //         &sensor_data.device_id
                            //     )
                            //     .as_str(),
                            // )
                            // .await;
                            return Err(format!("No location found for the device: {}", &sensor_data.device_id));
                        }
                        false => {
                            for location in locations {
                                let doc = location.as_document().unwrap();
                                let id = doc.get_object_id("_id").unwrap();
                                location_ids.push(id);
                            }
                        }
                    }

                    for sensor in sensor_data.sensors {
                        let sensor_type = sensor._type.clone();

                        if let Ok(Some(filter)) =
                            find_device_filter(sensor_type.clone(), device.id, db.clone()).await
                        {
                            info!("current sensor: {}", sensor_type);
                            if let Some(min) = sensor.min {
                                if !check_limits(min.value, filter.clone()) {
                                    generate_log_and_notification(
                                        min.value,
                                        min.timestamp,
                                        sensor_type.clone(),
                                        "minimo".to_string(),
                                        workspace_id.clone(),
                                        device.id,
                                        &mut ntfy_entries,
                                    );
                                }

                                // create data
                                generate_data(
                                    min.value,
                                    sensor_type.clone(),
                                    min.timestamp,
                                    device.id,
                                    sensor.unit.clone().unwrap_or("N/A".to_string()),
                                    location_ids.clone(),
                                    &mut data_entries,
                                );
                            }
                            if let Some(max) = sensor.max {
                                if !check_limits(max.value, filter.clone()) {
                                    generate_log_and_notification(
                                        max.value,
                                        max.timestamp,
                                        sensor_type.clone(),
                                        "maximo".to_string(),
                                        workspace_id.clone(),
                                        device.id,
                                        &mut ntfy_entries,
                                    );
                                }

                                // create data
                                generate_data(
                                    max.value,
                                    sensor_type.clone(),
                                    max.timestamp,
                                    device.id,
                                    sensor.unit.clone().unwrap_or("N/A".to_string()),
                                    location_ids.clone(),
                                    &mut data_entries,
                                );
                            }
                            if let Some(avg) = sensor.average {
                                // let timestamp = DateTime::from_millis(sensor_data.timestamp);
                                if !check_limits(avg, filter.clone()) {
                                    //generate_log_and_notification(max.value, max.timestamp, sensor_type, "maximo".to_string(), wo, device_id, notification_entries);
                                    generate_log_and_notification(
                                        avg,
                                        sensor_data.timestamp,
                                        sensor_type.clone(),
                                        "media".to_string(),
                                        workspace_id.clone(),
                                        device.id,
                                        &mut ntfy_entries,
                                    );
                                }

                                // create data
                                generate_data(
                                    avg,
                                    sensor_type.clone(),
                                    sensor_data.timestamp,
                                    device.id,
                                    sensor.unit.clone().unwrap_or("N/A".to_string()),
                                    location_ids.clone(),
                                    &mut data_entries,
                                );
                            }
                            if let Some(values) = sensor.values {
                                for entry in values {
                                    if !check_limits(entry.value, filter.clone()) {
                                        // generate_log_and_notification(value, timestamp, sensor_type, limit, workspace_id, device_id, notification_entries);
                                        generate_log_and_notification(
                                            entry.value,
                                            entry.timestamp,
                                            sensor_type.clone(),
                                            "único".to_string(),
                                            workspace_id.clone(),
                                            device.id,
                                            &mut ntfy_entries,
                                        );
                                    }

                                    // create data
                                    generate_data(
                                        entry.value,
                                        sensor_type.clone(),
                                        sensor_data.timestamp,
                                        device.id,
                                        sensor.unit.clone().unwrap_or("N/A".to_string()),
                                        location_ids.clone(),
                                        &mut data_entries,
                                    );
                                }
                            }
                        } else {
                            let err = format!(
                                "No filter '{}' found for this sensor from the device: {}",
                                sensor_type,
                                device.id.to_string()
                            );
                            // return Err(err);
                        }
                    }

                    let _ = data_coll
                        .clone_with_type()
                        .insert_many(data_entries, None)
                        .await
                        .unwrap();

                    if !ntfy_entries.is_empty() {
                        let _ = notification_coll
                            .clone_with_type()
                            .insert_many(ntfy_entries.clone(), None)
                            .await
                            .unwrap();

                        for obj in ntfy_entries {
                           let res = WsResult {
                               type_: "notification".to_string(),
                               data: serde_json::to_string(&obj).unwrap()
                           };

                           let msg = serde_json::to_string(&res).unwrap();

                           let conns = ws_conns.read().await;
                           conns.send_message(workspace_id.to_string(), &msg).await;

                           // println!("sending message: {}", msg);
                           // conns.send_message_to_client(workspace_id.to_string(), &msg).await;
                           // tokio::spawn({
                           //     let conn_map = Arc::clone(&ws_conns);
                           //     let msg = msg.clone();
                           //     let workspace_id = workspace_id.clone();

                           //     async move {
                           //         let conns = conn_map.lock().await;
                           //         conns.send_message_to_client(workspace_id.to_string(), &msg).await;
                           //         drop(conns);
                           //     }
                           // });
                        }
                    let locations = workspace[0].get_array("locations").unwrap();
                    }

                    Ok(())
                }
                Ok(None) => {
                    Err("Error finding device".to_string())
                }
                Err(err) => {
                    let err = format!("Mongo Error: {:?}", err);
                    Err(err)
                }
            }
        }
        Err(_) => {
            Err("Error finding device".to_string())
        }
    }
}

fn check_limits(value: f32, sensor_filter: Filter) -> bool {
    let filter_min_val = sensor_filter.min_value.unwrap();
    let filter_max_val = sensor_filter.max_value.unwrap();

    if value < filter_min_val {
        return false;
    }
    if value > filter_max_val {
        return false;
    }
    return true;
}

fn generate_log_and_notification(
    value: f32,
    timestamp: i64,
    sensor_type: String,
    limit: String,
    workspace_id: ObjectId,
    device_id: ObjectId,
    notification_entries: &mut Vec<Notification>,
    // log_entries: Vec<Log>
) {
    let msg = format!(
        "the sensor {} {} is out of the {} limit. device {}",
        sensor_type,
        value,
        limit,
        device_id.to_string()
    );
    let bson_datetime = DateTime::from_millis(timestamp);

    let ntf = Notification {
        id: ObjectId::new(),
        type_: "alert".to_string(),
        message: msg,
        read: false,
        severity: "high".to_string(),
        device_id: Some(device_id),
        workspace_id: Some(workspace_id),
        timestamp: bson_datetime,
        device: None,
        location: None,
        location_id: None,
        user: None,
        user_id: None,
        workspace: None,
        created_at: None,
    };

    notification_entries.push(ntf);
}

fn generate_data(
    value: f32,
    sensor_type: String,
    timestamp: i64,
    device_id: ObjectId,
    unit: String,
    location_ids: Vec<ObjectId>,
    data_entries: &mut Vec<Data>,
) {
    let bson_datetime = DateTime::from_millis(timestamp);
    let data = Data {
        id: ObjectId::new(),
        sensor_type: Some(sensor_type),
        timestamp: Some(bson_datetime),
        value: Some(value),
        unit: Some(unit),
        status: Some("ok".to_string()),
        location_id: Some(location_ids),
        device_id: Some(device_id),
        location: None,
        device: None,
        created_at: None,
    };

    data_entries.push(data);
}
