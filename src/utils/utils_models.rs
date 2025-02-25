use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

// Query for devices/data route
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct DeviceControllerQueries {
    pub start: String,
    pub end: String,
}

// Query for devices/status route
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct DeviceStatusQueries {
    pub serial: Option<String>
}

// Body on auth/signin route
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct SinginBody {
    pub email: String,
    pub password: String
}

// Custom message to return in routes when needed
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct CustomMessage {
    pub message: String,
    pub code: u16,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SensorData {
    #[serde(rename = "sensorType")]
    pub sensor_type: String,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Settings {
    pub timezone: String,
    pub tminterval: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Location {
    point: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DataPoint {
    timestamp: String,
    sensors: std::collections::HashMap<String, SensorValue>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SensorValue {
    unit: String,
    values: f64,
}

// Response for /devices/data route
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiDeviceDataResponse {
    #[serde (rename = "_id")]
    pub id: String,
    pub name: String,
    pub online: bool,
    pub status: String,
    #[serde (rename = "type")]
    pub _type: String,
    pub serial: String,
    pub settings: Settings,
    pub location: String,
    pub data: Vec<DataPoint>,
}
