use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use axum::{routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use utoipa::OpenApi;

use crate::app::app::App;
use crate::openapi::ApiDoc;
use crate::runtime_paths::RuntimePaths;
use crate::app::services::network::LocalNetworkSnapshot;
use crate::app::services::singbox::{SingboxBootstrapReport, SingboxRuntimeStatus};
use crate::app::services::subscription::{
    SubscriptionDefinitionSnapshot, SubscriptionRuntimeSnapshot,
};

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub ok: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AppEventAccepted {
    pub ok: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ControlStateResponse {
    pub runtime: RuntimePaths,
    pub bootstrap: SingboxBootstrapReport,
    pub status: SingboxRuntimeStatus,
    pub subscription: SubscriptionDefinitionSnapshot,
    pub subscription_runtime: SubscriptionRuntimeSnapshot,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ControlSnapshotResponse {
    pub tab: String,
    pub runtime: RuntimePaths,
    pub bootstrap: SingboxBootstrapReport,
    pub status: SingboxRuntimeStatus,
    pub app_log: String,
    pub singbox_log: String,
    pub session_raw: String,
    pub runtime_metadata: String,
    pub subscription: SubscriptionDefinitionSnapshot,
    pub subscription_runtime: SubscriptionRuntimeSnapshot,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LocalNetworkResponse {
    pub network: LocalNetworkSnapshot,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubscriptionApplyResponse {
    pub subscription_runtime: SubscriptionRuntimeSnapshot,
    pub status: SingboxRuntimeStatus,
}

pub fn routes(app: Arc<App>) -> Router {
    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/state", get(state))
        .route("/api/v1/start", post(start))
        .route("/api/v1/stop", post(stop))
        .route("/api/v1/restart", post(restart))
        .route("/api/v1/snapshot", get(snapshot))
        .route("/api/v1/network", get(network))
        .route("/api/v1/subscription/refresh", post(refresh_subscription))
        .route("/api/v1/subscription/apply", post(apply_subscription))
        .route("/api/v1/events", post(append_event))
        .route("/api/v1/logs/app", get(app_log))
        .route("/api/v1/logs/singbox", get(singbox_log))
        .route("/api/openapi.json", get(get_openapi))
        .with_state(app)
}

#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses((status = 200, description = "Control plane health", body = HealthResponse))
)]
pub async fn health() -> impl IntoResponse {
    Json(HealthResponse { ok: true })
}

#[utoipa::path(
    get,
    path = "/api/v1/state",
    responses((status = 200, description = "Aggregated local control state", body = ControlStateResponse))
)]
pub async fn state(
    State(app): State<Arc<App>>,
) -> Result<Json<ControlStateResponse>, (axum::http::StatusCode, String)> {
    let runtime = app.runtime_paths().map_err(internal_error)?;
    let bootstrap = app.bootstrap_singbox().map_err(internal_error)?;
    let status = app.singbox_status().map_err(internal_error)?;
    let subscription = app.subscription_definition().map_err(internal_error)?;
    let subscription_runtime = app.subscription_runtime().map_err(internal_error)?;

    Ok(Json(ControlStateResponse {
        runtime,
        bootstrap,
        status,
        subscription,
        subscription_runtime,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/snapshot",
    responses((status = 200, description = "UI-friendly snapshot for local control plane", body = ControlSnapshotResponse))
)]
pub async fn snapshot(
    State(app): State<Arc<App>>,
) -> Result<Json<ControlSnapshotResponse>, (axum::http::StatusCode, String)> {
    let runtime = app.runtime_paths().map_err(internal_error)?;
    let bootstrap = app.bootstrap_singbox().map_err(internal_error)?;
    let status = app.singbox_status().map_err(internal_error)?;
    let app_log = app.read_app_log().map_err(internal_error)?;
    let singbox_log = app.read_singbox_log().map_err(internal_error)?;
    let session_raw = app.read_session_raw().map_err(internal_error)?;
    let runtime_metadata = app.read_runtime_metadata().map_err(internal_error)?;
    let subscription = app.subscription_definition().map_err(internal_error)?;
    let subscription_runtime = app.subscription_runtime().map_err(internal_error)?;

    Ok(Json(ControlSnapshotResponse {
        tab: "singbox 控制面板".to_string(),
        runtime,
        bootstrap,
        status,
        app_log,
        singbox_log,
        session_raw,
        runtime_metadata,
        subscription,
        subscription_runtime,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/network",
    responses((status = 200, description = "Read-only local network snapshot", body = LocalNetworkResponse))
)]
pub async fn network(
    State(app): State<Arc<App>>,
) -> Result<Json<LocalNetworkResponse>, (axum::http::StatusCode, String)> {
    Ok(Json(LocalNetworkResponse {
        network: app.local_network_snapshot(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/start",
    responses((status = 200, description = "Starts sing-box", body = SingboxRuntimeStatus))
)]
pub async fn start(
    State(app): State<Arc<App>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let status = app.start_singbox().map_err(internal_error)?;
    Ok(Json(json!(status)))
}

#[utoipa::path(
    post,
    path = "/api/v1/stop",
    responses((status = 200, description = "Stops sing-box", body = SingboxRuntimeStatus))
)]
pub async fn stop(
    State(app): State<Arc<App>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let status = app.stop_singbox().map_err(internal_error)?;
    Ok(Json(json!(status)))
}

#[utoipa::path(
    post,
    path = "/api/v1/restart",
    responses((status = 200, description = "Restarts sing-box", body = SingboxRuntimeStatus))
)]
pub async fn restart(
    State(app): State<Arc<App>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let status = app.restart_singbox().map_err(internal_error)?;
    Ok(Json(json!(status)))
}

#[utoipa::path(
    post,
    path = "/api/v1/subscription/refresh",
    responses((status = 200, description = "Refreshes the encrypted subscription", body = SubscriptionRuntimeSnapshot))
)]
pub async fn refresh_subscription(
    State(app): State<Arc<App>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let app = app.clone();
    let snapshot = tokio::task::spawn_blocking(move || app.refresh_subscription())
        .await
        .map_err(|err| internal_error(format!("subscription refresh task failed: {err}")))?
        .map_err(internal_error)?;
    Ok(Json(json!(snapshot)))
}

#[utoipa::path(
    post,
    path = "/api/v1/subscription/apply",
    responses((status = 200, description = "Refreshes the encrypted subscription and restarts sing-box", body = SubscriptionApplyResponse))
)]
pub async fn apply_subscription(
    State(app): State<Arc<App>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let app = app.clone();
    let (subscription_runtime, status) = tokio::task::spawn_blocking(move || app.refresh_and_apply_subscription())
        .await
        .map_err(|err| internal_error(format!("subscription apply task failed: {err}")))?
        .map_err(internal_error)?;
    Ok(Json(json!(SubscriptionApplyResponse { subscription_runtime, status })))
}

#[utoipa::path(
    get,
    path = "/api/v1/logs/app",
    responses((status = 200, description = "Returns app log as plain text", content_type = "text/plain", body = String))
)]
pub async fn app_log(
    State(app): State<Arc<App>>,
) -> Result<String, (axum::http::StatusCode, String)> {
    app.read_app_log().map_err(internal_error)
}

#[utoipa::path(
    get,
    path = "/api/v1/logs/singbox",
    responses((status = 200, description = "Returns sing-box log as plain text", content_type = "text/plain", body = String))
)]
pub async fn singbox_log(
    State(app): State<Arc<App>>,
) -> Result<String, (axum::http::StatusCode, String)> {
    app.read_singbox_log().map_err(internal_error)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AppEventRequest {
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/events",
    request_body = AppEventRequest,
    responses((status = 200, description = "Appends an app event", body = AppEventAccepted))
)]
pub async fn append_event(
    State(app): State<Arc<App>>,
    Json(request): Json<AppEventRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    app.append_app_event(&request.message).map_err(internal_error)?;
    Ok(Json(json!({ "ok": true })))
}

#[utoipa::path(
    get,
    path = "/api/openapi.json",
    responses((status = 200, description = "OpenAPI schema", body = Object))
)]
pub async fn get_openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

fn internal_error(message: String) -> (axum::http::StatusCode, String) {
    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, message)
}
