use crate::{
    modules::mqtt::{models::DeviceMessage, mqtt_client::MqttClient},
    shared::db::{get_db_access_manager, PgPool},
};
use rumqttc::QoS;

pub async fn handler(client: &MqttClient, pool: PgPool) {
    let topic = "entry/data";
    let pool_clone = pool.clone();

    let subscription = client.subscribe(topic, QoS::AtLeastOnce).await;
    match subscription {
        Ok(_) => {}
        Err(err) => {
            log::error!("Error subscribing to {} err: {:#?}", topic, err);
            panic!();
        }
    }
    client
        .add_topic_handler(topic, move |payload| {
            let pool = pool_clone.clone();
            let mut db = get_db_access_manager(pool).unwrap();

            let message: DeviceMessage = serde_json::from_str(&payload).unwrap();

            let data = db.save_device_data(message);
            match data {
                Ok(_) => {
                    log::info!("Data saved successfully");
                }
                Err(e) => {
                    log::error!("Error saving data (err: {})", e.to_string());
                }
            }
        })
        .await;
}
