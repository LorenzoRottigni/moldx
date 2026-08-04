//! Axum-based web UI server.
//!
//! Serves a single-page HTML application at `/` and exposes a small REST API:
//!
//! | Method   | Path                | Description                         |
//! |----------|---------------------|-------------------------------------|
//! | `GET`    | `/`                 | HTML single-page application        |
//! | `GET`    | `/api/modules`      | List of discovered modules          |
//! | `POST`   | `/api/run`          | Spawn a command (strategy optional) |
//! | `GET`    | `/api/running`      | List of all tracked processes       |
//! | `DELETE` | `/api/running/:id`  | Kill a running process              |
//!
//! The server binds to `0.0.0.0` (all interfaces) but the HTML page is the
//! only client; cross-origin POST requests are blocked by the browser’s
//! same-origin policy because the JSON `Content-Type` triggers a CORS
//! preflight that the server does not satisfy.
use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::RwLock;

use crate::{
    config::MoldxConfig,
    detector::{self, AGNOSTIC_STRATEGY, Module},
    executor,
    state::{AppState, ProcessStatus},
};

/// The compiled-in HTML/JS single-page app, embedded at build time.
static HTML: &str = include_str!("static/index.html");

// ─── Shared state ─────────────────────────────────────────────────────────────

/// Per-request state injected by axum via [`axum::extract::State`].
#[derive(Clone)]
struct WebState {
    config: Arc<MoldxConfig>,
    processes: AppState,
    /// Module list populated asynchronously at server startup.
    modules: Arc<RwLock<Vec<Module>>>,
}
// ─── API types ────────────────────────────────────────────────────────────────

/// JSON representation of a discovered module.
#[derive(Serialize)]
struct ApiModule {
    path: String,
    strategies: HashMap<String, Vec<String>>,
}

/// Request body for `POST /api/run`.
///
/// `strategy` is optional. Omit it (or pass `"agnostic"`) to run
/// `.moldx/commands/<command>.sh`.
#[derive(Deserialize)]
struct RunRequest {
    module_path: String,
    strategy: Option<String>,
    command: String,
}

/// Response body for `POST /api/run`.
#[derive(Serialize)]
struct RunResponse {
    id: u64,
}

/// JSON representation of a tracked process returned by `GET /api/running`.
#[derive(Serialize)]
struct ApiProcess {
    id: u64,
    module_path: String,
    strategy: String,
    command: String,
    pid: Option<u32>,
    status: String,
    last_output: String,
}

// ─── Handlers ────────────────────────────────────────────────────────────────

async fn serve_html() -> Html<&'static str> {
    Html(HTML)
}

async fn get_modules(State(ws): State<WebState>) -> Json<Vec<ApiModule>> {
    let modules = ws.modules.read().await;
    Json(
        modules
            .iter()
            .map(|m| ApiModule {
                path: m.path.to_string_lossy().to_string(),
                strategies: m.strategies.clone(),
            })
            .collect(),
    )
}

async fn run_command(
    State(ws): State<WebState>,
    Json(req): Json<RunRequest>,
) -> Result<Json<RunResponse>, (StatusCode, String)> {
    let module_path = PathBuf::from(&req.module_path);

    if !is_valid_name(&req.command) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid command name: {}", req.command),
        ));
    }
    if let Some(ref strategy) = req.strategy
        && !is_valid_name(strategy)
    {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid strategy name: {}", strategy),
        ));
    }

    let detected = detector::detect_strategies(&ws.config.detector_path, &module_path)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Detector error: {}", e),
            )
        })?;

    let (script, strategy) = if let Some(strategy_hint) = req.strategy.clone() {
        if strategy_hint == AGNOSTIC_STRATEGY {
            let script = ws.config.commands_dir.join(format!("{}.sh", req.command));
            if !script.exists() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("Agnostic command '{}' not found", req.command),
                ));
            }
            (script, AGNOSTIC_STRATEGY.to_string())
        } else {
            if !detected.is_empty() && !detected.iter().any(|s| s == &strategy_hint) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!(
                        "Strategy '{}' is not available for {}. Available: {}",
                        strategy_hint,
                        module_path.display(),
                        detected.join(", ")
                    ),
                ));
            }

            let script = ws
                .config
                .commands_dir
                .join(&req.command)
                .join(format!("{}.sh", strategy_hint));
            if !script.exists() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!(
                        "Command '{}' has no '{}' variant",
                        req.command, strategy_hint
                    ),
                ));
            }
            (script, strategy_hint)
        }
    } else {
        let mut resolved: Option<(PathBuf, String)> = None;
        for strategy in &detected {
            let candidate = ws
                .config
                .commands_dir
                .join(&req.command)
                .join(format!("{}.sh", strategy));
            if candidate.exists() {
                resolved = Some((candidate, strategy.clone()));
                break;
            }
        }

        if let Some(found) = resolved {
            found
        } else {
            let agnostic = ws.config.commands_dir.join(format!("{}.sh", req.command));
            if agnostic.exists() {
                (agnostic, AGNOSTIC_STRATEGY.to_string())
            } else {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!(
                        "Command '{}' not found for {}",
                        req.command,
                        module_path.display()
                    ),
                ));
            }
        }
    };

    let id = ws
        .processes
        .add_process(&req.module_path, &strategy, &req.command, None);

    let processes = ws.processes.clone();
    tokio::spawn(executor::run_and_track(id, script, module_path, processes));

    Ok(Json(RunResponse { id }))
}

fn is_valid_name(name: &str) -> bool {
    !(name.contains('/') || name.contains('\\') || name == "." || name == "..")
}

async fn get_running(State(ws): State<WebState>) -> Json<Vec<ApiProcess>> {
    let all = ws.processes.get_all();
    Json(
        all.into_iter()
            .map(|p| {
                let status = match &p.status {
                    ProcessStatus::Running => "Running".to_string(),
                    ProcessStatus::Completed(code) => format!("Done({})", code),
                    ProcessStatus::Failed(msg) => format!("Failed: {}", msg),
                    ProcessStatus::Killed => "Killed".to_string(),
                };
                let last_output = p
                    .output_lines
                    .iter()
                    .rev()
                    .take(5)
                    .rev()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n");
                ApiProcess {
                    id: p.id,
                    module_path: p.module_path,
                    strategy: p.strategy,
                    command: p.command,
                    pid: p.pid,
                    status,
                    last_output,
                }
            })
            .collect(),
    )
}

async fn kill_process(State(ws): State<WebState>, Path(id): Path<u64>) -> StatusCode {
    ws.processes.kill_process(id);
    StatusCode::NO_CONTENT
}

// ─── Entry point ─────────────────────────────────────────────────────────────

/// Serve the embedded HTML application.
pub async fn run(config: MoldxConfig, port: u16, processes: AppState) -> Result<()> {
    let config = Arc::new(config);
    let modules: Arc<RwLock<Vec<Module>>> = Arc::new(RwLock::new(Vec::new()));

    // Background module discovery
    {
        let config2 = config.clone();
        let modules2 = modules.clone();
        tokio::spawn(async move {
            match detector::discover_modules(&config2.root, &config2, 3).await {
                Ok(found) => {
                    eprintln!("[moldx] Discovered {} module(s)", found.len());
                    *modules2.write().await = found;
                }
                Err(e) => eprintln!("[moldx] Module scan error: {}", e),
            }
        });
    }

    let web_state = WebState {
        config,
        processes,
        modules,
    };

    let app = Router::new()
        .route("/", get(serve_html))
        .route("/api/modules", get(get_modules))
        .route("/api/run", post(run_command))
        .route("/api/running", get(get_running))
        .route("/api/running/:id", delete(kill_process))
        .with_state(web_state);

    let addr = format!("0.0.0.0:{}", port);
    println!("MoldX web UI → http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
