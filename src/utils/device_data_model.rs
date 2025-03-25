use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MinMaxValues {
    pub timestamp: i64,
    pub value: f32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sensors {
    #[serde(rename = "type")]
    pub _type: String,
    pub unit: Option<String>,
    pub min: Option<MinMaxValues>,
    pub max: Option<MinMaxValues>,
    pub average: Option<f32>,
    pub values: Option<Vec<MinMaxValues>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceMessage {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    pub timestamp: i64,
    pub sensors: Vec<Sensors>
}
