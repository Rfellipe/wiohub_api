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
    pub device_id: uuid::Uuid,
    pub timestamp: i64,
    pub sensors: Vec<Sensors>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    #[serde(rename = "tenantId")]
    pub tenant_id: String,
    pub uuid: uuid::Uuid,
    pub mac: String,
    pub version: String,
    pub software: Option<String>,
    pub firmware: Version,
}
