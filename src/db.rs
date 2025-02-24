use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client,
};
use std::env;

pub async fn get_db() -> mongodb::error::Result<mongodb::Database> {
    // Load the MongoDB connection string from an environment variable:
    let client_uri =
        env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");

    println!("{}",client_uri);

    // A Client is needed to connect to MongoDB:
    // An extra line of code to work around a DNS issue on Windows:
    let options =
        ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
            .await?;
    let client = Client::with_options(options)?;

    // Ok(client.database("wiohub2-0"))
    Ok(client.database("wiohub"))
}

