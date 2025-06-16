use crate::{
    modules::api::{filters::UserInfo, routes::DeviceControllerQueries},
    shared::{
        db::{models::SensorValue, DBAccessManager},
        errors::{AppError, ErrorMessage, ErrorType},
    },
};
use bytes::Bytes;
use chrono::NaiveDateTime;
use std::convert::Infallible;
use utoipa::ToSchema;
use warp::{hyper::Body, reply::Response};

#[allow(unused)]
#[derive(ToSchema)]
struct DevicesDataResponse {
    name: String,
    online: bool,
    #[schema(rename = "type")]
    type_: String,
    settings: DevicesSettings,
    data: DevicesData,
}

#[allow(unused)]
#[derive(ToSchema)]
struct DevicesSettings {
    timezone: String,
    tminterval: String,
}

#[allow(unused)]
#[derive(ToSchema)]
struct DevicesData {
    timestamp: String,
    sensor: String,
}

#[utoipa::path(
        get,
        path = "/devices/data",
        params(DeviceControllerQueries),
        responses(
            (status = 200, description = "Devices datas received", body = Vec<DevicesDataResponse>),
            (status = 400, description = "Dates parse error", body = ErrorMessage),
            (status = 403, description = "Authorization token invalid or expired", body = ErrorMessage),
            (status = 500, description = "Internal Server Error", body = ErrorMessage),
        )
    )
]
pub async fn devices_data_handler(
    user_info: UserInfo,
    params: DeviceControllerQueries,
    mut db: DBAccessManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    let (start, end) = params_to_naive_datetime(params)?;

    let mut res = db
        .get_device_data(user_info.id, start, end)
        .map_err(|err| warp::reject::custom(err))?;

    for reading in &mut res {
        let rain_sum: f64 = reading
            .data
            .iter()
            .filter(|d| d.sensor_type == "rain")
            .map(|d| d.value)
            .sum();

        reading.data.retain(|d| d.sensor_type != "rain");
        reading.data.push(SensorValue {
            sensor_type: String::from("rain"),
            value: rain_sum,
        });
    }

    let stream = async_stream::stream! {
        yield Ok::<Bytes, Infallible>(Bytes::from("["));

        let mut first = true;
        for res_value in res.iter() {
            // Check if data is after device was last seen online
            if res_value.device_last_seen < res_value.time_bucket {
                continue;
            }

            let response_json = serde_json::json!({
                "name": res_value.device_name,
                "online": true,
                "type": res_value.device_type,
                "settings": {
                    "timezone": "America/Sao_Paulo",
                    "tminterval": "10"
                },
                "data": {
                    "timestamp": res_value.time_bucket,
                    "sensor": res_value.data
                }
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

        yield Ok(Bytes::from("]"));
    };

    let body = Body::wrap_stream(stream);
    let mut res = Response::new(body);
    res.headers_mut()
        .insert("Content-Type", "application/json".parse().unwrap());

    Ok(res)
}

fn params_to_naive_datetime(
    params: DeviceControllerQueries,
) -> Result<(NaiveDateTime, NaiveDateTime), warp::Rejection> {
    let start =
        NaiveDateTime::parse_from_str(&params.start, "%Y-%m-%d %H:%M:%S").map_err(|err| {
            let res = AppError {
                err_type: ErrorType::BadRequest,
                message: err.to_string(),
            };
            warp::reject::custom(res)
        })?;
    let end = NaiveDateTime::parse_from_str(&params.end, "%Y-%m-%d %H:%M:%S").map_err(|err| {
        let res = AppError {
            err_type: ErrorType::BadRequest,
            message: err.to_string(),
        };
        warp::reject::custom(res)
    })?;

    Ok((start, end))
}
