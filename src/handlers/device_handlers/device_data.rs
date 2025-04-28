use crate::errors::{mongo_error, bson_datetime_error};
use crate::handlers::auth_handlers::security::{decode_jwt, JWT_SECRET};
use crate::models::{Data, Device};
use crate::utils::{
    utils_functions::handle_time_interval,
    utils_models::{ApiDeviceDataResponse, CustomMessage, DeviceControllerQueries},
};
use bson::DateTime;
use futures::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::{Collection, Database};

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

    let (start, end) = match handle_time_interval(opts, true) {
        Ok(values) => values,
        Err(err) => {
            let response = CustomMessage {
                message: err.to_string(),
                code: warp::http::StatusCode::BAD_REQUEST.as_u16(),
            };

            return Ok(warp::reply::json(&response));
        }
    };

    let device_coll: Collection<Device> = db.collection("Device");
    let data_coll: Collection<Data> = db.collection("Data");
    
    let user_id = ObjectId::parse_str(user_info.client_id.as_ref().unwrap()).unwrap();

    // Fetch active device IDs
    let devices_id = device_coll
        .find(
            doc! {
                "clientId": user_id,
                "status": "active",
            },
            None,
        )
        .await
        .map_err(|e| {
            warp::reject::custom(mongo_error(e))
        })?
        .try_collect::<Vec<_>>()
        .await
        .map_err(|e| {
            warp::reject::custom(mongo_error(e))
        })?
        .into_iter()
        .map(|doc| doc.id)
        .collect::<Vec<_>>();

    // Fetch data for the devices within the time range and group by 10-minute intervals
    let pipeline = [
        doc! {
            "$match": doc! {
                "deviceId": doc! { "$in": devices_id },
                "timestamp": doc! {
                    "$gte": DateTime::parse_rfc3339_str(start).map_err(|e| warp::reject::custom(bson_datetime_error(e)))?,
                    "$lte": DateTime::parse_rfc3339_str(end).map_err(|e| warp::reject::custom(bson_datetime_error(e)))?
                },
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
