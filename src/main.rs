mod config;
mod tailscale;
mod routes {
    pub mod index;
    pub mod manage;
}
mod models {
    pub mod service;
    pub mod tailscale;
}

use axum::{
    Router,
    routing::{get, post},
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(routes::index::serve))
        .route("/manage", get(routes::manage::serve))
        .route("/add-service", post(routes::manage::add_service))
        .route("/remove-service", post(routes::manage::remove_service));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3003").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
