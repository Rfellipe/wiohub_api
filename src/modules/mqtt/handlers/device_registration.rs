use crate::{
    modules::mqtt::{models::DeviceInfo, mqtt_client::MqttClient},
    shared::db::{get_db_access_manager, models::NewDevice, PgPool},
};
use rumqttc::QoS;

pub async fn handler(client: &MqttClient, pool: PgPool) {
    let topic = "entry/registration";
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

            let parsed_payload: DeviceInfo = serde_json::from_str(&payload).unwrap();
            let new_device = NewDevice {
                serial_number: parsed_payload.uuid,
                status: String::from("Active"),
                name: format!("dev-{}", parsed_payload.mac),
                firmware: parsed_payload.firmware.version,
                organization_id: parsed_payload.tenant_id
            };

            let add_device = db.add_device(new_device);

            match add_device {
                Ok(dev) => {
                    log::info!("Device: {} registered successfully", dev.id);
                }
                Err(e) => {
                    log::error!(
                        "Error registering device with mac: {} (err: {})",
                        parsed_payload.mac,
                        e.to_string()
                    );
                }
            }
        })
        .await;
}
