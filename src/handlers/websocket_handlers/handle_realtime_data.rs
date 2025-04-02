use rumqttc::QoS;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::mqtt_srv::MqttClient;
use log::error;

#[derive(Debug, Deserialize)]
struct RealTimeReq {
    // #[serde(rename = "type")]
    // type_: String,
    #[serde(rename = "deviceId")]
    device_id: String,
    start: bool
}

#[derive(Debug, Serialize)]
struct RealTimeRes {    
    #[serde(rename = "deviceId")]
    device_id: String,
    start: bool
}

pub async fn start_stop_realtime_data(msg: &str, mqtt_client: Arc<MqttClient>) {
    let req: RealTimeReq = serde_json::from_str(msg).unwrap();
    let payload = serde_json::to_string( &RealTimeRes {
        device_id: req.device_id,
        start: req.start
    }).unwrap();

    if let Err(e) =  mqtt_client.publish("sensors/realtime", &payload, QoS::ExactlyOnce, false).await {
       error!("{}", e);
    }
    
    // send the message to MQTT topic: sensors/realtime
    // { "deviceId": "someId", "start": true }
    // device will now start sending data to topic: sensors/realtime/data
}

// pub fn realtime_data() {}
