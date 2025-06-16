pub mod zabbix;
pub mod errors;
pub mod db;

use serde::{Deserialize, Deserializer};
use chrono::NaiveDateTime;

pub fn deserialize_naive_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    println!("s: {}", s);
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
        .map_err(serde::de::Error::custom)
}
