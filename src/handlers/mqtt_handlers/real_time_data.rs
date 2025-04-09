use std::sync::Arc;

use bson::{doc, oid::ObjectId};
use mongodb::{options::FindOneOptions, Collection, Database};
use tokio::sync::RwLock;

use crate::{
    models::Device,
    utils::{device_data_model::DeviceMessage, utils_functions::find_workspace_with_device_id},
    websocket_srv::ClientsConnections,
};

pub async fn handle_real_time_data(
    db: Database,
    message: &str,
    ws_conns: Arc<RwLock<ClientsConnections>>,
) -> Result<(), String> {
    let sensor_data_result = serde_json::from_str::<DeviceMessage>(&message);

    match sensor_data_result {
        Ok(sensor_data) => {
            let find_opts = FindOneOptions::builder()
                .projection(doc! { "id": 1 })
                .build();
            let device_coll: Collection<Device> = db.collection("Device");

            match device_coll
                .find_one(doc! { "serial": &sensor_data.device_id}, find_opts)
                .await
            {
                Ok(Some(device)) => {
                    let workspace = find_workspace_with_device_id(device.id.clone(), db.clone())
                        .await
                        .unwrap();

                    let locations = workspace[0].get_array("locations").unwrap();
                    let workspace_id = workspace[0].get_object_id("_id").unwrap();
                    let mut location_ids: Vec<ObjectId> = Vec::new();

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
                            return Err(format!(
                                "No location found for the device: {}",
                                &sensor_data.device_id
                            ));
                        }
                        false => {
                            for location in locations {
                                let doc = location.as_document().unwrap();
                                let id = doc.get_object_id("_id").unwrap();
                                location_ids.push(id);
                            }
                        }
                    } 

                    let msg = serde_json::to_string(&sensor_data).unwrap();

                    let conns = ws_conns.read().await;
                    conns.send_message(workspace_id.to_string(), &msg).await;

                    Ok(())
                }
                Ok(None) => Err("No device found".to_string()),
                Err(err) => {
                    let err = format!("Mongo Error: {:?}", err);
                    Err(err)
                }
            }
        }
        Err(_) => Err("No device found".to_string()),
    }
}
