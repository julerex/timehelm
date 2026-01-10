//! Twitter/X OAuth 2.0 authentication module.
//!
//! **Note:** Currently commented out in main.rs as users/sessions tables are not in use.
//! This module provides OAuth authentication via Twitter/X.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthorizationCode, ClientId, ClientSecret,
    RedirectUrl, TokenResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

/// Authentication state containing OAuth client and database connection.
#[derive(Clone)]
pub struct AuthState {
    /// Twitter/X OAuth 2.0 client
    oauth_client: BasicClient,
    /// PostgreSQL connection pool
    db: PgPool,
}

/// User information structure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier (UUID string)
    pub id: String,
    /// Twitter/X username
    pub username: String,
    /// Display name
    pub display_name: String,
    /// Optional avatar/profile image URL
    pub avatar_url: Option<String>,
}

#[derive(sqlx::FromRow)]
struct DbUser {
    id: Uuid,
    twitter_id: String,
    username: String,
    display_name: String,
    avatar_url: Option<String>,
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    error: Option<String>,
}

impl AuthState {
    pub fn new(db: PgPool) -> anyhow::Result<Self> {
        let client_id = ClientId::new(
            std::env::var("TWITTER_CLIENT_ID")
                .map_err(|_| anyhow::anyhow!("TWITTER_CLIENT_ID not set"))?,
        );
        let client_secret = ClientSecret::new(
            std::env::var("TWITTER_CLIENT_SECRET")
                .map_err(|_| anyhow::anyhow!("TWITTER_CLIENT_SECRET not set"))?,
        );

        let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
        let redirect_url = format!("{}/auth/twitter/callback", base_url);

        let oauth_client = BasicClient::new(
            client_id,
            Some(client_secret),
            oauth2::AuthUrl::new("https://twitter.com/i/oauth2/authorize".to_string())?,
            Some(oauth2::TokenUrl::new(
                "https://api.twitter.com/2/oauth2/token".to_string(),
            )?),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_url)?);

        Ok(Self {
            oauth_client,
            db,
        })
    }

    pub async fn get_user(&self, session_id: &str) -> anyhow::Result<Option<User>> {
        let session_uuid = Uuid::parse_str(session_id)?;
        
        let result = sqlx::query_as!(
            DbUser,
            r#"
            SELECT u.id, u.twitter_id, u.username, u.display_name, u.avatar_url
            FROM users u
            INNER JOIN sessions s ON u.id = s.user_id
            WHERE s.id = $1 AND s.expires_at > NOW()
            "#,
            session_uuid
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(result.map(|u| User {
            id: u.id.to_string(),
            username: u.username,
            display_name: u.display_name,
            avatar_url: u.avatar_url,
        }))
    }

    pub async fn create_or_get_user(
        &self,
        twitter_id: &str,
        username: &str,
        display_name: &str,
        avatar_url: Option<&str>,
    ) -> anyhow::Result<Uuid> {
        // Try to get existing user
        let user_result = sqlx::query_as!(
            DbUser,
            r#"
            SELECT id, twitter_id, username, display_name, avatar_url
            FROM users
            WHERE twitter_id = $1
            "#,
            twitter_id
        )
        .fetch_optional(&self.db)
        .await?;

        let user_id = if let Some(user) = user_result {
            // Update user info if changed
            sqlx::query!(
                r#"
                UPDATE users
                SET username = $2, display_name = $3, avatar_url = $4
                WHERE id = $1
                "#,
                user.id,
                username,
                display_name,
                avatar_url
            )
            .execute(&self.db)
            .await?;
            user.id
        } else {
            // Create new user
            let new_id = Uuid::new_v4();
            sqlx::query!(
                r#"
                INSERT INTO users (id, twitter_id, username, display_name, avatar_url)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                new_id,
                twitter_id,
                username,
                display_name,
                avatar_url
            )
            .execute(&self.db)
            .await?;
            new_id
        };

        Ok(user_id)
    }

    pub async fn create_session(&self, user_id: Uuid) -> anyhow::Result<Uuid> {
        let session_id = Uuid::new_v4();
        let expires_at = chrono::Utc::now() + chrono::Duration::days(30);

        sqlx::query!(
            r#"
            INSERT INTO sessions (id, user_id, expires_at)
            VALUES ($1, $2, $3)
            "#,
            session_id,
            user_id,
            expires_at
        )
        .execute(&self.db)
        .await?;

        Ok(session_id)
    }

    pub async fn cleanup_expired_sessions(&self) -> anyhow::Result<()> {
        sqlx::query!("DELETE FROM sessions WHERE expires_at < NOW()")
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

use crate::AppState;

pub async fn twitter_login(State(state): State<AppState>) -> impl IntoResponse {
    let (auth_url, _csrf_token) = state
        .auth
        .oauth_client
        .authorize_url(oauth2::CsrfToken::new_random)
        .set_scopes(vec![
            oauth2::Scope::new("tweet.read".to_string()),
            oauth2::Scope::new("users.read".to_string()),
        ])
        .url();

    Redirect::to(auth_url.as_str())
}

pub async fn twitter_callback(
    Query(query): Query<CallbackQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Some(error) = query.error {
        tracing::error!("OAuth error: {}", error);
        return (StatusCode::BAD_REQUEST, format!("OAuth error: {}", error)).into_response();
    }

    let code = match query.code {
        Some(code) => AuthorizationCode::new(code),
        None => {
            return (StatusCode::BAD_REQUEST, "Missing authorization code").into_response();
        }
    };

    let token_result = state
        .auth
        .oauth_client
        .exchange_code(code)
        .request_async(async_http_client)
        .await;

    let token = match token_result {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Token exchange error: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to exchange token")
                .into_response();
        }
    };

    // Get user info from Twitter API
    let client = reqwest::Client::new();
    let user_response = client
        .get("https://api.twitter.com/2/users/me")
        .bearer_auth(token.access_token().secret())
        .query(&[("user.fields", "profile_image_url,username,name")])
        .send()
        .await;

    let (twitter_id, username, display_name, avatar_url) = match user_response {
        Ok(resp) => {
            let data: serde_json::Value = resp.json().await.unwrap_or_default();
            let user_data = data.get("data").and_then(|d| d.as_object());
            
            if let Some(user_obj) = user_data {
                (
                    user_obj
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    user_obj
                        .get("username")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    user_obj
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    user_obj
                        .get("profile_image_url")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                )
            } else {
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get user info")
                    .into_response();
            }
        }
        Err(e) => {
            tracing::error!("Failed to get user info: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get user info")
                .into_response();
        }
    };

    // Create or get user in database
    let user_id = match state.auth.create_or_get_user(
        &twitter_id,
        &username,
        &display_name,
        avatar_url.as_deref(),
    ).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to create/get user: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user")
                .into_response();
        }
    };

    // Create session
    let session_id = match state.auth.create_session(user_id).await {
        Ok(id) => id.to_string(),
        Err(e) => {
            tracing::error!("Failed to create session: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create session")
                .into_response();
        }
    };

    // Redirect to game with session cookie
    // In production, use proper HTTP-only cookies
    let redirect_url = format!("/?session={}", session_id);
    Redirect::to(&redirect_url).into_response()
}

pub async fn get_current_user(
    axum::extract::Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let session_id = match params.get("session") {
        Some(id) => id,
        None => {
            return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "No session"})))
                .into_response();
        }
    };

    match state.auth.get_user(session_id).await {
        Ok(Some(user)) => (StatusCode::OK, Json(user)).into_response(),
        Ok(None) => {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid session"})),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get user: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
            )
                .into_response()
        }
    }
}

