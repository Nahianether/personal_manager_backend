use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::Json,
};
use serde_json::json;
use crate::utils::jwt::verify_jwt;

pub struct AuthUser {
    pub user_id: String,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Get Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|header| header.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "Missing Authorization header"
                    })),
                )
            })?;

        // Check if it starts with "Bearer "
        if !auth_header.starts_with("Bearer ") {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Invalid Authorization header format. Expected: Bearer <token>"
                })),
            ));
        }

        // Extract token
        let token = &auth_header[7..]; // Remove "Bearer " prefix

        // Verify JWT token
        let claims = verify_jwt(token).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Invalid or expired token"
                })),
            )
        })?;

        Ok(AuthUser {
            user_id: claims.sub,
        })
    }
}