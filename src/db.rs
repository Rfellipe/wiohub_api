use log::info;
use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client,
};

use crate::config::DatabaseConfig;

pub async fn get_db(config: DatabaseConfig) -> mongodb::error::Result<mongodb::Database> {
    // A Client is needed to connect to MongoDB:
    // An extra line of code to work around a DNS issue on Windows:
    let options =
        ClientOptions::parse_with_resolver_config(&config.uri, ResolverConfig::cloudflare())
            .await?;
    let client = Client::with_options(options)?;

    info!("Connected to MongoDB! db: {}", config.db.clone());

    Ok(client.database(&config.db))
}
