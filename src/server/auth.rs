use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

const CACHE_TTL_SECS: u64 = 300; // 5 minutes

struct CacheEntry {
    github_user: String,
    validated_at: Instant,
}

pub struct TokenCache {
    entries: Mutex<HashMap<String, CacheEntry>>,
}

impl TokenCache {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Validate a GitHub access token, returning the username.
    /// Caches results for 5 minutes to avoid hammering GitHub API.
    pub async fn validate(&self, token: &str) -> Result<String, AuthError> {
        // Check cache first
        {
            let cache = self.entries.lock().unwrap();
            if let Some(entry) = cache.get(token) {
                if entry.validated_at.elapsed().as_secs() < CACHE_TTL_SECS {
                    return Ok(entry.github_user.clone());
                }
            }
        }

        // Call GitHub API
        let client = reqwest::Client::new();
        let resp = client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "learnlocal-server")
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| AuthError::GitHubApiError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(AuthError::Invalid);
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AuthError::GitHubApiError(e.to_string()))?;

        let login = body["login"]
            .as_str()
            .ok_or(AuthError::GitHubApiError(
                "No login field in response".to_string(),
            ))?
            .to_string();

        // Cache the result
        {
            let mut cache = self.entries.lock().unwrap();
            cache.insert(
                token.to_string(),
                CacheEntry {
                    github_user: login.clone(),
                    validated_at: Instant::now(),
                },
            );
        }

        Ok(login)
    }
}

#[derive(Debug)]
pub enum AuthError {
    Invalid,
    GitHubApiError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::Invalid => write!(f, "Invalid or expired token"),
            AuthError::GitHubApiError(e) => write!(f, "GitHub API error: {}", e),
        }
    }
}

/// Extract and validate the Bearer token from request headers.
pub async fn require_auth(
    state: &super::run::ServerState,
    headers: &axum::http::HeaderMap,
) -> Result<String, (axum::http::StatusCode, axum::Json<super::api::ApiError>)> {
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                axum::http::StatusCode::UNAUTHORIZED,
                axum::Json(super::api::ApiError {
                    error: "Authorization header required. Run 'learnlocal login' first."
                        .to_string(),
                }),
            )
        })?;

    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        (
            axum::http::StatusCode::UNAUTHORIZED,
            axum::Json(super::api::ApiError {
                error: "Invalid authorization format. Expected 'Bearer <token>'".to_string(),
            }),
        )
    })?;

    state.token_cache.validate(token).await.map_err(|e| {
        (
            axum::http::StatusCode::UNAUTHORIZED,
            axum::Json(super::api::ApiError {
                error: e.to_string(),
            }),
        )
    })
}
