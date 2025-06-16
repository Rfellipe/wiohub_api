mod config;
mod handlers;
mod mqtt_client;

use super::mqtt::{config::Configs, mqtt_client::MqttClient};
use std::{path::Path, sync::Arc};
use tokio::{sync::RwLock, task::JoinHandle};

pub async fn start_mqtt() -> JoinHandle<()> {
    let task = tokio::spawn(async {
        let config = std::env::var("MQTT_CONFIG").unwrap();
        let config_path = Path::new(&config);
        let mqtt_configs = match Configs::load_from_file(config_path) {
            Ok(c) => {
                log::info!("Configurations loaded");
                Arc::new(RwLock::new(c))
            }
            Err(e) => {
                log::info!("Failed to load configurations: {}", e);
                std::process::exit(1);
            }
        };

        let mqtt_settings = Arc::clone(&mqtt_configs).read().await.mqtt.clone();
        let server_status: Arc<RwLock<Option<i64>>> = Arc::new(RwLock::new(None));
        let mqtt = MqttClient::new(mqtt_settings, server_status.clone()).await;
        log::info!("Mqtt started...");

        handlers::device_registration::handler(&mqtt).await;
    });

    task
}
