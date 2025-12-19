use axum::{
    extract::{State, Query, Path},
    response::sse::{Sse, KeepAlive, Event},
    routing::{get, post},
    Json, Router
};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, sqlite::SqliteQueryResult, FromRow};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::broadcast::{self, Sender};
use tokio::time::Instant;
use tokio::net::TcpListener;
use chrono::{Utc, DateTime};
use tracing_subscriber::FmtSubscriber;
use anyhow::Result;

use futures_util::stream::{Stream, StreamExt};
use std::convert::Infallible;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Clone)]
struct AppState {
    db:SqlitePool,
    notify: broadcast::Sender<String>,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
struct Player {
    id: i64,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
struct Unit {
    id: i64,
    player_id: i64,
    x: i32,
    y: i32,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
struct MoveOrder {
    id: i64,
    unit_id: i64,
    from_x: i32,
    from_y: i32,
    to_x: i32,
    to_y: i32,
    arrival_time: i64, // Unix timestamp (seconds)
}

#[derive(Deserialize)]
struct MoveRequest {
    player_id: i64,
    unit_id: i64,
    to_x: i32,
    to_y: i32,
}

#[derive(Serialize)]
struct StateResponse {
    units: Vec<Unit>,
    now: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct EncounterEvent {
    r#type: String,
    player_a: i64,
    player_b: i64,
    x: i32,
    y: i32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // logging
    let subscriber = FmtSubscriber::builder().with_max_level(tracing::Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber)?;
// println!("Current working directory: {:?}", std::env::current_dir()?);
    // sqlite DB path (file)
    // let db = SqlitePool::connect("sqlite:///C:/Users/Mathieu/AppData/Local/sci4x/game.db").await?;
    let db = SqlitePool::connect("sqlite://./game.db?mode=rwc").await?;
    
    // let db = SqlitePool::connect("sqlite://./game.db").await?;
    sqlx::migrate!("./migrations").run(&db).await?;

    // broadcast channel for SSE
    let (sse_tx, _) = broadcast::channel::<String>(128);

    let app_state = AppState {
        db,
        notify : sse_tx,
    };

    let state = Arc::new(app_state);

    // spawn background worker
    let worker_state = state.clone();
    tokio::spawn(async move {
        run_worker(worker_state).await;
    });

    // routes
    let app = Router::new()
        .route("/api/state", get(api_state))
        .route("/api/move", post(api_move))
        .route("/api/events", get(api_events))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

// GET /api/state?player_id=1
async fn api_state(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<StateResponse>, (axum::http::StatusCode, String)> {
    let now = Utc::now();

    // return all units for prototype
    let units: Vec<Unit> =
    sqlx::query_as::<_, Unit>(
        "SELECT id, player_id, x, y FROM units"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(StateResponse { units, now }))
}

// POST /api/move
async fn api_move(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MoveRequest>,
) -> Result<axum::response::Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    // validate ownership of unit
    // let unit = sqlx::query("SELECT id, player_id, x, y FROM units WHERE id = ?", payload.unit_id)
    //     .fetch_optional(&state.db)
    //     .await
    //     .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

let unit =
    sqlx::query_as::<_, Unit>(
        "SELECT id, player_id, x, y FROM units WHERE id = ?"
    )
    .bind(payload.unit_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let unit = match unit {
        Some(u) => u,
        None => return Err((axum::http::StatusCode::BAD_REQUEST, "unit not found".into())),
    };

    if unit.player_id != payload.player_id {
        return Err((axum::http::StatusCode::FORBIDDEN, "not your unit".into()));
    }

    // create a move_order with arrival_time = now + 10s
    let arrival_time = (Utc::now() + chrono::Duration::seconds(10)).timestamp();

    let res = sqlx::query(
        r#"
        INSERT INTO move_orders (unit_id, from_x, from_y, to_x, to_y, arrival_time)
        VALUES (?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(payload.unit_id)
    .bind(unit.x)
    .bind(unit.y)
    .bind(payload.to_x)
    .bind(payload.to_y)
    .bind(arrival_time)
    .execute(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("move order created id={:?}", res.last_insert_rowid());

    Ok(Json(serde_json::json!({
        "ok": true,
        "arrival_time": DateTime::<Utc>::from_timestamp(arrival_time, 0).unwrap().to_rfc3339(),
    })))
}

// SSE endpoint
async fn api_events(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Sse<impl futures::Stream<Item = Result<Event, axum::Error>>> {
    // subscribe to broadcast channel
    let mut rx = state.notify.subscribe();

    // create a stream of axum SSE events
    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    // msg is a JSON string — we send to the client raw
                    let event = Event::default().data(msg);
                    yield Ok::<Event, axum::Error>(event);
                }
                Err(broadcast::error::RecvError::Lagged(_n)) => {
                    // skip / continue
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}


// Background worker that checks move_orders and resolves arrivals
async fn run_worker(state: Arc<AppState>) {
    tracing::info!("worker started");
    let db = state.db.clone();
    let tx = state.notify.clone();

    loop {
        // check for arrived move orders
        // arrival_time <= now
        let now = Utc::now().timestamp();
        // let arrival_time = (Utc::now() + chrono::Duration::seconds(10)).timestamp();

        let orders = match sqlx::query_as::<_, Unit>(
        r#"
            SELECT id, unit_id, from_x, from_y, to_x, to_y, arrival_time as "arrival_time: DateTime<Utc>"
            FROM move_orders
            WHERE arrival_time <= ?
            "#
    )
    .bind(now)

       .fetch_all(&db).await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("db error fetching orders: {}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        for o in orders.into_iter() {
            let tx_clone = tx.clone();
            let db_clone = db.clone();
            // process each arrival in its own task to not block
            tokio::spawn(async move {
                if let Err(e) = process_arrival(o, db_clone, tx_clone).await {
                    tracing::error!("error processing arrival: {:?}", e);
                }
            });
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn process_arrival(order_row: Unit, db: SqlitePool, tx: Sender<String>) -> anyhow::Result<()> {
    // Rehydrate fields
    // Note: We used a query! macro earlier; here we accept a generic row.
    // But to simplify, re-query the order by id.
    let id: i64 = order_row.id;
    let order = sqlx::query_as::<_, MoveOrder>(
        r#"
        SELECT id, unit_id, from_x, from_y, to_x, to_y,
            arrival_time as "arrival_time: DateTime<Utc>"
        FROM move_orders
        WHERE id = ?
        "#
    )
    .bind(id)
    .fetch_one(&db)
    .await?;

    // Update unit position inside transaction
    let mut txn = db.begin().await?;
    sqlx::query(
        r#"
        UPDATE units SET x = ?, y = ? WHERE id = ?
        "#)
        .bind(order.to_x)
        .bind(order.to_y)
        .bind(order.unit_id)
    .execute(&mut *txn).await?;

    txn.commit().await?;

    // check for encounter: any other unit on same tile
    let others = sqlx::query_as::<_, Unit>(
        r#"
        SELECT id, player_id FROM units WHERE x = ? AND y = ? AND id != ?
        "#)
        .bind(order.to_x)
        .bind(order.to_y)
        .bind(order.unit_id)
    .fetch_all(&db)
    .await?;

    if !others.is_empty() {
        // get this unit's player
        let self_player: (i64,) = sqlx::query_as(
            "SELECT player_id FROM units WHERE id = ?")
        .bind(order.unit_id)
            .fetch_one(&db)
            .await?;

        for other in others {
            let event = EncounterEvent {
                r#type: "encounter".to_string(),
                player_a: self_player.0,
                player_b: other.player_id,
                x: order.to_x,
                y: order.to_y,
            };
            let json = serde_json::to_string(&event)?;
            // broadcast; ignoring if no listeners
            let _ = tx.send(json);
            tracing::info!("encounter event broadcast");
        }
    }

    Ok(())
}

// // helper to extract column safely in process_arrival (SqliteRow access)
// use sqlx::Row;
