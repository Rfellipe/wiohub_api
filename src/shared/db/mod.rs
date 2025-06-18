pub mod jsonb_wrapper;
pub mod models;
pub mod schema;

use crate::shared::{
    db::models::{Device, NewDevice, NewSensorData, SensorData},
    errors::{AppError, ErrorType},
};
use chrono::NaiveDateTime;
use diesel::{
    pg::PgConnection,
    prelude::*,
    // query_builder::{QueryFragment, QueryId},
    // quIery_dsl::methods::LoadQuery,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};
// use uuid::Uuid;

type PooledPg = PooledConnection<ConnectionManager<PgConnection>>;

pub struct DBAccessManager {
    connection: PooledPg,
}

impl DBAccessManager {
    pub fn new(connection: PooledPg) -> DBAccessManager {
        DBAccessManager { connection }
    }

    pub fn get_devices_data(
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

    pub fn add_device(
        &mut self,
        new_device: NewDevice,
    ) -> Result<Device, Box<dyn std::error::Error + Send + Sync>> {
        use schema::devices::dsl::devices;

        let dev = diesel::insert_into(devices)
            .values(&new_device)
            .get_result::<Device>(&mut self.connection)?;
        Ok(dev)
    }

    pub fn save_device_data(
        &mut self,
        new_datas: Vec<NewSensorData>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use schema::sensor_data::dsl::sensor_data;

        let _datas = diesel::insert_into(sensor_data)
            .values(&new_datas)
            .get_results::<SensorData>(&mut self.connection)?;

        Ok(())
    }
}

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn pg_pool() -> PgPool {
    let db_url = std::env::var("DATABASE_URL").unwrap();

    let manager = ConnectionManager::<PgConnection>::new(&db_url);
    Pool::new(manager).expect("Postgres connection pool could not be created")
}

pub fn get_db_access_manager(pool: PgPool) -> Result<DBAccessManager, AppError> {
    // .select(Devices::as_select())
    match pool.get() {
        Ok(conn) => Ok(DBAccessManager::new(conn)),
        Err(err) => {
            log::error!("Error getting database access manager: {}", err.to_string());
            Err(AppError::new(
                "Error getting connection from pool",
                ErrorType::Internal,
            ))
        }
    }
}

// pub fn get_device(
//     &mut self,
//     device_serial: uuid::Uuid,
// ) -> Result<Option<Device>, Box<dyn std::error::Error + Send + Sync>> {
//     use schema::devices::dsl::{devices, id};
//
//     let device = devices
//         .filter(id.eq(&device_serial))
//         .first::<Device>(&mut self.connection)
//         .optional()?;
//
//     Ok(device)
// }
