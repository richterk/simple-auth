use actix_web::web;

mod auth;
mod user;

/// Initializes the routes for handlers
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/login", web::post().to(auth::login)),
    )
        .service(
            web::scope("/users")
                .route("/create", web::post().to(user::create_user))
                .route("/{id}", web::get().to(user::get_user))
                .route("/{id}/update", web::put().to(user::update_user))
                .route("/{id}/delete", web::delete().to(user::delete_user)),
        );
}