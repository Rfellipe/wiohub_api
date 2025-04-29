use crate::errors::{bson_datetime_error, mongo_error, AppError, ErrorType};
use crate::handlers::auth_handlers::security::{decode_jwt, JWT_SECRET};
use crate::models::Data;
use crate::utils::{
    utils_functions::handle_time_interval,
    utils_models::{ApiDeviceDataResponse, CustomMessage, DeviceControllerQueries},
};
use bson::DateTime;
use bytes::Bytes;
use futures::{
    stream::StreamExt,
    TryStreamExt,
};
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::{Collection, Database};
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::Infallible;
use std::time::Instant;
use warp::hyper::Body;
use warp::reply::Response;

#[derive(Debug, Deserialize)]
struct AggregatedData {
    #[serde(rename = "_id")]
    pub id: AggregationKey,
    pub data: Vec<SensorData>,
    // pub device: AggregationDevice,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AggregationKey {
    pub device_id: bson::oid::ObjectId,
    pub timestamp: String, // Already formatted as "%Y-%m-%dT%H:%M:00"
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SensorData {
    pub sensor_type: String,
    pub value: serde_json::Value, // Can be a number or an array
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
struct AggregationDevice {
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub point: String,
    pub serial: String,
    pub client_id: bson::oid::ObjectId,
    pub transmission_interval: i32,
    pub mode: String,
    pub created_at: bson::DateTime,
    pub updated_at: bson::DateTime,
    pub location_id: bson::oid::ObjectId,
    pub timezone: Option<i32>,
    pub status: String,
}

#[utoipa::path(
        get,
        path = "devices/data",
        params(DeviceControllerQueries),
        responses(
            (status = 200, description = "Devices datas received", body = [ApiDeviceDataResponse]),
            (status = 400, description = "Dates parse error", body = [CustomMessage]),
            (status = 403, description = "Authorization token invalid or expired", body = String),
            (status = 500, description = "Internal Server Error", body = String),
        )
    )
]
pub async fn devices_data_handler(
    authorization: String,
    opts: DeviceControllerQueries,
    db: Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    let user_info = decode_jwt(authorization, &JWT_SECRET, db.clone()).await?;

    let (start, end) = match handle_time_interval(opts, false) {
        Ok(values) => values,
        Err(err) => {
            let res = AppError {
                err_type: ErrorType::BadRequest,
                message: err.to_string(),
            };
            return Err(warp::reject::custom(res));
        }
    };

    let device_coll: Collection<AggregationDevice> = db.collection("Device");
    let data_coll: Collection<Data> = db.collection("Data");

    let user_id = ObjectId::parse_str(user_info.client_id.as_ref().unwrap()).unwrap();

    // Fetch active device IDs
    let devices_map: HashMap<ObjectId, AggregationDevice> = device_coll
        .find(doc! { "clientId": user_id, "status": "active" }, None)
        .await
        .map_err(|e| {
            println!("erro1");
            return warp::reject::custom(mongo_error(e))
        })?
        .try_collect::<Vec<_>>()
        .await
        .map_err(|e| {
            println!("erro2");
            return warp::reject::custom(mongo_error(e))
        })?
        .into_iter()
        .map(|d| (d.id.clone(), d))
        .collect();

    let devices_id: Vec<ObjectId> = devices_map.clone().into_keys().collect();

    // Fetch data for the devices within the time range and group by 10-minute intervals
    let pipeline = [
        doc! {
            "$match": doc! {
                "deviceId": doc! { "$in": &devices_id },
                "timestamp": doc! {
                    "$gte": DateTime::parse_rfc3339_str(start).map_err(|e| warp::reject::custom(bson_datetime_error(e)))?,
                    "$lte": DateTime::parse_rfc3339_str(end).map_err(|e| warp::reject::custom(bson_datetime_error(e)))?
                },
            }
        },
        doc! {
            "$project": doc! {
                "deviceId": 1,
                "timestamp": 1,
                "sensorType": 1,
                "value": 1,
            }
        },
        doc! {
            "$group": doc! {
                "_id": doc! {
                    "deviceId": "$deviceId",
                    "timestamp": doc! {
                        "$dateToString": doc! {
                            "format": "%Y-%m-%dT%H:%M:00",
                            "date": doc! {
                                "$dateTrunc": doc! {
                                    "date": "$timestamp",
                                    "unit": "minute",
                                    "binSize": 10,
                                },
                            },
                        },
                    }
                },
                "data": doc! {
                    "$push": doc! {
                        "sensorType": "$sensorType",
                        "value": "$value"
                    }
                }
            }
        },
        doc! {
            "$sort": doc! {
                "_id.timestamp": 1
            }
        },
    ];

    let start = Instant::now();
    let mut cursor = data_coll
        .aggregate(pipeline, None)
        .await
        .map_err(|e| {
            return warp::reject::custom(mongo_error(e))
        })?;
    let duration = start.elapsed();

    println!("Time elapsed in generating cursor is: {:?}", duration);

    let stream = async_stream::stream! {
        let start = Instant::now();
        yield Ok::<Bytes, Infallible>(Bytes::from("["));

        let mut first = true;
        // let mut grouped_devices: HashMap<AggregationDevice, Vec<serde_json::Value>> = HashMap::new();

        while let Some(doc_result) = cursor.next().await {
            match doc_result {
                Ok(doc) => {

                    let agg_data: AggregatedData = match bson::from_document(doc) {
                        Ok(data) => data,
                        Err(e) => {
                            eprintln!("Deserialization error: {:#?}", e);
                            continue;
                        }
                    };

                    let device = match devices_map.get(&agg_data.id.device_id) {
                        Some(dev) => dev,
                        None => continue
                    };

                    let sensor_map: HashMap<String, serde_json::Value> = agg_data.data.into_iter()
                        .map(|sensor| {
                            let clean_val = match sensor.value {
                                serde_json::Value::Number(n) => serde_json::json!(n.as_f64().unwrap_or(0.0)),
                                _ => serde_json::json!(0.0),
                            };
                            (sensor.sensor_type, serde_json::json!({ "values": clean_val }))
                        }).collect();

                    let response_json = serde_json::json!({
                        "name": device.name,
                        "online": true,
                        "status": device.status,
                        "type": device.type_,
                        "serial": device.serial,
                        "settings": {
                            "timezone": "America/Sao_Paulo",
                            "tminterval": "10"
                        },
                        "location": device.point,
                        "data": [
                            {
                                "timestamp": agg_data.id.timestamp,
                                "sensor": sensor_map
                            }
                        ]
                    });

                    if !first {
                        yield Ok(Bytes::from(","));
                    }
                    first = false;
                    match serde_json::to_vec(&response_json) {
                        Ok(json) => {
                            yield Ok(Bytes::from(json));
                        }
                        Err(e) => {
                            eprintln!("Serialization error: {:?}", e);
                            break; // Stop the stream on serialization error
                        }
                    }
                }
                Err(e) => {
                    eprintln!("MongoDB cursor error: {:?}", e);
                    break; // Stop the stream on DB error
                }
            }
        }

        let duration = start.elapsed();

        println!("Time elapsed in generating stream is: {:?}", duration);
        yield Ok(Bytes::from("]"));
    };

    let body = Body::wrap_stream(stream);
    let mut res = Response::new(body);
    res.headers_mut()
        .insert("Content-Type", "application/json".parse().unwrap());

    Ok(res)
}

#[utoipa::path(
        get,
        path = "devices/data/{id}",
        params(
            ("id" = String, Path, description = "Device database id to get data from"),
            DeviceControllerQueries
        ),
        responses(
            (status = 200, description = "Device datas received", body = [ApiDeviceDataResponse]),
            (status = 400, description = "Dates parse error", body = [CustomMessage]),
            (status = 403, description = "Authorization token invalid or expired", body = String),
            (status = 500, description = "Internal Server Error", body = String),
        ),
    )
]
pub async fn device_data_handler(
    device_id: String,
    authorization: String,
    opts: DeviceControllerQueries,
    db: Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    let _user_info = decode_jwt(authorization, &JWT_SECRET, db.clone()).await?;

    let (start, end) = match handle_time_interval(opts, false) {
        Ok(values) => values,
        Err(err) => {
            let response = CustomMessage {
                message: err.to_string(),
                code: warp::http::StatusCode::BAD_REQUEST.as_u16(),
            };

            return Ok(warp::reply::json(&response));
        }
    };

    let data_coll: Collection<Data> = db.collection("Data");

    // Fetch data for the devices within the time range and group by 10-minute intervals
    let pipeline = [
        doc! {
            "$match": doc! {
                "deviceId": ObjectId::parse_str(device_id).unwrap(),
                "timestamp": doc! {
                    "$gte": DateTime::parse_rfc3339_str(start).map_err(|e| warp::reject::custom(bson_datetime_error(e)))?,
                    "$lte": DateTime::parse_rfc3339_str(end).map_err(|e| warp::reject::custom(bson_datetime_error(e)))?
                }
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "Device",
                "localField": "deviceId",
                "foreignField": "_id",
                "as": "device"
            }
        },
        doc! {
            "$unwind": doc! {
                "path": "$device"
            }
        },
        doc! {
            "$group": doc! {
                "_id": doc! {
                    "deviceId": "$deviceId",
                    "timestamp": doc! {
                        "$dateToString": doc! {
                            "format": "%Y-%m-%dT%H:%M:00",
                            "date": doc! {
                                "$dateTrunc": doc! {
                                    "date": "$timestamp",
                                    "unit": "minute",
                                    "binSize": 10,
                                },
                            },
                        },
                    }
                },
                "data": doc! {
                    "$push": doc! {
                        "sensorType": "$sensorType",
                        "value": doc! {
                            "$cond": doc! {
                                "if": doc! {
                                    "$eq": [
                                        "$value",
                                        0
                                    ]
                                },
                                "then": 0,
                                "else": "$value"
                            },
                            "$cond": doc! {
                                "if": doc! {
                                    "$eq": [
                                        "$sensorType",
                                        "rain"
                                    ]
                                },
                                "then": doc! {
                                    "$sum": "$value"
                                },
                                "else": "$value"
                            }
                        }
                    }
                },
                "device": doc! {
                    "$first": "$device"
                }
            }
        },
        doc! {
            "$sort": doc! {
                "_id.timestamp": 1
            }
        },
        doc! {
            "$project": doc! {
                "_id": 0,
                "name": "$device.name",
                "online": doc! {
                    "$literal": true
                },
                "status": "$device.status",
                "type": "$device.type",
                "serial": "$device.serial",
                "settings": doc! {
                    "timezone": "America/Sao_Paulo",
                    "tminterval": "10"
                },
                "location": "$device.point",
                "data": doc! {
                    "$map": doc! {
                        "input": [
                            "$_id.timestamp"
                        ],
                        "as": "timestamp",
                        "in": doc! {
                            "timestamp": "$$timestamp",
                            "sensors": doc! {
                                "$arrayToObject": doc! {
                                    "$map": doc! {
                                        "input": "$data",
                                        "as": "sensor",
                                        "in": doc! {
                                            "k": "$$sensor.sensorType",
                                            "v": doc! {
                                                "unit": "$$sensor.unit",
                                                "values": doc! {
                                                    "$cond": doc! {
                                                        "if": doc! {
                                                            "$eq": [
                                                                "$$sensor.value",
                                                                0
                                                            ]
                                                        },
                                                        "then": 0,
                                                        "else": "$$sensor.value"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        doc! {
            "$group": doc! {
                "_id": "$name",
                "name": doc! {
                    "$first": "$name"
                },
                "online": doc! {
                    "$first": "$online"
                },
                "status": doc! {
                    "$first": "$status"
                },
                "type": doc! {
                    "$first": "$type"
                },
                "serial": doc! {
                    "$first": "$serial"
                },
                "settings": doc! {
                    "$first": "$settings"
                },
                "location": doc! {
                    "$first": "$location"
                },
                "data": doc! {
                    "$push": "$data"
                }
            }
        },
    ];

    let all_data = data_coll
        .aggregate(pipeline, None)
        .await
        .map_err(|e| warp::reject::custom(mongo_error(e)))?
        .try_collect::<Vec<Document>>()
        .await
        .map_err(|e| warp::reject::custom(mongo_error(e)))?;

    Ok(warp::reply::json(&all_data))
}
