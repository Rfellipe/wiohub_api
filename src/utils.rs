use serde::{Deserialize, Serialize};

// Query for devices/data route
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceControllerQueries {
    pub start: String,
    pub end: String,
}

