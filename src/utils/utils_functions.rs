use super::utils_models::DeviceControllerQueries;
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
