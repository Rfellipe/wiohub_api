use super::models::{Data, Device};
use super::utils::{DeviceControllerQueries, ApiDeviceDataResponse};
use super::{BsonRejection, MongoRejection};
use bson::oid::ObjectId;
use chrono::{DateTime, FixedOffset, Utc};
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::{Collection, Database};
use std::convert::Infallible;
use utoipa::{ Modify, OpenApi };

#[derive(OpenApi)]
#[openapi(paths(device_data_handler))]
pub struct WiohubApi;


#[utoipa::path(
        get,
        path = "devices/data",
        params(DeviceControllerQueries),
        responses(
            (status = 200, description = "Devices datas received", body = [ApiDeviceDataResponse])
        )
    )]
pub async fn device_data_handler(
    opts: DeviceControllerQueries,
    db: Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    let start_dt: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339(&opts.start).expect("Failed to parse start string");
    let end_dt: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339(&opts.end).expect("Failed to parse end string");

    let start_utc: DateTime<Utc> = start_dt.with_timezone(&Utc);
    let end_utc: DateTime<Utc> = end_dt.with_timezone(&Utc);

    let client_id = ObjectId::parse_str("6707040fe35f054bd65e5d73")
        .map_err(|e| warp::reject::custom(BsonRejection(e)))?;

    let device_coll: Collection<Device> = db.collection("Device");
    let data_coll: Collection<Data> = db.collection("Data");

    // Fetch active device IDs
    let devices_id = device_coll
        .find(
            doc! {
                "clientId": client_id,
                "status": "active",
            },
            None,
        )
        .await
        .map_err(|e| warp::reject::custom(MongoRejection(e)))?
        .try_collect::<Vec<_>>()
        .await
        .map_err(|e| warp::reject::custom(MongoRejection(e)))?
        .into_iter()
        .map(|doc| doc.id)
        .collect::<Vec<_>>();

    // Fetch data for the devices within the time range and group by 10-minute intervals
    let pipeline = [
        doc! {
            "$match": doc! {
                "deviceId": doc! { "$in": devices_id },
                "timestamp": doc! {
                    "$gte": start_utc,
                    "$lte": end_utc,
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
                        "value": "$value"
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
                                                    "min": doc! {
                                                        "$min": "$$sensor.value"
                                                    },
                                                    "max": doc! {
                                                        "$max": "$$sensor.value"
                                                    },
                                                    "average": doc! {
                                                        "$avg": "$$sensor.value"
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
        .map_err(|e| warp::reject::custom(MongoRejection(e)))?
        .try_collect::<Vec<Document>>()
        .await
        .map_err(|e| warp::reject::custom(MongoRejection(e)))?;

    Ok(warp::reply::json(&all_data))
}

pub async fn hello_handler(s: String) -> Result<impl warp::Reply, Infallible> {
    println!("update_todo: id={}", s);

    Ok(warp::http::StatusCode::OK)
}
