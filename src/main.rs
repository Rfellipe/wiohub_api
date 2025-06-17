mod logger;
mod modules;
mod shared;

use crate::shared::db::pg_pool;

fn check_vars() -> Result<(), std::env::VarError> {
    let vars = vec![
        "DATABASE_URL",
        "SUPABASE_PUBLIC",
        "MQTT_CONFIG"
    ];

    for var in vars {
        std::env::var(var).expect(&format!("Variable {} should be set on env file", var));
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenvy::dotenv().ok();
    logger::start_log();

    check_vars().unwrap();
    let pg_pool = pg_pool(); // Start db

    let mqtt_task = modules::mqtt::start_mqtt(pg_pool.clone()).await;
    let api_task = modules::api::start_api(pg_pool.clone()).await;

    let _ = tokio::try_join!(api_task, mqtt_task);

    Ok(())
}
