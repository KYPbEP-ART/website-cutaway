use axum::{
    extract::ConnectInfo,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::net::SocketAddr;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::rngs::OsRng; // 👈 Добавлено для генерации соли
use jsonwebtoken::{encode, Header, EncodingKey};
use serde_json::json;
use chrono::Utc;
use uuid::Uuid; // 👈 Добавлено для генерации ID

use crate::models::user::{LoginRequest, RegisterRequest, AuthResponse, User, UserPublic};
use crate::rate_limiter; // 👈 Наш модуль защиты от брутфорса

// ==========================================
// РЕГИСТРАЦИЯ
// ==========================================
pub async fn register(Json(payload): Json<RegisterRequest>) -> impl IntoResponse {
    // 1. ВАЛИДАЦИЯ
    if payload.login.len() < 3 {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Логин должен быть не менее 3 символов"}))).into_response();
    }
    if payload.login.len() > 30 {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Логин не может быть длиннее 30 символов"}))).into_response();
    }
    if !payload.email.contains('@') || !payload.email.contains('.') {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Некорректный email"}))).into_response();
    }
    if payload.password.len() < 6 {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Пароль должен быть не менее 6 символов"}))).into_response();
    }

    let conn = crate::db::sqlite::DB_POOL.get().unwrap();

    // 2. Проверка на существование
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM users WHERE login = ?1 OR email = ?2",
            [&payload.login, &payload.email],
            |row| row.get(0),
        )
        .unwrap_or(0) > 0;

    if exists {
        return (StatusCode::CONFLICT, Json(json!({"error": "Пользователь уже существует"}))).into_response();
    }

    // 3. Хеширование пароля
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    // 4. Создание пользователя
    let user_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    // 👇 ВОТ ЭТОГО БЛОКА НЕ ХВАТАЛО (он использует password_hash и created_at) 👇
    conn.execute(
        "INSERT INTO users (id, login, email, password, avatar, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![user_id, payload.login, payload.email, password_hash, None::<String>, created_at],
    ).unwrap();

    // 5. Получаем созданного пользователя обратно
    let user = conn.query_row(
        "SELECT id, login, email, password, avatar, created_at FROM users WHERE id = ?1",
        [&user_id],
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
    ).unwrap();

    // 6. Генерация токена и ответ
    let token = generate_token(&user.id).unwrap();
    let response = AuthResponse {
        token,
        user: UserPublic::from(user),
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

// ==========================================
// ВХОД (с Rate Limiting)
// ==========================================
pub async fn login(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let ip = addr.ip();

    // 1. ПРОВЕРКА RATE LIMIT
    if let Err(msg) = rate_limiter::check_rate_limit(ip) {
        return (StatusCode::TOO_MANY_REQUESTS, Json(json!({"error": msg}))).into_response();
    }

    

    let conn = crate::db::sqlite::DB_POOL.get().unwrap();

    // 2. Поиск пользователя
    let user = conn.query_row(
        "SELECT id, login, email, password, avatar, created_at FROM users WHERE login = ?1",
        [&payload.login],
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
        Err(_) => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Неверный логин или пароль"}))).into_response(),
    };

    // 3. Проверка пароля
    let parsed_hash = match PasswordHash::new(&user.password) {
        Ok(hash) => hash,
        Err(_) => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Неверный логин или пароль"}))).into_response(),
    };

    if Argon2::default().verify_password(payload.password.as_bytes(), &parsed_hash).is_err() {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Неверный логин или пароль"}))).into_response();
    }

    // 4. Успешный вход -> СБРАСЫВАЕМ счётчик попыток для этого IP!
    rate_limiter::reset_rate_limit(ip);

    // 5. Генерация токена
    let token = generate_token(&user.id).unwrap();

    let response = AuthResponse {
        token,
        user: UserPublic::from(user),
    };

    (StatusCode::OK, Json(response)).into_response()
}

// ==========================================
// ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ
// ==========================================
fn generate_token(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    // 👇 ИСПРАВЛЕНО: теперь берет ключ из переменных окружения (NixOS), а не из кода!
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in environment");
    
    let claims = json!({
        "sub": user_id,
        "exp": (Utc::now() + chrono::Duration::days(7)).timestamp()
    });

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}