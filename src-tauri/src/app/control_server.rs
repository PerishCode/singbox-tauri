use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::app::app::App;
use crate::app::handlers;

pub const CONTROL_SERVER_ADDR: &str = "127.0.0.1:18427";

pub fn spawn(app: Arc<App>) {
    tauri::async_runtime::spawn(async move {
        if let Err(err) = serve(app).await {
            eprintln!("control server failed: {err}");
        }
    });
}

async fn serve(app: Arc<App>) -> Result<(), String> {
    let addr: SocketAddr = CONTROL_SERVER_ADDR
        .parse()
        .map_err(|err| format!("invalid control server addr: {err}"))?;

    let router = Router::new()
        .merge(handlers::control::routes(app))
        .layer(CorsLayer::permissive());

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|err| format!("failed to bind control server: {err}"))?;

    axum::serve(listener, router)
        .await
        .map_err(|err| format!("failed to serve control server: {err}"))
}
