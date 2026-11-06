use axum::{
    routing::{get, post},
    Router,
    http::Method, // 👈 БЕРЕМ Method ОТСЮДА, а не из отдельного crate http
};
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;

mod routes;
mod models;
mod db;
mod rate_limiter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    db::sqlite::init_db().expect("Failed to initialize database");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/register", post(routes::auth::register))
        .route("/api/login", post(routes::auth::login))
        .route("/api/me", get(routes::users::get_current_user))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    tracing::info!("🚀 Сервер запущен на http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    // применяем обёртку для получения IP ко всему приложению
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}