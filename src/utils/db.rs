// src/utils/db.rs

use mongodb::{options::ClientOptions, Client};
use std::env;

pub async fn init_db() -> Client {
    // Retrieve MongoDB URI from environment variables
    let db_uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");

    // Parse the MongoDB URI and set client options
    let mut client_options = ClientOptions::parse(&db_uri)
        .await
        .expect("Failed to parse MONGODB_URI");

    client_options.app_name = Some("AuthService".to_string());

    // Initialize and return the MongoDB client
    Client::with_options(client_options).expect("Failed to initialize MongoDB client")
}
