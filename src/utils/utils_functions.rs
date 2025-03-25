use crate::models::{Filter, Workspace};
use super::utils_models::DeviceControllerQueries;
use bson::oid::ObjectId;
use bson::Document;
use chrono::ParseError;
use chrono::{DateTime, FixedOffset, Utc};
use futures::TryStreamExt;
use mongodb::{bson::doc, Collection, Database};
use std::process::Command;

pub async fn find_device_filter(sensor_type: String, device_id: ObjectId, db: Database) -> Result<Option<Filter>, mongodb::error::Error> {
    let filter_coll: Collection<Filter> = db.collection("Filter");
    let mongo_filter = doc! {
        "sensorType": sensor_type,
        "deviceId": device_id
    };

    let filter = filter_coll.find_one(mongo_filter, None)
        .await?;
    
    Ok(filter)
} 

pub async fn find_workspace_with_device_id(
    device_id: ObjectId,
    db: Database,
) -> Result<Vec<Document>, mongodb::error::Error> {
    let workspace_coll: Collection<Workspace> = db.collection("Workspace");

    let pipeline = [
        doc! {
            "$lookup": doc! {
                "from": "Location",
                "localField": "locationId",
                "foreignField": "_id",
                "as": "matchedLocations"
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "Device",
                "localField": "matchedLocations._id",
                "foreignField": "locationId",
                "as": "matchedDevices"
            }
        },
        doc! {
            "$match": doc! {
                "matchedDevices._id": device_id,
            }
        },
        doc! {
            "$project": doc! {
                "clientId": 1,
                "locations": doc! {
                    "$filter": doc! {
                        "input": "$matchedLocations",
                        "as": "location",
                        "cond": doc! {
                            "$in": [
                                "$$location._id",
                                "$matchedDevices.locationId"
                            ]
                        }
                    }
                }
            }
        },
        doc! {
            "$project": doc! {
                "clientId": 1,
                "locations": doc! {
                    "_id": 1
                }
            }
        },
    ];

    let workspace = workspace_coll
        .aggregate(pipeline, None)
        .await?
        .try_collect::<Vec<Document>>()
        .await?;

    Ok(workspace)
}

pub fn handle_time_interval(
    time_interval: DeviceControllerQueries,
) -> Result<(String, String), ParseError> {
    let start_dt: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339(&time_interval.start).expect("Failed to parse start string");
    let end_dt: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339(&time_interval.end).expect("Failed to parse end string");

    let start_utc: DateTime<Utc> = start_dt.with_timezone(&Utc);
    let end_utc: DateTime<Utc> = end_dt.with_timezone(&Utc);

    #[expect(deprecated)]
    let target_offset = FixedOffset::east(3 * 3600);

    let start_target = start_utc.with_timezone(&target_offset);
    let end_target = end_utc.with_timezone(&target_offset);

    let start_res = start_target.to_rfc3339();
    let end_res = end_target.to_rfc3339();

    Ok((start_res, end_res))
}

pub fn send_to_zabbix(metric: &str, value: String) {
    let hostname = "api_rust"; // Change this to match your Zabbix hostname
    let zabbix_server = "192.168.122.116"; // Replace with your Zabbix server IP

    // verify that the zabbix_sender is installed
    if !Command::new("which")
        .arg("zabbix_sender")
        .output()
        .expect("Failed to check if zabbix_sender is installed")
        .status
        .success()
    {
        eprintln!("zabbix_sender is not installed");
        return;
    }

    let output = Command::new("zabbix_sender")
        .args(&[
            "-z",
            zabbix_server,
            "-s",
            hostname,
            "-k",
            metric,
            "-o",
            &value.to_string(),
        ])
        .output()
        .expect("Failed to send data to Zabbix");

    if !output.status.success() {
        eprintln!("Zabbix Sender failed: {:?}", output);
    }
}

