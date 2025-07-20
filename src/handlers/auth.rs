use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use bcrypt::{hash, verify, DEFAULT_COST};
use anyhow::Result;

use crate::models::{User, CreateUserRequest, LoginRequest, AuthResponse, UserResponse};

#[derive(Debug, Deserialize)]
pub struct SigninRequest {
    pub name: Option<String>,
    pub email: String,
    pub password: String,
}

impl SigninRequest {
    pub fn to_create_user_request(self) -> CreateUserRequest {
        CreateUserRequest {
            name: self.name.unwrap_or_else(|| "User".to_string()),
            email: self.email,
            password: self.password,
        }
    }
    
    pub fn to_login_request(self) -> LoginRequest {
        LoginRequest {
            email: self.email,
            password: self.password,
        }
    }
}
use crate::services::database::DbPool;
use crate::utils::jwt::create_jwt;

pub async fn signup(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user already exists
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = ?",
    )
    .bind(&payload.email)
    .fetch_optional(&pool)
    .await;

    match existing_user {
        Ok(Some(_)) => {
            return Err((
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "User with this email already exists"
                })),
            ));
        }
        Ok(None) => {}
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Database error"
                })),
            ));
        }
    }

    // Hash password
    let password_hash = match hash(&payload.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to hash password"
                })),
            ));
        }
    };

    // Create new user
    let user = User::new(payload.name, payload.email, password_hash);

    // Insert user into database
    let result = sqlx::query(
        "INSERT INTO users (id, name, email, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&user.id)
    .bind(&user.name)
    .bind(&user.email)
    .bind(&user.password_hash)
    .bind(&user.created_at)
    .bind(&user.updated_at)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            // Generate JWT token
            let token = match create_jwt(&user.id) {
                Ok(token) => token,
                Err(_) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "Failed to create token"
                        })),
                    ));
                }
            };

            let response = AuthResponse {
                token,
                user: UserResponse::from(user),
            };

            Ok(Json(json!(response)))
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to create user"
            })),
        )),
    }
}

pub async fn login(
    State(pool): State<DbPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Find user by email
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = ?",
    )
    .bind(&payload.email)
    .fetch_optional(&pool)
    .await;

    let user = match user {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Invalid email or password"
                })),
            ));
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Database error"
                })),
            ));
        }
    };

    // Verify password
    let is_valid = match verify(&payload.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to verify password"
                })),
            ));
        }
    };

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Invalid email or password"
            })),
        ));
    }

    // Generate JWT token
    let token = match create_jwt(&user.id) {
        Ok(token) => token,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to create token"
                })),
            ));
        }
    };

    let response = AuthResponse {
        token,
        user: UserResponse::from(user),
    };

    Ok(Json(json!(response)))
}

pub async fn signin(
    State(pool): State<DbPool>,
    Json(payload): Json<SigninRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let email = payload.email.trim().to_lowercase();
    
    // First try to find existing user
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = ?",
    )
    .bind(&email)
    .fetch_optional(&pool)
    .await;

    match existing_user {
        Ok(Some(user)) => {
            // User exists, try to login
            let is_valid = match verify(&payload.password, &user.password_hash) {
                Ok(valid) => valid,
                Err(_) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "Failed to verify password"
                        })),
                    ));
                }
            };

            if !is_valid {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "Invalid email or password"
                    })),
                ));
            }

            // Generate JWT token
            let token = match create_jwt(&user.id) {
                Ok(token) => token,
                Err(_) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "Failed to create token"
                        })),
                    ));
                }
            };

            let response = AuthResponse {
                token,
                user: UserResponse::from(user),
            };

            Ok(Json(json!(response)))
        }
        Ok(None) => {
            // User doesn't exist, create new account
            if payload.name.is_none() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Name is required for new user registration"
                    })),
                ));
            }

            // Hash password
            let password_hash = match hash(&payload.password, DEFAULT_COST) {
                Ok(hash) => hash,
                Err(_) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "Failed to hash password"
                        })),
                    ));
                }
            };

            // Create new user
            let user = User::new(
                payload.name.unwrap(),
                email,
                password_hash,
            );

            // Insert user into database
            let result = sqlx::query(
                "INSERT INTO users (id, name, email, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&user.id)
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.password_hash)
            .bind(&user.created_at)
            .bind(&user.updated_at)
            .execute(&pool)
            .await;

            match result {
                Ok(_) => {
                    // Generate JWT token
                    let token = match create_jwt(&user.id) {
                        Ok(token) => token,
                        Err(_) => {
                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({
                                    "error": "Failed to create token"
                                })),
                            ));
                        }
                    };

                    let response = AuthResponse {
                        token,
                        user: UserResponse::from(user),
                    };

                    Ok(Json(json!(response)))
                }
                Err(_) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Failed to create user"
                    })),
                )),
            }
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Database error"
            })),
        )),
    }
}