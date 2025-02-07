use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

// Query for devices/data route
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct DeviceControllerQueries {
    pub start: String,
    pub end: String,
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
    pub data: Vec<Data>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Settings {
    pub timezone: String,
    pub tminterval: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Data {
    pub timestamp: String,
    pub sensors: Sensors
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Sensors {
    #[serde (rename = "sensorType")]
    pub sensor_type: Vec<SensorValues>
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SensorValues {
    pub value: MinMaxAvg
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MinMaxAvg {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub average: Option<f64>,
}
