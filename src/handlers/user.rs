// src/handlers/user.rs

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::models::app_state::AppState;
use mongodb::bson::{doc, oid::ObjectId, Bson};
use bcrypt::{hash, DEFAULT_COST};
use mongodb::bson;

/// Struct to deserialize incoming user creation requests
#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Struct to deserialize update user requests
#[derive(Deserialize)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Account {
    #[serde(serialize_with = "bson::serde_helpers::serialize_object_id_as_hex_string")]
    pub _id: ObjectId,
    pub username: String,
    pub roles: Vec<String>
}

/// Handler to create a new user
pub async fn create_user(
    data: web::Data<AppState>,
    user: web::Json<CreateUser>,
) -> impl Responder {
    // Access the MongoDB collection with explicit type annotation
    let collection: mongodb::Collection<bson::Document> = data
        .db_client
        .database(&data.database_name)
        .collection("users");

    // Hash the user's password
    let hashed_password = match hash(&user.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to hash password"),
    };

    // Create a BSON document to insert
    let new_user = doc! {
        "username": &user.username,
        "email": &user.email,
        "password": hashed_password,
        "roles": ["ROLE_PLAYER"]
        // Add other fields as necessary
    };

    // Insert the new user into the collection
    match collection.insert_one(new_user, None).await {
        Ok(insert_result) => {
            match collection.find_one(doc! { "_id": insert_result.inserted_id }, None).await {
                Ok(Some(user_doc)) =>{
                    let account: Account = bson::from_bson(Bson::Document(user_doc)).unwrap();
                    HttpResponse::Ok().json(account)
                }
                Ok(None) => HttpResponse::NotFound().body("User not found"),
                Err(e) => {
                    eprintln!("Error fetching user: {}", e);
                    HttpResponse::InternalServerError().body("Failed to fetch user")
                }
            }
        }
        Err(e) => {
            eprintln!("Error inserting user: {}", e);
            HttpResponse::InternalServerError().body("Failed to create user")
        }
    }
}

/// Handler to get a user's information
pub async fn get_user(
    data: web::Data<AppState>,
    user_id: web::Path<String>,
) -> impl Responder {
    // Access the MongoDB collection with explicit type annotation
    let collection: mongodb::Collection<bson::Document> = data
        .db_client
        .database(&data.database_name)
        .collection("users");

    // Parse the user ID
    let object_id = match ObjectId::parse_str(&*user_id) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().body("Invalid user ID format"),
    };

    // Find the user by ID
    let filter = doc! { "_id": object_id };
    match collection.find_one(filter, None).await {
        Ok(Some(user_doc)) => {
            // Exclude the password from the response
            //let mut user_doc = user_doc;
            //user_doc.remove("password");
            //user_doc.remove("email");
            let account: Account = bson::from_bson(Bson::Document(user_doc)).unwrap();
            HttpResponse::Ok().json(account)
        }
        Ok(None) => HttpResponse::NotFound().body("User not found"),
        Err(e) => {
            eprintln!("Error fetching user: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch user")
        }
    }
}

/// Handler to update an existing user
pub async fn update_user(
    data: web::Data<AppState>,
    user_id: web::Path<String>,
    update_info: web::Json<UpdateUser>,
) -> impl Responder {
    // Access the MongoDB collection with explicit type annotation
    let collection: mongodb::Collection<bson::Document> = data
        .db_client
        .database(&data.database_name)
        .collection("users");

    // Parse the user ID
    let object_id = match ObjectId::parse_str(&*user_id) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().body("Invalid user ID format"),
    };

    // Build the update document
    let mut update_doc = doc! {};
    if let Some(username) = &update_info.username {
        update_doc.insert("username", username);
    }
    if let Some(email) = &update_info.email {
        update_doc.insert("email", email);
    }
    if let Some(password) = &update_info.password {
        let hashed_password = match hash(password, DEFAULT_COST) {
            Ok(h) => h,
            Err(_) => return HttpResponse::InternalServerError().body("Failed to hash password"),
        };
        update_doc.insert("password", hashed_password);
    }

    if update_doc.is_empty() {
        return HttpResponse::BadRequest().body("No valid fields to update");
    }

    // Perform the update
    let filter = doc! { "_id": object_id };
    let update = doc! { "$set": update_doc };

    match collection.update_one(filter, update, None).await {
        Ok(update_result) => {
            if update_result.matched_count == 1 {
                HttpResponse::Ok().body("User updated successfully")
            } else {
                HttpResponse::NotFound().body("User not found")
            }
        }
        Err(e) => {
            eprintln!("Error updating user: {}", e);
            HttpResponse::InternalServerError().body("Failed to update user")
        }
    }
}

/// Handler to delete a user
pub async fn delete_user(
    data: web::Data<AppState>,
    user_id: web::Path<String>,
) -> impl Responder {
    // Access the MongoDB collection with explicit type annotation
    let collection: mongodb::Collection<bson::Document> = data
        .db_client
        .database(&data.database_name)
        .collection("users");

    // Parse the user ID
    let object_id = match ObjectId::parse_str(&*user_id) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().body("Invalid user ID format"),
    };

    // Perform the deletion
    let filter = doc! { "_id": object_id };

    match collection.delete_one(filter, None).await {
        Ok(delete_result) => {
            if delete_result.deleted_count == 1 {
                HttpResponse::Ok().body("User deleted successfully")
            } else {
                HttpResponse::NotFound().body("User not found")
            }
        }
        Err(e) => {
            eprintln!("Error deleting user: {}", e);
            HttpResponse::InternalServerError().body("Failed to delete user")
        }
    }
}
