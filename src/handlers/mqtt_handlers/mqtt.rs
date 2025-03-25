use crate::handlers::mqtt_handlers::entry_data::handle_entry_data;
use mongodb::Database;
use rumqttc::{AsyncClient, Event, EventLoop, Packet, QoS};
use std::sync::{Arc, Mutex};

// eventloop: Arc<EventLoop>,
// let eventloop = Arc::try_unwrap(eventloop);

pub async fn run_mqtt(client: Arc<AsyncClient>, mut eventloop: EventLoop, db: Database) {
    client
        .subscribe("entry/data", QoS::AtMostOnce)
        .await
        .unwrap();

    client
        .subscribe("entry/registration", QoS::AtMostOnce)
        .await
        .unwrap();

    client
        .subscribe("entry/heartbeat", QoS::AtMostOnce)
        .await
        .unwrap();

    client
        .subscribe("entry/heartbeat/threads", QoS::AtMostOnce)
        .await
        .unwrap();

    while let Ok(notification) = eventloop.poll().await {
        match notification {
            event => {
                if let Event::Incoming(Packet::Publish(publish)) = event {
                    let payload = String::from_utf8_lossy(&publish.payload);
                    match publish.topic.as_str() {
                        "entry/data" => {
                            handle_entry_data(client.clone(), db.clone(), &payload.into_owned())
                                .await
                        }
                        "entry/registration" => println!("registration received"),
                        "entry/heartbeat" => println!("heartbeat received"),
                        "entry/heartbeat/threads" => println!("threads heartbeat received"),
                        _ => println!("unhandled topic"),
                    }
                }
            }
        }
    }
}

pub async fn publish_message(
    client: Arc<AsyncClient>,
    topic: &str,
    qos: QoS,
    retain: bool,
    payload: &str,
) {
    client.publish(topic, qos, retain, payload).await.unwrap();
}

pub async fn publish_device_report(
    client: Arc<AsyncClient>,
    device_id: &str,
    qos: QoS,
    payload: &str,
) {
    client
        .publish(
            format!("entry/reports/{}", device_id).to_string(),
            qos,
            false,
            payload,
        )
        .await
        .unwrap();
}

pub async fn publish_general_report(client: Arc<AsyncClient>, qos: QoS, payload: &str) {
    client
        .publish("entry/reports", qos, false, payload)
        .await
        .unwrap();
}
