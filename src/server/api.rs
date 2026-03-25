use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::sync::Arc;

use super::auth::require_auth;
use super::db::{CourseRow, NewCourse};
use super::run::ServerState;

#[derive(serde::Serialize)]
pub struct ApiError {
    pub error: String,
}

fn api_err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ApiError>) {
    (status, Json(ApiError { error: msg.into() }))
}

pub fn routes() -> Router<Arc<ServerState>> {
    Router::new()
        // Public
        .route("/courses", get(list_courses))
        .route("/courses/{id}", get(get_course))
        .route("/packages/{filename}", get(serve_package))
        // Auth
        .route("/auth/device", post(auth_device_start))
        .route("/auth/device/poll", post(auth_device_poll))
        .route("/auth/me", get(auth_me))
        // Ratings & Reviews
        .route("/courses/{id}/ratings", post(submit_rating))
        .route("/courses/{id}/reviews", post(submit_review))
        // Publish
        .route("/publish", post(publish_course))
}

// --- Public endpoints ---

/// GET /api/v1/courses — list approved courses with ratings.
/// Returns same JSON shape as the Registry type for client compatibility.
async fn list_courses(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking({
        let state = state.clone();
        move || state.db.list_courses(false)
    })
    .await;

    match result {
        Ok(Ok(courses)) => {
            let entries: Vec<serde_json::Value> =
                courses.iter().map(|c| course_to_json(c, &state)).collect();
            let registry = serde_json::json!({
                "version": 1,
                "updated_at": chrono::Utc::now().to_rfc3339(),
                "courses": entries,
            });
            Json(registry).into_response()
        }
        Ok(Err(e)) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// GET /api/v1/courses/:id — course detail with reviews.
async fn get_course(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking({
        let state = state.clone();
        let id = id.clone();
        move || {
            let course = state.db.get_course(&id)?;
            let reviews = state.db.get_reviews(&id)?;
            let ratings = state.db.get_ratings(&id)?;
            Ok::<_, anyhow::Error>((course, reviews, ratings))
        }
    })
    .await;

    match result {
        Ok(Ok((Some(course), reviews, ratings))) => {
            let mut json = course_to_json(&course, &state);
            json["reviews"] = serde_json::json!(reviews);
            json["ratings_summary"] = serde_json::json!(ratings);
            Json(json).into_response()
        }
        Ok(Ok((None, _, _))) => {
            api_err(StatusCode::NOT_FOUND, format!("Course '{}' not found", id)).into_response()
        }
        Ok(Err(e)) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// GET /api/v1/packages/:filename — serve a course package file.
async fn serve_package(
    State(state): State<Arc<ServerState>>,
    Path(filename): Path<String>,
) -> impl IntoResponse {
    // Security: reject path traversal
    if filename.contains('/')
        || filename.contains('\\')
        || filename.contains("..")
        || filename.contains('\0')
    {
        return api_err(StatusCode::BAD_REQUEST, "Invalid filename").into_response();
    }
    if !filename.ends_with(".tar.gz") {
        return api_err(StatusCode::BAD_REQUEST, "Only .tar.gz files served").into_response();
    }

    let path = state.packages_dir.join(&filename);
    if !path.exists() {
        return api_err(StatusCode::NOT_FOUND, "Package not found").into_response();
    }

    // Increment download counter (best-effort)
    let course_id = filename
        .rsplit_once('-')
        .map(|(id, _)| id.to_string())
        .unwrap_or_default();
    if !course_id.is_empty() {
        let state2 = state.clone();
        tokio::task::spawn_blocking(move || {
            let _ = state2.db.increment_downloads(&course_id);
        });
    }

    match tokio::fs::read(&path).await {
        Ok(data) => (
            StatusCode::OK,
            [
                ("content-type", "application/gzip"),
                (
                    "content-disposition",
                    &format!("attachment; filename=\"{}\"", filename),
                ),
            ],
            data,
        )
            .into_response(),
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- Auth endpoints ---

#[derive(serde::Deserialize)]
struct DevicePollBody {
    device_code: String,
}

/// POST /api/v1/auth/device — initiate GitHub device flow.
async fn auth_device_start(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let client_id = match &state.github_client_id {
        Some(id) => id.clone(),
        None => {
            return api_err(
                StatusCode::SERVICE_UNAVAILABLE,
                "GitHub OAuth not configured on server",
            )
            .into_response()
        }
    };

    let client = reqwest::Client::new();
    let resp = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", &client_id),
            ("scope", &"read:user".to_string()),
        ])
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => match r.json::<serde_json::Value>().await {
            Ok(data) => Json(data).into_response(),
            Err(e) => api_err(
                StatusCode::BAD_GATEWAY,
                format!("GitHub parse error: {}", e),
            )
            .into_response(),
        },
        Ok(r) => {
            let status = r.status();
            let body = r.text().await.unwrap_or_default();
            api_err(
                StatusCode::BAD_GATEWAY,
                format!("GitHub error {}: {}", status, body),
            )
            .into_response()
        }
        Err(e) => api_err(
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Cannot reach GitHub: {}", e),
        )
        .into_response(),
    }
}

/// POST /api/v1/auth/device/poll — poll for device flow completion.
async fn auth_device_poll(
    State(state): State<Arc<ServerState>>,
    Json(body): Json<DevicePollBody>,
) -> impl IntoResponse {
    let client_id = match &state.github_client_id {
        Some(id) => id.clone(),
        None => {
            return api_err(
                StatusCode::SERVICE_UNAVAILABLE,
                "GitHub OAuth not configured",
            )
            .into_response()
        }
    };

    let client = reqwest::Client::new();
    let resp = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id.as_str()),
            ("device_code", body.device_code.as_str()),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => match r.json::<serde_json::Value>().await {
            Ok(data) => Json(data).into_response(),
            Err(e) => api_err(StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
        },
        Ok(r) => {
            let body = r.text().await.unwrap_or_default();
            api_err(StatusCode::BAD_GATEWAY, body).into_response()
        }
        Err(e) => api_err(StatusCode::SERVICE_UNAVAILABLE, e.to_string()).into_response(),
    }
}

/// GET /api/v1/auth/me — current user info.
async fn auth_me(
    State(state): State<Arc<ServerState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    match require_auth(&state, &headers).await {
        Ok(user) => Json(serde_json::json!({ "github_user": user })).into_response(),
        Err(e) => e.into_response(),
    }
}

// --- Ratings & Reviews ---

#[derive(serde::Deserialize)]
struct RatingBody {
    stars: i32,
}

#[derive(serde::Deserialize)]
struct ReviewBody {
    body: String,
}

/// POST /api/v1/courses/:id/ratings — submit a star rating (1-5).
async fn submit_rating(
    State(state): State<Arc<ServerState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<RatingBody>,
) -> impl IntoResponse {
    let user = match require_auth(&state, &headers).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

    if !(1..=5).contains(&body.stars) {
        return api_err(StatusCode::BAD_REQUEST, "Stars must be between 1 and 5").into_response();
    }

    let result = tokio::task::spawn_blocking({
        let state = state.clone();
        let id = id.clone();
        move || {
            // Verify course exists
            let course = state.db.get_course(&id)?;
            if course.is_none() {
                anyhow::bail!("Course not found");
            }
            state.db.upsert_rating(&id, &user, body.stars)?;
            state.db.get_ratings(&id)
        }
    })
    .await;

    match result {
        Ok(Ok(summary)) => Json(serde_json::json!({
            "status": "ok",
            "ratings": summary,
        }))
        .into_response(),
        Ok(Err(e)) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                api_err(StatusCode::NOT_FOUND, msg).into_response()
            } else {
                api_err(StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
        }
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// POST /api/v1/courses/:id/reviews — submit a text review.
async fn submit_review(
    State(state): State<Arc<ServerState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<ReviewBody>,
) -> impl IntoResponse {
    let user = match require_auth(&state, &headers).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

    let text = body.body.trim().to_string();
    if text.is_empty() || text.len() > 2000 {
        return api_err(
            StatusCode::BAD_REQUEST,
            "Review must be between 1 and 2000 characters",
        )
        .into_response();
    }

    let result = tokio::task::spawn_blocking({
        let state = state.clone();
        let id = id.clone();
        move || {
            let course = state.db.get_course(&id)?;
            if course.is_none() {
                anyhow::bail!("Course not found");
            }
            state.db.insert_review(&id, &user, &text)
        }
    })
    .await;

    match result {
        Ok(Ok(())) => Json(serde_json::json!({ "status": "ok" })).into_response(),
        Ok(Err(e)) => {
            let msg = e.to_string();
            if msg.contains("UNIQUE") {
                api_err(
                    StatusCode::CONFLICT,
                    "You have already reviewed this course",
                )
                .into_response()
            } else if msg.contains("not found") {
                api_err(StatusCode::NOT_FOUND, msg).into_response()
            } else {
                api_err(StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
        }
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- Publish ---

/// POST /api/v1/publish — upload a course package (multipart).
async fn publish_course(
    State(state): State<Arc<ServerState>>,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let github_user = match require_auth(&state, &headers).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

    let mut archive_data: Option<Vec<u8>> = None;
    let mut manifest_json: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "package" => match field.bytes().await {
                Ok(bytes) => {
                    // 50MB limit
                    if bytes.len() > 50 * 1024 * 1024 {
                        return api_err(
                            StatusCode::PAYLOAD_TOO_LARGE,
                            "Package exceeds 50MB limit",
                        )
                        .into_response();
                    }
                    archive_data = Some(bytes.to_vec());
                }
                Err(e) => {
                    return api_err(
                        StatusCode::BAD_REQUEST,
                        format!("Failed to read package: {}", e),
                    )
                    .into_response()
                }
            },
            "manifest" => match field.text().await {
                Ok(text) => manifest_json = Some(text),
                Err(e) => {
                    return api_err(
                        StatusCode::BAD_REQUEST,
                        format!("Failed to read manifest: {}", e),
                    )
                    .into_response()
                }
            },
            _ => {}
        }
    }

    let Some(archive) = archive_data else {
        return api_err(StatusCode::BAD_REQUEST, "Missing 'package' field").into_response();
    };
    let Some(manifest_str) = manifest_json else {
        return api_err(StatusCode::BAD_REQUEST, "Missing 'manifest' field").into_response();
    };

    // Parse manifest
    let manifest: serde_json::Value = match serde_json::from_str(&manifest_str) {
        Ok(v) => v,
        Err(e) => {
            return api_err(
                StatusCode::BAD_REQUEST,
                format!("Invalid manifest JSON: {}", e),
            )
            .into_response()
        }
    };

    let course_id = manifest["course_id"].as_str().unwrap_or("").to_string();
    let version = manifest["version"].as_str().unwrap_or("").to_string();
    if course_id.is_empty() || version.is_empty() {
        return api_err(
            StatusCode::BAD_REQUEST,
            "Manifest must include course_id and version",
        )
        .into_response();
    }

    // Verify checksum
    let declared_checksum = manifest["checksum"].as_str().unwrap_or("").to_string();
    let actual_hash = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&archive);
        format!("sha256:{:x}", hasher.finalize())
    };
    if !declared_checksum.is_empty() && declared_checksum != actual_hash {
        return api_err(
            StatusCode::BAD_REQUEST,
            format!(
                "Checksum mismatch: declared={}, actual={}",
                declared_checksum, actual_hash
            ),
        )
        .into_response();
    }

    let filename = format!("{}-{}.tar.gz", course_id, version);
    let package_path = state.packages_dir.join(&filename);

    // Write archive to disk
    if let Err(e) = tokio::fs::write(&package_path, &archive).await {
        return api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save package: {}", e),
        )
        .into_response();
    }

    // Validate the course (extract to temp, load, validate)
    let validation_result = tokio::task::spawn_blocking({
        let archive = archive.clone();
        move || -> Result<(), String> {
            let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
            let archive_path = tmp.path().join("package.tar.gz");
            std::fs::write(&archive_path, &archive).map_err(|e| e.to_string())?;

            let extract_dir = tmp.path().join("extracted");
            std::fs::create_dir(&extract_dir).map_err(|e| e.to_string())?;
            crate::community::download::extract_tar_gz(&archive_path, &extract_dir)?;

            // Find course root
            let course_root = if extract_dir.join("course.yaml").exists() {
                extract_dir.clone()
            } else {
                let entries: Vec<_> = std::fs::read_dir(&extract_dir)
                    .map_err(|e| e.to_string())?
                    .flatten()
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .collect();
                if entries.len() == 1 && entries[0].path().join("course.yaml").exists() {
                    entries[0].path()
                } else {
                    return Err("No course.yaml found in package".to_string());
                }
            };

            let c = crate::course::load_course(&course_root).map_err(|e| e.to_string())?;
            let result = crate::course::validate_course(&c);
            if !result.all_passed() {
                let failures: Vec<_> = result
                    .checks
                    .iter()
                    .filter(|c| !c.passed)
                    .map(|c| c.message.clone())
                    .collect();
                return Err(format!("Validation failed: {}", failures.join("; ")));
            }
            Ok(())
        }
    })
    .await;

    match validation_result {
        Ok(Err(e)) => {
            // Clean up the saved file
            let _ = tokio::fs::remove_file(&package_path).await;
            return api_err(StatusCode::BAD_REQUEST, e).into_response();
        }
        Err(e) => {
            let _ = tokio::fs::remove_file(&package_path).await;
            return api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
        Ok(Ok(())) => {} // Validation passed
    }

    // Ownership check: if this course ID already exists, only the owner can publish new versions
    let ownership_result = tokio::task::spawn_blocking({
        let state = state.clone();
        let course_id = course_id.clone();
        let github_user = github_user.clone();
        move || -> Result<(), String> {
            if let Some(owner) = state.db.get_owner(&course_id).map_err(|e| e.to_string())? {
                if owner != github_user {
                    return Err(format!(
                        "Course '{}' is owned by '{}'. Only the owner can publish new versions. \
                         To create a derivative, use a different course ID and set forked_from in the manifest.",
                        course_id, owner
                    ));
                }
            }
            Ok(())
        }
    })
    .await;

    match ownership_result {
        Ok(Err(e)) => {
            let _ = tokio::fs::remove_file(&package_path).await;
            return api_err(StatusCode::FORBIDDEN, e).into_response();
        }
        Err(e) => {
            let _ = tokio::fs::remove_file(&package_path).await;
            return api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
        Ok(Ok(())) => {}
    }

    // Insert into database
    let new_course = NewCourse {
        id: course_id.clone(),
        name: manifest["name"].as_str().unwrap_or(&course_id).to_string(),
        version: version.clone(),
        author: manifest["author"]
            .as_str()
            .unwrap_or(&github_user)
            .to_string(),
        author_github: Some(github_user.clone()),
        description: manifest["description"].as_str().unwrap_or("").to_string(),
        language_id: manifest["language_id"].as_str().unwrap_or("").to_string(),
        language_display: manifest["language_display"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        license: manifest["license"].as_str().map(String::from),
        lessons: manifest["lessons"].as_i64().unwrap_or(0),
        exercises: manifest["exercises"].as_i64().unwrap_or(0),
        has_stages: manifest["has_stages"].as_bool().unwrap_or(false),
        platform: manifest["platform"].as_str().map(String::from),
        provision: manifest["provision"]
            .as_str()
            .unwrap_or("system")
            .to_string(),
        tags: manifest["tags"].to_string(),
        estimated_hours: manifest["estimated_hours"].as_f64(),
        checksum: actual_hash,
        min_learnlocal_version: manifest["min_learnlocal_version"]
            .as_str()
            .map(String::from),
        package_filename: filename,
        owner_github: github_user,
        forked_from_id: manifest["forked_from_id"].as_str().map(String::from),
        forked_from_version: manifest["forked_from_version"].as_str().map(String::from),
        forked_from_author: manifest["forked_from_author"].as_str().map(String::from),
    };

    let db_result = tokio::task::spawn_blocking({
        let state = state.clone();
        move || state.db.insert_course(&new_course)
    })
    .await;

    match db_result {
        Ok(Ok(())) => Json(serde_json::json!({
            "status": "pending",
            "course_id": course_id,
            "message": "Course uploaded and queued for review.",
        }))
        .into_response(),
        Ok(Err(e)) => {
            let msg = e.to_string();
            if msg.contains("UNIQUE") || msg.contains("PRIMARY KEY") {
                api_err(
                    StatusCode::CONFLICT,
                    format!(
                        "Course '{}' already exists. Update the version to republish.",
                        course_id
                    ),
                )
                .into_response()
            } else {
                api_err(StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
        }
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- Helpers ---

fn course_to_json(c: &CourseRow, _state: &ServerState) -> serde_json::Value {
    let download_url = format!(
        "https://learnlocal.aiquest.info/api/v1/packages/{}",
        c.package_filename
    );
    let tags: Vec<String> = serde_json::from_str(&c.tags).unwrap_or_default();

    serde_json::json!({
        "id": c.id,
        "name": c.name,
        "version": c.version,
        "author": c.author,
        "author_github": c.author_github,
        "description": c.description,
        "language_id": c.language_id,
        "language_display": c.language_display,
        "license": c.license,
        "lessons": c.lessons,
        "exercises": c.exercises,
        "has_stages": c.has_stages,
        "platform": c.platform,
        "provision": c.provision,
        "tags": tags,
        "estimated_hours": c.estimated_hours,
        "download_url": download_url,
        "checksum": c.checksum,
        "published_at": c.published_at,
        "min_learnlocal_version": c.min_learnlocal_version,
        "avg_rating": c.avg_rating,
        "review_count": c.review_count,
        "downloads": c.downloads,
        "owner_github": c.owner_github,
        "forked_from": if c.forked_from_id.is_some() {
            serde_json::json!({
                "id": c.forked_from_id,
                "version": c.forked_from_version,
                "author": c.forked_from_author,
            })
        } else {
            serde_json::Value::Null
        },
    })
}
