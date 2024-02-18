use crate::{
    routes::{health_check, subscribe},
    Result, Storage,
};
use axum::{
    routing::{get, post},
    Extension, Router,
};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

pub fn app(storage: Storage) -> Router {
    let state = std::sync::Arc::new(storage);
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(state))
}
pub async fn run(listener: TcpListener, app: Router) -> Result<()> {
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
