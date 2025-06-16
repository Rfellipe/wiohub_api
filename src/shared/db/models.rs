use super::{jsonb_wrapper::Json, schema};
use chrono::NaiveDateTime;
use diesel::{prelude::QueryableByName, sql_types::*, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Selectable, QueryableByName)]
#[diesel(table_name = schema::devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Devices {
    #[diesel(sql_type = Int4)]
    pub id: i32,
    #[diesel(sql_type = Varchar)]
    pub name: String,
    #[diesel(sql_type = Int4)]
    pub location_id: i32,
    #[diesel(sql_type = Varchar)]
    pub device_type: String,
    #[diesel(sql_type = Int4)]
    pub device_group_id: i32,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub last_seen: Option<NaiveDateTime>,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub created_at: Option<NaiveDateTime>,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Selectable, QueryableByName)]
#[diesel(table_name = schema::sensor_data)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SensorData {
    #[diesel(sql_type = Int8)]
    pub id: i64,
    #[diesel(sql_type = Int4)]
    pub device_id: i32,
    #[diesel(sql_type = Timestamp)]
    pub timestamp: NaiveDateTime,
    #[diesel(sql_type = Varchar)]
    pub type_: String,
    #[diesel(sql_type = Float8)]
    pub value: f64,
    #[diesel(sql_type = Varchar)]
    pub unit: String,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub created_at: Option<NaiveDateTime>,
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
    pub value: f64
}
