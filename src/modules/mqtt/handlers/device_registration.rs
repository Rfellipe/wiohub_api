use bson::oid::ObjectId;
use bson::Document;
use rumqttc::QoS;
use serde::Deserialize;

use crate::modules::mqtt::mqtt_client::MqttClient;

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

pub async fn handler(client: &MqttClient) {
    let topic = "entry/registration";
    // let parsed_msg: DeviceInfo = serde_json::from_str(msg).unwrap();

    let subscription = client.subscribe(topic, QoS::AtLeastOnce).await;
    match subscription {
        Ok(_) => {},
        Err(err) => {
            log::error!("Error subscribing to {} err: {:#?}", topic, err);
            panic!();
        }
    }
    client
        .add_topic_handler(topic, move |payload| println!("got: {}", payload))  
        .await;
}

