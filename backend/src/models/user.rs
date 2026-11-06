use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub login: String,
    pub email: String,
    #[serde(skip_serializing)] // Не отправляем пароль в ответах
    pub password: String,
    pub avatar: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub login: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserPublic,
}

#[derive(Debug, Serialize, Clone)]
pub struct UserPublic {
    pub id: String,
    pub login: String,
    pub email: String,
    pub avatar: Option<String>,
    pub created_at: String,
}

impl From<User> for UserPublic {
    fn from(user: User) -> Self {
        UserPublic {
            id: user.id,
            login: user.login,
            email: user.email,
            avatar: user.avatar,
            created_at: user.created_at,
        }
    }
}