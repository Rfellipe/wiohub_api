use super::{jsonb_wrapper::Json, schema};
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::{
    prelude::{Insertable, Queryable, QueryableByName},
    sql_types::*,
    Selectable,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = schema::devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Device {
    pub id: uuid::Uuid,
    pub workspace_id: uuid::Uuid,
    pub name: String,
    pub type_: String,
    pub status: String,
    pub location: Option<String>,
    pub last_seen_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub location_id: Option<uuid::Uuid>,
}

#[derive(Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = schema::devices)]
pub struct NewDevice {
    pub workspace_id: uuid::Uuid,
    pub name: String,
    pub type_: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = schema::device_metrics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeviceMetric {
    id: uuid::Uuid,
    workspace_id: uuid::Uuid,
    device_id: uuid::Uuid,
    metric_name: String,
    #[diesel(sql_type = Numeric)]
    metric_value: BigDecimal,
    unit: Option<String>,
    timestamp: Option<NaiveDateTime>,
    created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = schema::device_metrics)]
pub struct NewDeviceMetric {
    pub workspace_id: uuid::Uuid,
    pub device_id: uuid::Uuid,
    pub metric_name: String,
    pub metric_value: BigDecimal,
}

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
