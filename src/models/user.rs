// src/models/user.rs
//
// NOTE: In V1 we are NOT implementing authentication.
// The users table exists in the database schema to support the eventual order → user relationship.
// This model is a placeholder — auth (JWT, sessions, password hashing) will be implemented
// when we have a reason to measure its impact on performance.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a user row from the `users` table.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    /// NEVER serialize password_hash to JSON responses.
    /// The #[serde(skip_serializing)] attribute ensures this field is never sent to clients.
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterUserDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponseDto {
    pub token: String,
    pub user: UserResponseDto,
}

#[derive(Debug, Serialize)]
pub struct UserResponseDto {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponseDto {
    fn from(user: User) -> Self {
        UserResponseDto {
            id: user.id,
            email: user.email,
            created_at: user.created_at,
        }
    }
}

/// JWT Token Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // User ID (uuid)
    pub email: String,
    pub exp: usize,  // Expiration timestamp
}
