pub mod jsonb_wrapper;
pub mod models;
pub mod schema;

use chrono::NaiveDateTime;
use diesel::{
    pg::PgConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};

use crate::shared::errors::AppError;

type PooledPg = PooledConnection<ConnectionManager<PgConnection>>;

pub struct DBAccessManager {
    connection: PooledPg,
}

impl DBAccessManager {
    pub fn new(connection: PooledPg) -> DBAccessManager {
        DBAccessManager { connection }
    }

    pub fn get_device_data(
        &mut self,
        user_id: String,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> Result<Vec<models::SensorDeviceResult>, AppError> {
        let sensors_data = diesel::sql_query(
            r#"          
            select
              sensor_data.device_id as device_id,
              devices.name as device_name,
              devices.device_type as device_type,
              devices.last_seen as device_last_seen,
              -- Calculate the 10-minute bucket once
              date_trunc('hour', timestamp) + INTERVAL '10 min' * floor(
                EXTRACT(
                  minute
                  from
                    timestamp
                )::int / 10
              ) as time_bucket,
              JSONB_AGG(
                JSONB_BUILD_OBJECT('sensor_type', type, 'value', value)
              ) as data
            from
              sensor_data
              inner join public.devices on sensor_data.device_id = devices.id
              inner join public.access_control_entry on devices.id = access_control_entry.device_id
            where
              access_control_entry.user_id = $1
              and sensor_data.timestamp between $2 and $3
              -- Group by the calculated 10-minute bucket only
            group by
              sensor_data.device_id,
              devices.name,
              devices.device_type,
              devices.last_seen,
              time_bucket
            order by
              time_bucket;
            "#,
        )
        .bind::<diesel::sql_types::Text, _>(user_id)
        .bind::<diesel::sql_types::Timestamp, _>(start)
        .bind::<diesel::sql_types::Timestamp, _>(end)
        .load::<models::SensorDeviceResult>(&mut self.connection)
        .map_err(|err| AppError::from_diesel_err(err, "While getting sensors data:"))?;

        Ok(sensors_data)
    }
}

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn pg_pool() -> PgPool {
    let db_url = std::env::var("DATABASE_URL").unwrap();

    let manager = ConnectionManager::<PgConnection>::new(&db_url);
    Pool::new(manager).expect("Postgres connection pool could not be created")
}
