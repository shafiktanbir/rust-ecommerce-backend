// src/services/auth_service.rs

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sqlx::PgPool;

use crate::{
    errors::{AppError, AppResult},
    models::user::{AuthResponseDto, Claims, LoginUserDto, RegisterUserDto, UserResponseDto},
    repositories::user_repository,
};

pub async fn register(
    pool: &PgPool,
    jwt_secret: &str,
    dto: RegisterUserDto,
) -> AppResult<AuthResponseDto> {
    if dto.email.trim().is_empty() || !dto.email.contains('@') {
        return Err(AppError::Validation("Invalid email address".into()));
    }
    if dto.password.len() < 6 {
        return Err(AppError::Validation("Password must be at least 6 characters".into()));
    }

    if user_repository::find_by_email(pool, &dto.email).await?.is_some() {
        return Err(AppError::Conflict("User with this email already exists".into()));
    }

    let password_hash = hash(dto.password, DEFAULT_COST)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Password hashing failed: {}", e)))?;

    let user = user_repository::create_user(pool, &dto.email, &password_hash).await?;

    let token = generate_token(&user.id.to_string(), &user.email, jwt_secret)?;

    Ok(AuthResponseDto {
        token,
        user: UserResponseDto::from(user),
    })
}

pub async fn login(
    pool: &PgPool,
    jwt_secret: &str,
    dto: LoginUserDto,
) -> AppResult<AuthResponseDto> {
    let user = user_repository::find_by_email(pool, &dto.email)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".into()))?;

    let is_valid = verify(&dto.password, &user.password_hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Password verification failed: {}", e)))?;

    if !is_valid {
        return Err(AppError::Unauthorized("Invalid email or password".into()));
    }

    let token = generate_token(&user.id.to_string(), &user.email, jwt_secret)?;

    Ok(AuthResponseDto {
        token,
        user: UserResponseDto::from(user),
    })
}

pub fn generate_token(user_id: &str, email: &str, jwt_secret: &str) -> AppResult<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("Token generation error: {}", e)))
}

pub fn verify_token(token: &str, jwt_secret: &str) -> AppResult<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))?;

    Ok(token_data.claims)
}
