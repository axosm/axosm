use axum::extract::FromRef;
use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use chrono::Utc;
use sqlx::SqlitePool;

use anyhow::Result;
use tokio::sync::broadcast;
use tracing_subscriber::FmtSubscriber;

use crate::handlers;

// #[derive(Clone)]
// pub enum DbPool {
//     Sqlite(sqlx::SqlitePool),
//     Postgres(sqlx::PgPool),
// }

// 1. Define the DbPool type alias based on the enabled feature
#[cfg(feature = "local_mode")]
pub type DbPool = sqlx::SqlitePool;

#[cfg(not(feature = "local_mode"))]
pub type DbPool = sqlx::PgPool;

#[cfg(feature = "local_mode")]
pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub notify: broadcast::Sender<String>,
}

#[cfg(feature = "prod_mode")]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub jwt_secret: String,
    //     pub notify: broadcast::Sender<String>,
}

// pub struct AppState {
//     pub db: SqlitePool,
//     pub notify: broadcast::Sender<String>,
// }

pub fn init_tracing() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

pub async fn init_state() -> anyhow::Result<Arc<AppState>> {
    let db = SqlitePool::connect("sqlite://./game.db?mode=rwc").await?;

    // Set WAL mode immediately after connecting
    sqlx::query("PRAGMA journal_mode = WAL;")
        .execute(&db)
        .await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let (tx, _) = broadcast::channel(128);

    Ok(Arc::new(AppState { db, notify: tx }))
}

// pub fn spawn_worker(state: Arc<AppState>) {
//     tokio::spawn(async move {
//         if let Err(e) = run_worker(state).await {
//             tracing::error!("worker failed: {:?}", e);
//         }
//     });
// }

// // Background worker that checks move_orders and resolves arrivals
// async fn run_worker(state: Arc<AppState>) {
//     tracing::info!("worker started");
//     let db = state.db.clone();
//     let tx = state.notify.clone();

//     loop {

//     }
// }

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        // Auth
        // .route("/auth/register", post(handlers::auth::register))
        // .route("/auth/login",    post(handlers::auth::login))
        // Game
        .route("/api/state", get(handlers::state::get_game_state))
        // .route("/api/state/:player_id", get(api::state::get_state))
        // .route("/api/move", post(api::move_unit::handler))
        // .route("/api/events", get(api::events::handler))
        .with_state(state)
}
