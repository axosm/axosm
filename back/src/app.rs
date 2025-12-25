use std::sync::Arc;

use axum::{Router, routing::{get, post}};
use sqlx::{SqlitePool};

use anyhow::Result;
use tokio::sync::broadcast;
use tracing_subscriber::FmtSubscriber;

use crate::api;

pub struct AppState {
    pub db: SqlitePool,
    pub notify: broadcast::Sender<String>,
}



pub fn init_tracing() -> Result<()> {
    let subscriber = FmtSubscriber::builder().with_max_level(tracing::Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

pub async fn init_state() -> anyhow::Result<Arc<AppState>> {
    let db = SqlitePool::connect("sqlite://./game.db?mode=rwc").await?;
    sqlx::migrate!("./migrations").run(&db).await?;

    let (tx, _) = broadcast::channel(128);

    Ok(Arc::new(AppState { db, notify: tx }))
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/state", get(api::state::handler))
        .route("/api/move", post(api::move_unit::handler))
        .route("/api/events", get(api::events::handler))
        .with_state(state)
}