use super::{jsonb_wrapper::Json, schema};
use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime};
use diesel::{
    prelude::{Insertable, Queryable, QueryableByName},
    sql_types::*,
    Selectable,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = schema::connection_protocols)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ConnectionProtocol {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = schema::device_categories)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeviceCategories {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = schema::device_manufacturers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeviceManufacturers {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = schema::device_status)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeviceStatus {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = schema::devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Device {
    pub id: uuid::Uuid,
    pub location_id: Option<uuid::Uuid>,
    pub device_type: String,
    pub serial_number: Option<String>,
    pub status: Option<String>,
    pub last_seen_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub category: Option<String>,
    pub connection_protocol: Option<String>,
    pub credentials: Option<Json<serde_json::Value>>,
    pub mqtt_config: Option<Json<serde_json::Value>>,
    pub http_config: Option<Json<serde_json::Value>>,
    pub ftp_config: Option<Json<serde_json::Value>>,
    pub tcp_config: Option<Json<serde_json::Value>>,
    pub modbus_config: Option<Json<serde_json::Value>>,
    pub opcua_config: Option<Json<serde_json::Value>>,
    pub lorawan_config: Option<Json<serde_json::Value>>,
    pub firmware: Option<String>,
    pub hardware_serial: Option<String>,
    pub api_serial: Option<String>,
    pub tags: Option<Vec<Option<String>>>,
    pub data_format: Option<String>,
    pub sample_rate: Option<i32>,
    pub battery_powered: Option<bool>,
    pub battery_level: Option<f32>,
    pub installation_date: Option<NaiveDate>,
    pub last_maintenance: Option<NaiveDate>,
    pub next_maintenance: Option<NaiveDate>,
    pub notes: Option<String>,
    pub responsible_person: Option<String>,
    pub custom_fields: Option<Json<serde_json::Value>>,
    pub organization_id: Option<uuid::Uuid>,
    pub workspace_id: Option<uuid::Uuid>,
    pub updated_at: Option<NaiveDateTime>,
    pub manufacturer_id: Option<i32>,
    pub category_id: Option<i32>,
    pub status_id: Option<i32>,
    pub protocol_id: Option<i32>,
}

#[derive(Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = schema::devices)]
pub struct NewDevice {
    pub serial_number: String,
    pub status: String,
    pub name: String,
    pub firmware: String,
    pub organization_id: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinMaxValues {
    pub timestamp: i64,
    pub value: f32,
}

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = schema::sensor_data)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SensorData {
    pub id: uuid::Uuid,
    pub device_id: uuid::Uuid,
    pub timestamp: NaiveDateTime,
    pub type_: String,
    pub min: Json<MinMaxValues>,
    pub max: Json<MinMaxValues>,
    pub average: BigDecimal,
    pub values: Json<Vec<MinMaxValues>>,
    pub unit: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = schema::sensor_data)]
pub struct NewSensorData {
    pub device_id: uuid::Uuid,
    pub timestamp: NaiveDateTime,
    pub type_: String,
    pub min: Json<MinMaxValues>,
    pub max: Json<MinMaxValues>,
    pub average: BigDecimal,
    pub values: Json<Vec<MinMaxValues>>,
    pub unit: String,
}

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = schema::locations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Locations {
    pub id: uuid::Uuid,
    pub workspace_id: Option<uuid::Uuid>,
    pub name: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub description: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub address: Option<String>,
    pub last_updated_at: Option<NaiveDateTime>,
    pub image: Option<String>,
    pub type_: Option<String>,
    pub status: Option<String>,
    pub perimeter: Option<Json<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = schema::organization_users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrganizationUsers {
    pub id: uuid::Uuid,
    pub organization_id: Option<uuid::Uuid>,
    pub user_id: Option<uuid::Uuid>,
    pub role: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = schema::organizations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Organizations {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Users {
    pub id: uuid::Uuid,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub current_workspace_id: Option<uuid::Uuid>,
    pub default_organization_id: Option<uuid::Uuid>,
    pub organization_id: Option<uuid::Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = schema::workspace_users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkspaceUsers {
    pub id: uuid::Uuid,
    pub workspace_id: Option<uuid::Uuid>,
    pub user_id: Option<uuid::Uuid>,
    pub role: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = schema::workspaces)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Workspaces {
    pub id: uuid::Uuid,
    pub organization_id: Option<uuid::Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

/// API related models
#[derive(Debug, Serialize, Deserialize, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SensorDeviceResult {
    #[diesel(sql_type = Integer)]
    pub device_id: i32,

    #[diesel(sql_type = Varchar)]
    pub device_name: String,

    #[diesel(sql_type = Varchar)]
    pub device_type: String,

    #[diesel(sql_type = Timestamp)]
    pub device_last_seen: NaiveDateTime,

    #[diesel(sql_type = Timestamp)]
    pub time_bucket: NaiveDateTime,

    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    pub data: Json<Vec<SensorValue>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensorValue {
    pub sensor_type: String,
    pub value: f64,
}
