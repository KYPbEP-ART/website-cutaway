use axum::{
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_json::json;

use crate::db::sqlite::DB_POOL; // 👈 Импортируем пул, а не get_db
use crate::models::user::{User, UserPublic};

pub async fn get_current_user(headers: HeaderMap) -> impl IntoResponse {
    // Получаем токен из заголовка Authorization
    let auth_header = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Токен не предоставлен"}))).into_response(),
    };

    // Проверяем токен
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in environment");
    let token_data = match decode::<serde_json::Value>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data,
        Err(_) => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Невалидный токен"}))).into_response(),
    };

    let user_id = token_data.claims["sub"].as_str().unwrap();

    // 👇 БЕРЕМ СОЕДИНЕНИЕ ИЗ ПУЛА
    let conn = DB_POOL.get().unwrap();
    
    let user = conn.query_row(
        "SELECT id, login, email, password, avatar, created_at FROM users WHERE id = ?1",
        [user_id],
        |row| {
            Ok(User {
                id: row.get(0)?,
                login: row.get(1)?,
                email: row.get(2)?,
                password: row.get(3)?,
                avatar: row.get(4)?,
                created_at: row.get(5)?,
            })
        },
    );

    let user = match user {
        Ok(u) => u,
        Err(_) => return (StatusCode::NOT_FOUND, Json(json!({"error": "Пользователь не найден"}))).into_response(),
    };

    (StatusCode::OK, Json(UserPublic::from(user))).into_response()
}