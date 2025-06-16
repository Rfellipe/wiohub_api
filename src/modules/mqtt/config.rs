use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Configs {
    pub mqtt: MqttConfig,

    #[serde(skip)]
    config_path: PathBuf,
}

impl Configs {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(&path)?;
        let mut configs: Configs = toml::from_str(&config_content)?;
        configs.config_path = path.as_ref().to_path_buf();
        Ok(configs)
    }

    #[allow(dead_code)]
    pub fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_content = toml::to_string_pretty(self)?;
        let mut file = fs::File::create(&self.config_path)?;
        file.write_all(config_content.as_bytes())?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn update_and_save<F>(&mut self, updater: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut Self),
    {
        updater(self);
        self.save_to_file()?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MqttConfig {
    pub broker: String,
    pub port: u16,
    pub backup_dir: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub keep_alive: u16,
    pub ca_cert: Option<String>,
    pub client_cert: Option<String>,
    pub client_key: Option<String>,
    pub server_status_timeout: Option<i64>,
}
