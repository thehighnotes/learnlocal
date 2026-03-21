use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::server::AppState;
use super::workspace;
use super::yaml_rw;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // Workspace (project management)
        .route("/workspace/status", get(workspace_status))
        .route("/workspace/projects", get(list_projects))
        .route("/workspace/create", post(create_project))
        .route("/workspace/open", post(open_project))
        .route("/workspace/meta", get(get_meta))
        .route("/workspace/meta", put(update_meta))
        .route("/workspace/history", get(get_history))
        // Course
        .route("/course", get(get_course))
        .route("/course", put(update_course))
        // Lessons
        .route("/lessons", get(list_lessons))
        .route("/lessons", post(create_lesson))
        .route("/lessons/:lid", delete(delete_lesson))
        .route("/lessons/reorder", put(reorder_lessons))
        // Exercises
        .route("/lessons/:lid/exercises", get(list_exercises))
        .route("/lessons/:lid/exercises", post(create_exercise))
        .route("/lessons/:lid/exercises/:eid", get(get_exercise))
        .route("/lessons/:lid/exercises/:eid", put(update_exercise))
        .route("/lessons/:lid/exercises/:eid", delete(delete_exercise))
        .route("/lessons/:lid/exercises/reorder", put(reorder_exercises))
        // Execution
        .route("/run-solution", post(run_solution))
        .route("/validate", post(validate_course))
        .route("/validate/:lid/:eid", post(validate_exercise))
        .route("/toolcheck", get(toolcheck))
        // AI chat proxy
        .route("/ai/chat", post(ai_chat))
}

// --- Helpers ---

fn course_path(state: &AppState) -> Result<std::path::PathBuf, (StatusCode, Json<ApiError>)> {
    state.course_path.read().unwrap().clone().ok_or_else(|| {
        api_err(
            StatusCode::BAD_REQUEST,
            "No course loaded. Open or create a project first.",
        )
    })
}

fn record_action(state: &AppState, action: &str, details: Option<&str>) {
    if let Ok(path) = course_path(state) {
        if let Some(mut meta) = workspace::load_meta(&path) {
            meta.record(&state.author_name, action, details);
            let _ = workspace::save_meta(&path, &meta);
        }
    }
}

#[derive(Serialize)]
struct ApiError {
    error: String,
}

fn api_err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ApiError>) {
    (status, Json(ApiError { error: msg.into() }))
}

// --- Workspace ---

#[derive(Serialize)]
struct WorkspaceStatus {
    has_course: bool,
    course_path: Option<String>,
    author_name: String,
}

async fn workspace_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let cp = state.course_path.read().unwrap();
    Json(WorkspaceStatus {
        has_course: cp.is_some(),
        course_path: cp.as_ref().map(|p| p.to_string_lossy().to_string()),
        author_name: state.author_name.clone(),
    })
}

async fn list_projects(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let mut all = Vec::new();

        // Workspace concepts
        if let Ok(ws) = workspace::list_workspace_projects() {
            all.extend(ws);
        }

        // Also scan the courses/ directory relative to the executable
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                for dir in [
                    exe_dir.join("courses"),
                    exe_dir.parent().unwrap_or(exe_dir).join("courses"),
                ] {
                    if dir.exists() {
                        if let Ok(courses) = workspace::list_directory_projects(&dir) {
                            for c in courses {
                                if !all
                                    .iter()
                                    .any(|p: &workspace::ProjectSummary| p.path == c.path)
                                {
                                    all.push(c);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok::<_, anyhow::Error>(all)
    })
    .await;

    match result {
        Ok(Ok(projects)) => Json(projects).into_response(),
        Ok(Err(e)) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
struct CreateProjectBody {
    name: String,
    language_id: String,
    language_display: String,
    extension: String,
}

async fn create_project(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateProjectBody>,
) -> impl IntoResponse {
    let author = state.author_name.clone();
    let state_clone = Arc::clone(&state);

    let result = tokio::task::spawn_blocking(move || {
        let path = workspace::create_concept(
            &body.name,
            &body.language_id,
            &body.language_display,
            &body.extension,
            &author,
        )?;
        // Set as current project
        *state_clone.course_path.write().unwrap() = Some(path.clone());
        Ok::<_, anyhow::Error>(path.to_string_lossy().to_string())
    })
    .await;

    match result {
        Ok(Ok(path)) => Json(serde_json::json!({ "path": path })).into_response(),
        Ok(Err(e)) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
struct OpenProjectBody {
    path: String,
}

async fn open_project(
    State(state): State<Arc<AppState>>,
    Json(body): Json<OpenProjectBody>,
) -> impl IntoResponse {
    let p = std::path::PathBuf::from(&body.path);
    if !p.join("course.yaml").exists() {
        return api_err(StatusCode::BAD_REQUEST, "Not a valid course directory").into_response();
    }

    // Ensure studio metadata exists
    let author = state.author_name.clone();
    if workspace::load_meta(&p).is_none() {
        let meta = workspace::StudioMeta::new(&author);
        let _ = workspace::save_meta(&p, &meta);
    } else {
        // Record the open action
        if let Some(mut meta) = workspace::load_meta(&p) {
            meta.record(&author, "opened", Some("Opened in LearnLocal Studio"));
            let _ = workspace::save_meta(&p, &meta);
        }
    }

    *state.course_path.write().unwrap() = Some(p);
    Json(serde_json::json!({ "ok": true })).into_response()
}

async fn get_meta(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match workspace::load_meta(&cp) {
        Some(meta) => Json(meta).into_response(),
        None => Json(serde_json::json!(null)).into_response(),
    }
}

#[derive(Deserialize)]
struct UpdateMetaBody {
    status: Option<String>,
}

async fn update_meta(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateMetaBody>,
) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    let mut meta =
        workspace::load_meta(&cp).unwrap_or_else(|| workspace::StudioMeta::new(&state.author_name));

    if let Some(ref status) = body.status {
        meta.status = match status.as_str() {
            "concept" => workspace::ProjectStatus::Concept,
            "draft" => workspace::ProjectStatus::Draft,
            "review" => workspace::ProjectStatus::Review,
            "published" => workspace::ProjectStatus::Published,
            _ => meta.status,
        };
        meta.record(
            &state.author_name,
            "status_change",
            Some(&format!("Status changed to {}", status)),
        );
    }

    match workspace::save_meta(&cp, &meta) {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_history(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match workspace::load_meta(&cp) {
        Some(meta) => Json(serde_json::json!({
            "authors": meta.authors,
            "history": meta.history,
        }))
        .into_response(),
        None => Json(serde_json::json!({ "authors": [], "history": [] })).into_response(),
    }
}

// --- Course ---

async fn get_course(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match yaml_rw::read_course_meta(&cp) {
        Ok(meta) => Json(meta).into_response(),
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
struct UpdateCourseBody {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    author: Option<String>,
    license: Option<String>,
}

async fn update_course(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateCourseBody>,
) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match yaml_rw::update_course_meta(
        &cp,
        body.name.as_deref(),
        body.version.as_deref(),
        body.description.as_deref(),
        body.author.as_deref(),
        body.license.as_deref(),
    ) {
        Ok(()) => {
            record_action(&state, "update_course", Some("Updated course metadata"));
            StatusCode::OK.into_response()
        }
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- Lessons ---

async fn list_lessons(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match yaml_rw::list_lessons(&cp) {
        Ok(lessons) => Json(lessons).into_response(),
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
struct CreateLessonBody {
    id: String,
    title: String,
}

async fn create_lesson(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateLessonBody>,
) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match yaml_rw::create_lesson(&cp, &body.id, &body.title) {
        Ok(()) => {
            record_action(
                &state,
                "create_lesson",
                Some(&format!("Created lesson '{}'", body.id)),
            );
            StatusCode::CREATED.into_response()
        }
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn delete_lesson(
    State(state): State<Arc<AppState>>,
    AxumPath(lid): AxumPath<String>,
) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match yaml_rw::delete_lesson(&cp, &lid) {
        Ok(()) => {
            record_action(
                &state,
                "delete_lesson",
                Some(&format!("Deleted lesson '{}'", lid)),
            );
            StatusCode::OK.into_response()
        }
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
struct ReorderBody {
    order: Vec<String>,
}

async fn reorder_lessons(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ReorderBody>,
) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match yaml_rw::reorder_lessons(&cp, &body.order) {
        Ok(()) => {
            record_action(&state, "reorder_lessons", None);
            StatusCode::OK.into_response()
        }
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- Exercises ---

async fn list_exercises(
    State(state): State<Arc<AppState>>,
    AxumPath(lid): AxumPath<String>,
) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match yaml_rw::list_exercises(&cp, &lid) {
        Ok(exercises) => Json(exercises).into_response(),
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_exercise(
    State(state): State<Arc<AppState>>,
    AxumPath((lid, eid)): AxumPath<(String, String)>,
) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match yaml_rw::read_exercise(&cp, &lid, &eid) {
        Ok(exercise) => Json(exercise).into_response(),
        Err(e) => api_err(StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

async fn create_exercise(
    State(state): State<Arc<AppState>>,
    AxumPath(lid): AxumPath<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    let id = body.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
    match yaml_rw::create_exercise(&cp, &lid, &body) {
        Ok(()) => {
            record_action(
                &state,
                "create_exercise",
                Some(&format!("Created exercise '{}/{}' ", lid, id)),
            );
            StatusCode::CREATED.into_response()
        }
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn update_exercise(
    State(state): State<Arc<AppState>>,
    AxumPath((lid, eid)): AxumPath<(String, String)>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match yaml_rw::update_exercise(&cp, &lid, &eid, &body) {
        Ok(()) => {
            record_action(
                &state,
                "update_exercise",
                Some(&format!("Updated exercise '{}/{}'", lid, eid)),
            );
            StatusCode::OK.into_response()
        }
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn delete_exercise(
    State(state): State<Arc<AppState>>,
    AxumPath((lid, eid)): AxumPath<(String, String)>,
) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match yaml_rw::delete_exercise(&cp, &lid, &eid) {
        Ok(()) => {
            record_action(
                &state,
                "delete_exercise",
                Some(&format!("Deleted exercise '{}/{}'", lid, eid)),
            );
            StatusCode::OK.into_response()
        }
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn reorder_exercises(
    State(state): State<Arc<AppState>>,
    AxumPath(lid): AxumPath<String>,
    Json(body): Json<ReorderBody>,
) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match yaml_rw::reorder_exercises(&cp, &lid, &body.order) {
        Ok(()) => {
            record_action(
                &state,
                "reorder_exercises",
                Some(&format!("Reordered exercises in '{}'", lid)),
            );
            StatusCode::OK.into_response()
        }
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- Execution ---

#[derive(Deserialize)]
struct RunSolutionBody {
    lesson: String,
    exercise: String,
}

#[derive(Serialize)]
struct RunSolutionResponse {
    success: bool,
    stdout: String,
    stderr: String,
}

async fn run_solution(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RunSolutionBody>,
) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    let lesson = body.lesson;
    let exercise = body.exercise;

    let result = tokio::task::spawn_blocking(move || {
        let c = crate::course::load_course(&cp)?;
        let lesson_obj = c
            .loaded_lessons
            .iter()
            .find(|l| l.id == lesson)
            .ok_or_else(|| anyhow::anyhow!("Lesson '{}' not found", lesson))?;
        let exercise_obj = lesson_obj
            .loaded_exercises
            .iter()
            .find(|e| e.id == exercise)
            .ok_or_else(|| anyhow::anyhow!("Exercise '{}' not found", exercise))?;
        let solution_files = exercise_obj.get_solution_files(&c.language.extension);
        if solution_files.is_empty() {
            return Ok::<_, anyhow::Error>(RunSolutionResponse {
                success: false,
                stdout: String::new(),
                stderr: "No solution provided".to_string(),
            });
        }
        let output = crate::exec::runner::run_exercise_with_sandbox(
            &c,
            exercise_obj,
            &solution_files,
            crate::exec::sandbox::SandboxLevel::Basic,
        )?;
        Ok(RunSolutionResponse {
            success: output.success,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    })
    .await;

    match result {
        Ok(Ok(resp)) => Json(resp).into_response(),
        Ok(Err(e)) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Serialize)]
struct ValidationCheck {
    name: String,
    passed: bool,
    message: String,
}

#[derive(Serialize)]
struct ValidateResponse {
    all_passed: bool,
    checks: Vec<ValidationCheck>,
}

async fn validate_course(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    let result = tokio::task::spawn_blocking(move || {
        let c = crate::course::load_course(&cp)?;
        let result = crate::course::validate_course(&c);
        Ok::<_, anyhow::Error>(ValidateResponse {
            all_passed: result.all_passed(),
            checks: result
                .checks
                .into_iter()
                .map(|c| ValidationCheck {
                    name: c.name,
                    passed: c.passed,
                    message: c.message,
                })
                .collect(),
        })
    })
    .await;

    match result {
        Ok(Ok(resp)) => Json(resp).into_response(),
        Ok(Err(e)) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn validate_exercise(
    State(state): State<Arc<AppState>>,
    AxumPath((lid, eid)): AxumPath<(String, String)>,
) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    let result = tokio::task::spawn_blocking(move || {
        let c = crate::course::load_course(&cp)?;
        let exercise = c
            .loaded_lessons
            .iter()
            .find(|l| l.id == lid)
            .and_then(|l| l.loaded_exercises.iter().find(|e| e.id == eid))
            .ok_or_else(|| anyhow::anyhow!("Exercise {}/{} not found", lid, eid))?;
        let solution_files = exercise.get_solution_files(&c.language.extension);
        if solution_files.is_empty() {
            return Ok::<_, anyhow::Error>(RunSolutionResponse {
                success: false,
                stdout: String::new(),
                stderr: "No solution provided".to_string(),
            });
        }
        let (exec_result, _) = crate::exec::execute_exercise(&c, exercise, &solution_files)?;
        let output = crate::exec::runner::run_exercise_with_sandbox(
            &c,
            exercise,
            &solution_files,
            crate::exec::sandbox::SandboxLevel::Basic,
        )?;
        Ok(RunSolutionResponse {
            success: exec_result.is_success(),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    })
    .await;

    match result {
        Ok(Ok(resp)) => Json(resp).into_response(),
        Ok(Err(e)) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn toolcheck(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let cp = match course_path(&state) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    let result = tokio::task::spawn_blocking(move || {
        let c = crate::course::load_course(&cp)?;
        let tools = crate::exec::toolcheck::check_language_tools(&c.language);
        let missing: Vec<_> = tools
            .iter()
            .filter(|t| !t.found)
            .map(|t| t.command.clone())
            .collect();
        Ok::<_, anyhow::Error>((missing.is_empty(), missing))
    })
    .await;

    match result {
        Ok(Ok((all_found, missing))) => {
            Json(serde_json::json!({ "all_found": all_found, "missing": missing })).into_response()
        }
        Ok(Err(e)) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- AI Chat Proxy ---

#[derive(Deserialize)]
struct AiChatBody {
    messages: Vec<AiMessage>,
    #[allow(dead_code)]
    context: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
struct AiMessage {
    role: String,
    content: String,
}

const AI_SYSTEM_PROMPT: &str = r#"You are an expert course authoring assistant for LearnLocal, an offline terminal-based programming tutorial framework.

You help authors create high-quality programming courses. You understand:

**Course structure:** A course has lessons, each lesson has a content.md (markdown read by students) and exercises (YAML files).

**Exercise YAML schema:**
```yaml
id: exercise-id
title: "Human-readable title"
type: write|fix|fill-blank|command|predict|multiple-choice
prompt: "What the student sees as the task description"
starter: |
  // Code the student starts with
solution: |
  // The correct solution
validation:
  method: output|regex|compile-only|state
  expected_output: "exact output to match"
  pattern: "regex pattern"
hints:
  - "Progressive hints, revealed one at a time"
  - "Each hint should get more specific"
  - "Last hint should nearly give the answer"
explanation: "Shown after the student passes — teach the WHY"
```

**Staged exercises** (optional, for iterative challenges):
```yaml
stages:
  - id: basic
    title: "Basic Solution"
    prompt: "Start with the simple case"
    validation: { method: output, expected_output: "..." }
    hints: ["..."]
    solution: |
      // Solution for this stage
```
Student code carries forward between stages.

**Best practices:**
- Prompts should be clear and specific — state exactly what output is expected
- Hints should progress from conceptual to specific (3 hints is ideal)
- Solutions must produce the exact expected_output
- Explanations should teach the concept, not just restate the answer
- Exercise difficulty should ramp gradually within a lesson
- Each lesson should have 6-8 exercises
- Mix exercise types: mostly "write", some "fix" and "fill-blank"
- Command exercises use type: command and validate with method: state (filesystem assertions)

You can help with: writing exercise prompts, generating starter code, suggesting hints, creating solutions, reviewing exercise quality, structuring lessons, and general course design advice."#;

async fn ai_chat(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AiChatBody>,
) -> impl IntoResponse {
    let mut messages = vec![AiMessage {
        role: "system".to_string(),
        content: AI_SYSTEM_PROMPT.to_string(),
    }];

    // Add course context if available
    if let Ok(cp) = course_path(&state) {
        if let Ok(meta) = yaml_rw::read_course_meta(&cp) {
            messages.push(AiMessage {
                role: "system".to_string(),
                content: format!(
                    "Current course: \"{}\" v{} — {}",
                    meta.name, meta.version, meta.description
                ),
            });
        }
    }

    messages.extend(body.messages);

    let ollama_url =
        std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string());

    let client = reqwest::Client::new();
    let result = client
        .post(format!("{}/v1/chat/completions", ollama_url))
        .json(&serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false,
        }))
        .send()
        .await;

    match result {
        Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>().await {
            Ok(data) => Json(data).into_response(),
            Err(e) => api_err(StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
        },
        Ok(resp) => {
            let s = resp.status();
            let b = resp.text().await.unwrap_or_default();
            api_err(StatusCode::BAD_GATEWAY, format!("Ollama {}: {}", s, b)).into_response()
        }
        Err(e) => api_err(
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Cannot reach Ollama at {}: {}", ollama_url, e),
        )
        .into_response(),
    }
}
