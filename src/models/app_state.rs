use mongodb::Client;
use crate::middleware::RateLimiter;

pub struct AppState {
    pub db_client: Client,
    pub jwt_secret: String,
    pub rate_limiter: RateLimiter,
    pub database_name: String, // Added to store the database name
}