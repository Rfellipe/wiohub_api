use super::errors::{AuthError, HashRejection, MongoRejection, SignInError};
use super::models::{Client, Data, Device, User};
use super::security::{decode_jwt, generate_jwt};
use super::utils::{ApiDeviceDataResponse, DeviceControllerQueries, SinginBody};
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use chrono::{DateTime, FixedOffset, Utc};
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::options::FindOneOptions;
use mongodb::{Collection, Database};
use utoipa::OpenApi;
use warp::http::Response;
use warp::reply::Reply;
use warp_rate_limit::{add_rate_limit_headers, RateLimitInfo};

static JWT_SECRET: &'static str = "wiohub-secret";

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
    authorization: String,
    opts: DeviceControllerQueries,
    db: Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    let token = authorization.trim_start_matches("Bearer ").to_owned();
    let user_info =
        decode_jwt(&token, &JWT_SECRET).map_err(|_e| warp::reject::custom(AuthError))?;

    println!("{:#?}", user_info);

    let start_dt: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339(&opts.start).expect("Failed to parse start string");
    let end_dt: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339(&opts.end).expect("Failed to parse end string");

    let start_utc: DateTime<Utc> = start_dt.with_timezone(&Utc);
    let end_utc: DateTime<Utc> = end_dt.with_timezone(&Utc);
    let find_options = FindOneOptions::builder()
        .projection(doc! {
            "id": 1
        })
        .build();
    let client_coll: Collection<Client> = db.collection("Client");

    let client = client_coll
        .find_one(
            doc! {
                "tenantId": user_info.tenant
            },
            find_options,
        )
        .await
        .map_err(|e| warp::reject::custom(MongoRejection(e)))?
        .ok_or_else(|| warp::reject::reject())?;

    let device_coll: Collection<Device> = db.collection("Device");
    let data_coll: Collection<Data> = db.collection("Data");

    // Fetch active device IDs
    let devices_id = device_coll
        .find(
            doc! {
                "clientId": client.id,
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

pub async fn auth_signin_handler(
    rate_limit_info: RateLimitInfo,
    body: SinginBody,
    db: Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    let user_coll: Collection<User> = db.collection("User");
    let user = user_coll
        .find_one(
            doc! {
                "email": body.email
            },
            FindOneOptions::builder()
                .projection(doc! {
                    "name": 1,
                    "email": 1,
                    "phone": 1,
                    "password": 1,
                    "role": 1,
                    "tenantId": 1
                })
                .build(),
        )
        .await
        .map_err(|e| warp::reject::custom(MongoRejection(e)))?
        .ok_or_else(|| warp::reject::custom(SignInError))?;


    let password = user.clone().password;
    let id = user.clone().id;
    let tenant_id = user.tenant_id.ok_or_else(|| warp::reject::not_found())?;

    let parsed_hash =
        PasswordHash::new(&password).map_err(|e| warp::reject::custom(HashRejection(e)))?;

    let password_match = Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !password_match {
        return Err(warp::reject::custom(SignInError));
    }

    match generate_jwt(&id.to_string(), &tenant_id.to_string(), JWT_SECRET, 3600) {
        Ok(token) => {
            let mut response = warp::reply::with_status(
                token,
                warp::http::StatusCode::OK
            ).into_response();
            let _ = add_rate_limit_headers(response.headers_mut(), &rate_limit_info);
            return Ok(response)
        }
        Err(_e) => Err(warp::reject::reject()),
    }
}
