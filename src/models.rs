use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use bson::oid::ObjectId;
// use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: Option<String>,
    pub tenant_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub ftp: Option<Ftp>,
    pub ftp_id: Option<ObjectId>,
    pub locations: Option<Vec<Location>>,
    pub workspaces: Option<Vec<Workspace>>,
    pub users: Option<Vec<User>>,
    pub extensions: Option<Vec<Extensions>>,
    pub devices: Option<Vec<Device>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Ftp {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub client: Vec<Client>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub package: serde_json::Value,
    pub path: String,
    pub devices: Vec<ExtensionsOnDevice>,
    pub client: Client,
    pub client_id: ObjectId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionsOnDevice {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub extension: Extensions,
    pub extension_id: ObjectId,
    pub device: Device,
    pub device_id: ObjectId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: Option<String>,
    pub client: Option<Client>,
    pub client_id: Option<ObjectId>,
    pub polygon: Option<serde_json::Value>,
    pub devices: Option<Vec<Device>>,
    pub data: Option<Vec<Data>>,
    pub logs: Option<Vec<Log>>,
    pub notifications: Option<Vec<Notification>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub workspaces: Option<Vec<Workspace>>,
    pub workspace_id: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: Option<String>,
    pub users: Option<Vec<User>>,
    pub user_id: Option<Vec<ObjectId>>,
    pub active: Option<bool>,
    pub client_id: Option<ObjectId>,
    pub client: Option<Client>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub locations: Option<Vec<Location>>,
    pub location_id: Option<Vec<String>>,
    pub notifications: Option<Vec<Notification>>,
    pub logs: Option<Vec<Log>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub title: String,
    pub content: String,
    pub author: User,
    pub author_id: ObjectId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub point: Option<serde_json::Value>,
    pub serial: Option<String>,
    pub location: Option<Location>,
    pub location_id: Option<ObjectId>,
    pub client: Option<Client>,
    pub client_id: Option<ObjectId>,
    pub transmission_interval: Option<i32>,
    pub sensors_status: Option<String>,
    pub data: Option<Vec<Data>>,
    pub mode: Option<String>,
    pub status: Option<String>,
    pub last_connection: Option<bson::DateTime>,
    pub mac_address: Option<String>,
    pub hardware_version: Option<String>,
    pub os_version: Option<String>,
    pub kernel_version: Option<String>,
    pub cpu_architecture: Option<String>,
    pub total_memory: Option<i32>,
    pub storage_capacity: Option<i32>,
    pub ip_address: Option<String>,
    pub temp_sensor_data_path: Option<String>,
    pub temp_log_path: Option<String>,
    pub temp_file_storage_path: Option<String>,
    pub connections: Option<Vec<DeviceConnection>>,
    pub logs: Option<Vec<Log>>,
    pub notifications: Option<Vec<Notification>>,
    pub filters: Option<Vec<Filter>>,
    pub calculations: Option<Vec<Calculation>>,
    pub configurations: Option<Vec<DeviceConfiguration>>,
    pub software_updates: Option<Vec<SoftwareUpdate>>,
    pub file_uploads: Option<Vec<FileUpload>>,
    pub created_at: Option<bson::DateTime>,
    pub updated_at: Option<bson::DateTime>,
    pub extensions: Option<Vec<ExtensionsOnDevice>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConnection {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "type")]
    pub type_: String,
    pub status: String,
    pub signal_strength: Option<i32>,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub connection_details: Option<serde_json::Value>,
    pub device: Device,
    pub device_id: ObjectId,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileUpload {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub filename: String,
    pub size: i32,
    pub path: String,
    pub status: String,
    pub device: Device,
    pub device_id: ObjectId,
    pub user: Option<User>,
    pub user_id: Option<ObjectId>,
    pub uploaded_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub processed_by: Option<String>,
    pub validation_status: Option<String>,
    pub error_details: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub avatar: Option<String>,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub password: String,
    pub client: Option<Client>,
    pub client_id: Option<ObjectId>,
    pub is_pending: Option<bool>,
    // pub role: Option<Role>,
    pub tenant_id: Option<String>,
    pub api_ket: Option<String>,
    pub failed_login_attempts: Option<i32>,
    pub lock_until: Option<DateTime<Utc>>,
    pub is_locked: Option<bool>,
    pub refresh_token: Option<Vec<RefreshToken>>,
    pub logs: Option<Vec<Log>>,
    pub notes: Option<Vec<Note>>,
    pub notifications: Option<Vec<Notification>>,
    pub api_keys: Option<Vec<ApiKey>>,
    pub software_update: Option<Vec<SoftwareUpdate>>,
    pub file_uploads: Option<Vec<FileUpload>>,
    pub workspaces: Option<Vec<Workspace>>,
    pub workspaces_id: Option<Vec<String>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// #[serde(rename_all = "camelCase")]
// pub enum Role {
//     ADMIN,
//     OPERATOR,
//     VIEWER,
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub sensor_type: Option<String>,
    pub value: Option<f32>,
    pub unit: Option<String>,
    pub status: Option<String>,
    // #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub timestamp: Option<bson::DateTime>,
    pub location: Option<Vec<Location>>,
    pub location_id: Option<Vec<ObjectId>>,
    pub device: Option<Device>,
    pub device_id: Option<ObjectId>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "sensorType")]
    pub sensor_type: Option<String>,
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
    pub unit: Option<String>,
    // pub created_at: Option<DateTime<Utc>>,
    // pub updated_at: Option<DateTime<Utc>>,
    pub device_id: Option<ObjectId>,
    pub device: Option<Device>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Calculation {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub sensor_type: String,
    pub formula: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub device_id: Option<ObjectId>,
    pub device: Option<Device>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "type")]
    pub type_: String,
    pub message: String,
    pub level: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub device: Option<Device>,
    pub device_id: Option<ObjectId>,
    pub location: Option<Location>,
    pub location_id: Option<ObjectId>,
    pub user: Option<User>,
    pub user_id: Option<ObjectId>,
    pub workspace: Option<Workspace>,
    pub workspace_id: Option<Vec<ObjectId>>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "type")]
    pub type_: String,
    pub message: String,
    pub read: bool,
    pub timestamp: bson::DateTime,
    pub severity: String,
    pub device: Option<Device>,
    pub device_id: Option<ObjectId>,
    pub location: Option<Location>,
    pub location_id: Option<ObjectId>,
    pub user: Option<User>,
    pub user_id: Option<ObjectId>,
    pub workspace: Option<Workspace>,
    pub workspace_id: Option<ObjectId>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfiguration {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub software: String,
    pub config_json: serde_json::Value,
    pub device: Device,
    pub device_id: ObjectId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareUpdate {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub software: Software,
    pub software_id: ObjectId,
    pub version: String,
    pub update_type: String,
    pub release_notes: Option<String>,
    pub status: String,
    pub device: Device,
    pub device_id: ObjectId,
    pub initiated_by: Option<User>,
    pub user_id: Option<ObjectId>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Software {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub version: String,
    pub download_path: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub description: Option<String>,
    pub release_date: DateTime<Utc>,
    pub checksum: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub software_updates: Vec<SoftwareUpdate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiKey {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub key: String,
    pub name: String,
    pub user: User,
    pub user_id: ObjectId,
    pub active: bool,
    pub max_requests_per_day: Option<i32>,
    pub max_requests_per_hour: Option<i32>,
    pub max_request_duration: Option<i32>,
    pub blocked: bool,
    pub security_settings: Option<serde_json::Value>,
    pub permissions: Vec<ApiPermission>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub api_logs: Vec<ApiLog>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiPermission {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub api_key: ApiKey,
    pub api_key_id: ObjectId,
    pub access_type: String,
    pub resource: String,
    pub resource_id: Option<ObjectId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiLog {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub api_key: ApiKey,
    pub api_key_id: ObjectId,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub response_time: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RefreshToken {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub token: String,
    pub user_id: ObjectId,
    pub user: User,
    pub revoked: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


