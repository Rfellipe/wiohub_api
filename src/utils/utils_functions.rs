use super::utils_models::DeviceControllerQueries;
use std::process::Command;
use chrono::{DateTime, Utc, FixedOffset};
use chrono::ParseError;

pub fn handle_time_interval(time_interval: DeviceControllerQueries) -> Result<(DateTime<Utc>, DateTime<Utc>), ParseError> {
    let start_dt: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339(&time_interval.start).expect("Failed to parse start string");
    let end_dt: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339(&time_interval.end).expect("Failed to parse end string");

    let start_utc: DateTime<Utc> = start_dt.with_timezone(&Utc);
    let end_utc: DateTime<Utc> = end_dt.with_timezone(&Utc);

    Ok((start_utc, end_utc))
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
        .args(&["-z", zabbix_server, "-s", hostname, "-k", metric, "-o", &value.to_string()])
        .output()
        .expect("Failed to send data to Zabbix");

    if !output.status.success() {
        eprintln!("Zabbix Sender failed: {:?}", output);
    }
}
