// src/main.rs

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenvy::dotenv;
use env_logger::Env;
use std::env;
use std::time::Duration;

mod handlers;
mod middleware;
mod models;
mod utils;

use models::app_state::AppState;
use middleware::RateLimiter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize dotenvy to load environment variables
    dotenv().expect("Failed to load .env file");

    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Initialize the MongoDB client once
    let db_client = utils::db::init_db().await;

    // Retrieve JWT_SECRET and DATABASE_NAME from environment variables
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let database_name = env::var("DATABASE_NAME").expect("DATABASE_NAME must be set");

    // Initialize rate limiter
    let rate_limiter = RateLimiter::new(100, Duration::from_secs(60));

    // Create shared application state
    let app_state = web::Data::new(AppState {
        db_client,
        jwt_secret,
        rate_limiter,
        database_name,
    });

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .wrap(middleware::RateLimiterMiddleware::new(
                app_state.rate_limiter.clone(),
            ))
            .app_data(app_state.clone()) // Share the application state
            .configure(handlers::init_routes)
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
