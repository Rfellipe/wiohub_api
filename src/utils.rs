use super::models::Data;
use chrono::{DateTime, Duration, TimeZone, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Query for devices/data route
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceControllerQueries {
    pub start: String,
    pub end: String,
}

// Sensor Values
#[derive(Debug, Serialize, Deserialize)]
pub struct Values {
    min: Option<f32>,
    max: Option<f32>,
    average: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedData {
    timestamp: DateTime<Utc>,
    rain: RainData,
    temperature: SensorData,
    humidity: SensorData,
    pressure: SensorData,
    wind_speed: SensorData,
    wind_dir: SensorData,
    lux: LuxData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RainData {
    unit: String,
    accumulation: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorData {
    unit: String,
    values: Values,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LuxData {
    unit: String,
    values: Option<Values>,
}

fn round_to_ten_minutes(dt: DateTime<Utc>) -> DateTime<Utc> {
    let minutes = dt.minute();
    let rounded_minutes = (minutes / 10) * 10;
    dt.with_minute(rounded_minutes)
        .and_then(|dt| dt.with_second(0))
        .and_then(|dt| dt.with_nanosecond(0))
        .unwrap_or(dt)
}

fn update_average(existing_average: Option<f32>, new_value: f32) -> f32 {
    match existing_average {
        Some(avg) => (avg + new_value) / 2.0,
        None => new_value,
    }
}

pub fn group_by_ten_minutes(data: Vec<Data>) -> Vec<Data> {
    let mut grouped: HashMap<(DateTime<Utc>, String), Data> = HashMap::new();

    for item in data {
        let rounded_date = round_to_ten_minutes(item.timestamp);
        let key = (rounded_date, item.sensor_type.clone());

        if item.sensor_type == "rain" || item.sensor_type == "rainfall" || item.sensor_type == "rainRate" {
            if let Some(existing_item) = grouped.get_mut(&key) {
                if let (Some(existing_value), Some(new_value)) = (existing_item.value, item.value) {
                    existing_item.value = Some(existing_value + new_value);
                }
            } else {
                grouped.insert(key, Data {
                    id: item.id,
                    sensor_type: item.sensor_type.clone(),
                    value: item.value,
                    unit: item.unit.clone(),
                    status: item.status.clone(),
                    timestamp: rounded_date,
                    location: item.location.clone(),
                    location_id: item.location_id.clone(),
                    device: item.device.clone(),
                    device_id: item.device_id.clone(),
                    created_at: item.created_at.clone(),
                });
            }
        } else {
            grouped.insert(key, Data {
                id: item.id,
                sensor_type: item.sensor_type.clone(),
                value: item.value,
                unit: item.unit.clone(),
                status: item.status.clone(),
                timestamp: rounded_date,
                location: item.location.clone(),
                location_id: item.location_id.clone(),
                device: item.device.clone(),
                device_id: item.device_id.clone(),
                created_at: item.created_at.clone(),
            });
        }
    }

    grouped.into_values().collect()
}

pub fn process_data_with_fill(
    data: Vec<Data>,
    date_start: DateTime<Utc>,
    date_end: DateTime<Utc>,
) -> Vec<ProcessedData> {
    let mut result_data: Vec<ProcessedData> = Vec::new();
    let mut current_date = round_to_ten_minutes(date_start);

    while current_date <= date_end {
        result_data.push(ProcessedData {
            timestamp: current_date,
            rain: RainData {
                unit: "mm".to_string(),
                accumulation: 0.0,
            },
            temperature: SensorData {
                unit: "°C".to_string(),
                values: Values {
                    min: None,
                    max: None,
                    average: None,
                },
            },
            humidity: SensorData {
                unit: "%".to_string(),
                values: Values {
                    min: None,
                    max: None,
                    average: None,
                },
            },
            pressure: SensorData {
                unit: "hPa".to_string(),
                values: Values {
                    min: None,
                    max: None,
                    average: None,
                },
            },
            wind_speed: SensorData {
                unit: "mph".to_string(),
                values: Values {
                    min: None,
                    max: None,
                    average: None,
                },
            },
            wind_dir: SensorData {
                unit: "Degrees".to_string(),
                values: Values {
                    min: None,
                    max: None,
                    average: None,
                },
            },
            lux: LuxData {
                unit: "lux".to_string(),
                values: None,
            },
        });

        current_date = current_date + Duration::minutes(10);
    }

    for item in data {
        let rounded_date = round_to_ten_minutes(item.timestamp);
        if let Some(target_interval) = result_data
            .iter_mut()
            .find(|entry| entry.timestamp == rounded_date)
        {
            if let Some(value) = item.value {
                match item.sensor_type.as_str() {
                    "temperatureMin" => target_interval.temperature.values.min = Some(value),
                    "temperatureMax" => target_interval.temperature.values.max = Some(value),
                    "temperatureAvg" => target_interval.temperature.values.average = Some(update_average(
                        target_interval.temperature.values.average,
                        value,
                    )),
                    "humidityMin" => target_interval.humidity.values.min = Some(value),
                    "humidityMax" => target_interval.humidity.values.max = Some(value),
                    "humidityAvg" => target_interval.humidity.values.average = Some(update_average(
                        target_interval.humidity.values.average,
                        value,
                    )),
                    "pressureMin" => target_interval.pressure.values.min = Some(value),
                    "pressureMax" => target_interval.pressure.values.max = Some(value),
                    "pressureAvg" => target_interval.pressure.values.average = Some(update_average(
                        target_interval.pressure.values.average,
                        value,
                    )),
                    "wind_speedMin" => target_interval.wind_speed.values.min = Some(value),
                    "wind_speedMax" => target_interval.wind_speed.values.max = Some(value),
                    "wind_speedAvg" => target_interval.wind_speed.values.average = Some(update_average(
                        target_interval.wind_speed.values.average,
                        value,
                    )),
                    "wind_directionMin" => target_interval.wind_dir.values.min = Some(value),
                    "wind_directionMax" => target_interval.wind_dir.values.max = Some(value),
                    "wind_directionAvg" => target_interval.wind_dir.values.average = Some(update_average(
                        target_interval.wind_dir.values.average,
                        value,
                    )),
                    "rain" | "rainfall" | "rainRate" => target_interval.rain.accumulation += value,
                    _ => {}
                }
            }
        }
    }

    result_data
}
