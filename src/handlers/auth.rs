// src/handlers/auth.rs

use std::env;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use crate::models::app_state::AppState;
use bcrypt::{verify};
use jsonwebtoken::{encode, Header, EncodingKey};
use mongodb::bson;
use serde::Serialize;
use mongodb::bson::{doc, Bson};
use crate::handlers::user::Account;

/// Struct to deserialize incoming login requests
#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Struct to serialize JWT claims
#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    username: String,
    roles: Vec<String>
}

/// Handler for user login
pub async fn login(
    data: web::Data<AppState>,
    login_info: web::Json<LoginRequest>,
) -> impl Responder {
    // Access the MongoDB collection
    let collection = data
        .db_client
        .database(&env::var("DATABASE_NAME").expect("DATABASE_NAME must be set"))
        .collection("users");

    // Find the user by email
    let filter = doc! { "email": &login_info.email };
    let user = collection.find_one(filter, None).await.unwrap();

    match user {
        Some(user_doc) => {
            let user_bson: bson::Document = user_doc;
            let hashed_password = user_bson.get_str("password").unwrap_or("").to_string();
            let user_id = user_bson.get_object_id("_id").unwrap().to_hex();

            // Verify the password
            if verify(&login_info.password, &hashed_password).unwrap_or(false) {
                // Create JWT claims
                let account: Account = bson::from_bson(Bson::Document(user_bson)).unwrap();
                let claims = Claims {
                    sub: user_id,
                    exp: 10000000000, // Set appropriate expiration
                    username: account.username,
                    roles: account.roles
                };

                // Encode the JWT
                let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(data.jwt_secret.as_ref()),
                )
                    .unwrap();

                HttpResponse::Ok().json(serde_json::json!({ "token": token }))
            } else {
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        None => HttpResponse::Unauthorized().body("User not found"),
    }
}
