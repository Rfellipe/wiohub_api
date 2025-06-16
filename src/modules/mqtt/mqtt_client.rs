use log::{error, info, warn};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS, TlsConfiguration, Transport};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{Mutex, RwLock},
    task,
    time::{interval, sleep},
};

use super::config::MqttConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BackupData<T> {
    topic: String,
    payload: T,
}

#[derive(Clone)]
pub struct MqttClient {
    client: AsyncClient,
    backup_dir: PathBuf,
    topic_handlers: Arc<Mutex<HashMap<String, Box<dyn FnMut(String) + Send + Sync + 'static>>>>,
    disconnected: Arc<Mutex<bool>>,
    server_status: Arc<RwLock<Option<i64>>>, // timestamp do último status do servidor
    server_status_timeout: i64,
}

impl MqttClient {
    pub async fn new(config: MqttConfig, server_status: Arc<RwLock<Option<i64>>>) -> Self {
        // client id is a tenant id
        // let tenant_id = hardware::machine_id().unwrap_or_else(|_| "unknown".to_string());

        // Create MQTT options
        let mut options = MqttOptions::new("syncos", config.broker, config.port);
        options.set_keep_alive(Duration::from_secs(config.keep_alive.into()));
        options.set_clean_session(false);

        // options.set_pending_throttle(duration::from_secs(1));
        // options.set_reconnect_delay(Duration::from_secs(5), Duration::from_secs(60));

        // Set username and password if provided
        if let (Some(username), Some(password)) = (config.username, config.password) {
            options.set_credentials(username, password);
        }

        // TLS configuration if SSL certificates are provided
        if let (Some(ca_cert), Some(client_cert), Some(client_key)) =
            (&config.ca_cert, &config.client_cert, &config.client_key)
        {
            let ca = load_certificate(ca_cert);
            let client_cert = load_certificate(client_cert);
            let client_key = load_certificate(client_key);

            let tls_config = TlsConfiguration::Simple {
                ca,
                alpn: None,
                client_auth: Some((client_cert, client_key)),
            };

            options.set_transport(Transport::tls_with_config(tls_config));
        }

        let (client, mut eventloop) = AsyncClient::new(options, 250);

        // Initialize topic handlers HashMap
        let topic_handlers: Arc<
            Mutex<HashMap<String, Box<dyn FnMut(String) + Send + Sync + 'static>>>,
        > = Arc::new(Mutex::new(HashMap::new()));

        // Clone topic_handlers for the task
        let task_topic_handlers = Arc::clone(&topic_handlers);
        let disconnected = Arc::new(Mutex::new(false));
        let task_disconnected = Arc::clone(&disconnected);

        // Create the backup directory if it doesn't exist
        let backup_dir = PathBuf::from(config.backup_dir);

        let server_status_timeout = match config.server_status_timeout {
            Some(timeout) => timeout * 1000, // to milliseconds
            None => 180 * 1000,              // 3 minutes
        };

        let mqtt_client = MqttClient {
            client,
            backup_dir,
            topic_handlers,
            disconnected,
            server_status,
            server_status_timeout,
        };

        let mqtt_client_clone = mqtt_client.clone();

        // Task for handling incoming messages
        task::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Packet::Publish(publish))) => {
                        let topic = publish.topic.clone();
                        let payload = String::from_utf8_lossy(&publish.payload);
                        info!("Received message on topic '{}': {}", topic, payload);

                        if let Some(handler) = task_topic_handlers.lock().await.get_mut(&topic) {
                            handler(payload.to_string());
                        }
                    }
                    Ok(Event::Incoming(Packet::ConnAck(_))) => {
                        info!("Connection established with broker.");
                        *task_disconnected.lock().await = false;

                        // se escreve nos topicos para garantir o recebimento
                        let handlers = task_topic_handlers.lock().await;
                        for topic in handlers.keys() {
                            let client = mqtt_client_clone.client.clone();
                            let topic = topic.clone();
                            tokio::spawn(async move {
                                info!("Reinscrevendo no tópico: {}", topic);
                                if let Err(e) = client
                                    .subscribe(topic.clone(), rumqttc::QoS::AtLeastOnce)
                                    .await
                                {
                                    error!("Failed to subscribe to topic '{}': {}", topic, e);
                                }
                            });
                        }
                    }
                    Ok(Event::Outgoing(_)) => {
                        // Outgoing events can be monitored here
                    }
                    Ok(_) => (),
                    Err(e) => {
                        error!("Erro no evento MQTT Check: {}", e.to_string());
                        sleep(Duration::from_secs(1)).await;
                        if e.to_string().contains("Network is unreachable") {
                            let mut is_disconnected = task_disconnected.lock().await;
                            if !*is_disconnected {
                                *is_disconnected = true;
                                warn!("Connection lost with broker.");
                            }
                        }
                    }
                }
            }
        });

        // Start the task to resend backup data periodically
        mqtt_client.start_resend_task().await;

        mqtt_client
    }

    pub async fn publish<T>(
        &self,
        topic: &str,
        payload: &T,
        qos: QoS,
        backup: bool,
    ) -> Result<(), String>
    where
        T: Serialize + Clone,
    {
        // Verifica o estado de conexão de forma assíncrona
        let is_disconnected = self.disconnected.lock().await;

        // timestamp do último status do servidor
        let server_status = match self.server_status.read().await.clone() {
            Some(status) => status,
            None => 0,
        };
        // let server_status_timeout = self.server_status_timeout;
        let now = chrono::Utc::now().timestamp_millis();

        // info!("is_disconnected: {}", *is_disconnected);
        warn!(
            "Resend task: disconnected: {}, server_status: {:?}, now: {} : last: ",
            *is_disconnected, server_status, now
        );

        let payload_str = match serde_json::to_string(payload) {
            Ok(payload) => payload,
            Err(e) => {
                error!("Falha ao serializar payload para JSON: {}", e);
                return Err(format!("Falha ao serializar payload para JSON: {}", e));
            }
        };

        // verifica se esta desconectado e se o timestamp do servidor é maior que 3 minutos
        // if server_status == 0 || ((now - server_status) > server_status_timeout) || *is_disconnected {
        //     if backup {
        //         if let Err(backup_err) = self.backup_data(topic, &payload_str).await {
        //             error!("Falha ao fazer backup dos dados: {}", backup_err);
        //         }
        //     }
        //     return Err("Connection with broker is not established".to_string());
        // }

        match self
            .client
            .publish(topic, qos, false, payload_str.clone())
            .await
        {
            Ok(_) => {
                info!("Data sent successfully to topic '{}'", topic);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send data to topic '{}': {}", topic, e);

                if backup {
                    if let Err(backup_err) = self.backup_data(topic, &payload_str).await {
                        error!("Failed to backup data: {}", backup_err);
                    }
                }

                Err(format!("Failed to send data to topic '{}': {}", topic, e))
            }
        }
    }

    pub async fn subscribe(&self, topic: &str, qos: QoS) -> Result<(), rumqttc::ClientError> {
        self.client.subscribe(topic, qos).await?;
        Ok(())
    }

    pub async fn add_topic_handler<F>(&self, topic: &str, handler: F)
    where
        F: FnMut(String) + Send + Sync + 'static,
    {
        let mut handlers = self.topic_handlers.lock().await;
        handlers.insert(topic.to_string(), Box::new(handler));
    }

    /// Salva os dados em backup se não puderem ser enviados
    pub async fn backup_data(&self, topic: &str, payload: &str) -> io::Result<()> {
        // Verifica e cria o diretório, se necessário
        if !self.backup_dir.exists() {
            if let Err(e) = fs::create_dir_all(&self.backup_dir).await {
                error!(
                    "Failed to create backup directory '{}': {}",
                    self.backup_dir.display(),
                    e
                );
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to create backup directory: {}", e),
                ));
            }
        }

        // Cria o conteúdo do backup manualmente
        let backup_data = BackupData {
            topic: topic.to_string(),
            payload,
        };

        // Define o nome do arquivo de backup
        let backup_file = self.backup_dir.join(format!(
            "backup_{}.json",
            chrono::Utc::now().timestamp_millis()
        ));

        let backup_data = match serde_json::to_string(&backup_data) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to serialize backup data: {}", e);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to serialize backup data: {}", e),
                ));
            }
        };

        // Abre o arquivo para escrita e grava o conteúdo
        match fs::File::create(&backup_file).await {
            Ok(mut file) => {
                if let Err(e) = file.write_all(backup_data.as_bytes()).await {
                    error!(
                        "Failed to write data to backup file '{}': {}",
                        backup_file.display(),
                        e
                    );
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to write data: {}", e),
                    ));
                }
                info!(
                    "Backup created for data on topic '{}', file: '{}'",
                    topic,
                    backup_file.display()
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    "Failed to open backup file '{}': {}",
                    backup_file.display(),
                    e
                );
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to open backup file: {}", e),
                ))
            }
        }
    }

    /// Task para reenviar dados de backup
    async fn start_resend_task(&self) {
        let backup_dir = self.backup_dir.clone();
        let client = self.client.clone();
        let disconnected = self.disconnected.clone();
        let server_status = self.server_status.clone();
        let server_status_timeout = self.server_status_timeout;

        task::spawn(async move {
            let mut resend_interval = interval(Duration::from_secs(60));

            loop {
                resend_interval.tick().await;

                // Primeiro, pega o status do servidor para evitar bloqueio prolongado
                let server_status_value = {
                    let status_guard = server_status.read().await;
                    status_guard.unwrap_or(0)
                };

                let disconnected_value = *disconnected.lock().await;
                let now = chrono::Utc::now().timestamp_millis();

                warn!(
                    "Resend task: disconnected: {}, server_status: {}, now: {}, last: {}",
                    disconnected_value,
                    server_status_value,
                    now,
                    (now - server_status_value)
                );

                if server_status_value == 0
                    || (now - server_status_value > server_status_timeout)
                    || disconnected_value
                {
                    warn!(
                        "Connection with broker is not established. Skipping backup data resend."
                    );
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }

                // Rodar `resend_backup_data` em uma nova task para evitar bloqueios
                let client_clone = client.clone();
                let backup_dir_clone = backup_dir.clone();
                tokio::spawn(async move {
                    if let Err(e) = resend_backup_data(&client_clone, &backup_dir_clone).await {
                        error!("Failed to resend backup data: {}", e);
                    }
                });
            }
        });
    }
}

/// Reenvia dados de backup até que a transmissão seja confirmada pelo servidor
async fn resend_backup_data(client: &AsyncClient, backup_dir: &PathBuf) -> io::Result<()> {
    let mut entries = fs::read_dir(backup_dir).await?;

    warn!("Resending backup data...");

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        // Processar apenas arquivos com extensão .json
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let mut file = match fs::File::open(&path).await {
                Ok(f) => f,
                Err(e) => {
                    error!("Failed to open backup file '{}': {}", path.display(), e);
                    continue;
                }
            };

            let mut contents = String::new();
            if let Err(e) = file.read_to_string(&mut contents).await {
                error!("Failed to read backup file '{}': {}", path.display(), e);
                continue;
            }

            let (topic, payload) = match serde_json::from_str::<BackupData<String>>(&contents) {
                Ok(data) => (data.topic, data.payload),
                Err(e) => {
                    error!(
                        "Failed to deserialize backup data from file '{}': {}",
                        path.display(),
                        e
                    );
                    continue;
                }
            };

            tokio::time::sleep(Duration::from_secs(10)).await;

            // Tentar reenviar os dados
            match client
                .publish(&topic, QoS::ExactlyOnce, false, payload.clone())
                .await
            {
                Ok(_) => {
                    info!("Successfully resent data for topic '{}'", topic);
                    if let Err(e) = fs::remove_file(&path).await {
                        error!("Failed to remove backup file '{}': {}", path.display(), e);
                    }
                }
                Err(e) => {
                    error!("Failed to resend data for topic '{}': {}", topic, e);
                }
            }
        }
    }

    Ok(())
}

fn load_certificate(path: &str) -> Vec<u8> {
    let mut file = File::open(path).expect("Failed to open certificate file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .expect("Failed to read certificate file");
    buffer
}
