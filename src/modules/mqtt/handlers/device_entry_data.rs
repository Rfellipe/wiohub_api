use crate::{
    modules::mqtt::{models::DeviceMessage, mqtt_client::MqttClient},
    shared::db::{
        get_db_access_manager,
        jsonb_wrapper::Json,
        models::{MinMaxValues, NewSensorData},
        PgPool,
    },
};
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::Local;
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
            let now = Local::now().naive_local();

            let message: DeviceMessage = serde_json::from_str(&payload).unwrap();
            let mut new_datas: Vec<NewSensorData> = vec![];

            for sensor in message.sensors {
                let new_data = NewSensorData {
                    device_id: message.device_id,
                    timestamp: now,
                    type_: sensor._type,
                    average: BigDecimal::from_f32(sensor.average.unwrap_or(0.0)).unwrap(),
                    min: Json(sensor.min.unwrap_or(MinMaxValues {
                        timestamp: 0,
                        value: 0.0,
                    })),
                    max: Json(sensor.max.unwrap_or(MinMaxValues {
                        timestamp: 0,
                        value: 0.0,
                    })),
                    values: Json(sensor.values.unwrap_or(vec![MinMaxValues {
                        timestamp: 0,
                        value: 0.0,
                    }])),
                    unit: sensor.unit.unwrap_or(String::from("No unit provided"))
                };

                new_datas.push(new_data);
            }

            let data = db.save_device_data(new_datas);
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
