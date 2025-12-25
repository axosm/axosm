// refacto https://chatgpt.com/c/694c7daf-6844-832e-97fc-52ba63b2fb58
// src/
// ├── main.rs
// ├── app.rs            # AppState, router setup
// ├── db/
// │   ├── mod.rs
// │   ├── models.rs     # Player, Unit, MoveOrder
// │   └── queries.rs    # SQLx queries
// ├── api/
// │   ├── mod.rs
// │   ├── state.rs      # GET /api/state
// │   ├── move.rs       # POST /api/move
// │   └── events.rs     # SSE endpoint
// ├── worker/
// │   ├── mod.rs
// │   ├── runner.rs     # run_worker loop
// │   └── arrivals.rs  # process_arrival
// ├── domain/
// │   └── events.rs     # EncounterEvent, domain logic
// └── error.rs          # API error handling

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

mod api;
mod app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // logging
    app::init_tracing()?;

    let state = app::init_state().await?;
    app::spawn_worker(state.clone());

    let app = app::router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

// // GET /api/state?player_id=1
// async fn api_state(
//     State(state): State<Arc<AppState>>,
//     Query(params): Query<std::collections::HashMap<String, String>>,
// ) -> Result<Json<StateResponse>, (axum::http::StatusCode, String)> {
//     let now = Utc::now();

//     // return all units for prototype
//     let units: Vec<Unit> =
//     sqlx::query_as::<_, Unit>(
//         "SELECT id, player_id, x, y FROM units"
//     )
//     .fetch_all(&state.db)
//     .await
//     .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

//     Ok(Json(StateResponse { units, now }))
// }

// // POST /api/move
// async fn api_move(
//     State(state): State<Arc<AppState>>,
//     Json(payload): Json<MoveRequest>,
// ) -> Result<axum::response::Json<serde_json::Value>, (axum::http::StatusCode, String)> {
//     // validate ownership of unit
//     // let unit = sqlx::query("SELECT id, player_id, x, y FROM units WHERE id = ?", payload.unit_id)
//     //     .fetch_optional(&state.db)
//     //     .await
//     //     .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

// let unit =
//     sqlx::query_as::<_, Unit>(
//         "SELECT id, player_id, x, y FROM units WHERE id = ?"
//     )
//     .bind(payload.unit_id)
//     .fetch_optional(&state.db)
//     .await
//     .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

//     let unit = match unit {
//         Some(u) => u,
//         None => return Err((axum::http::StatusCode::BAD_REQUEST, "unit not found".into())),
//     };

//     if unit.player_id != payload.player_id {
//         return Err((axum::http::StatusCode::FORBIDDEN, "not your unit".into()));
//     }

//     // create a move_order with arrival_time = now + 10s
//     let arrival_time = (Utc::now() + chrono::Duration::seconds(10)).timestamp();

//     let res = sqlx::query(
//         r#"
//         INSERT INTO move_orders (unit_id, from_x, from_y, to_x, to_y, arrival_time)
//         VALUES (?, ?, ?, ?, ?, ?)
//         "#
//     )
//     .bind(payload.unit_id)
//     .bind(unit.x)
//     .bind(unit.y)
//     .bind(payload.to_x)
//     .bind(payload.to_y)
//     .bind(arrival_time)
//     .execute(&state.db)
//     .await
//     .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

//     tracing::info!("move order created id={:?}", res.last_insert_rowid());

//     Ok(Json(serde_json::json!({
//         "ok": true,
//         "arrival_time": DateTime::<Utc>::from_timestamp(arrival_time, 0).unwrap().to_rfc3339(),
//     })))
// }

// // SSE endpoint
// async fn api_events(
//     State(state): State<Arc<AppState>>,
//     Query(params): Query<std::collections::HashMap<String, String>>,
// ) -> Sse<impl futures::Stream<Item = Result<Event, axum::Error>>> {
//     // subscribe to broadcast channel
//     let mut rx = state.notify.subscribe();

//     // create a stream of axum SSE events
//     let stream = async_stream::stream! {
//         loop {
//             match rx.recv().await {
//                 Ok(msg) => {
//                     // msg is a JSON string — we send to the client raw
//                     let event = Event::default().data(msg);
//                     yield Ok::<Event, axum::Error>(event);
//                 }
//                 Err(broadcast::error::RecvError::Lagged(_n)) => {
//                     // skip / continue
//                     continue;
//                 }
//                 Err(broadcast::error::RecvError::Closed) => {
//                     break;
//                 }
//             }
//         }
//     };

//     Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
// }


// // Background worker that checks move_orders and resolves arrivals
// async fn run_worker(state: Arc<AppState>) {
//     tracing::info!("worker started");
//     let db = state.db.clone();
//     let tx = state.notify.clone();

//     loop {
//         // check for arrived move orders
//         // arrival_time <= now
//         let now = Utc::now().timestamp();
//         // let arrival_time = (Utc::now() + chrono::Duration::seconds(10)).timestamp();

//         let orders = match sqlx::query_as::<_, Unit>(
//         r#"
//             SELECT id, unit_id, from_x, from_y, to_x, to_y, arrival_time as "arrival_time: DateTime<Utc>"
//             FROM move_orders
//             WHERE arrival_time <= ?
//             "#
//     )
//     .bind(now)

//        .fetch_all(&db).await {
//             Ok(v) => v,
//             Err(e) => {
//                 tracing::error!("db error fetching orders: {}", e);
//                 tokio::time::sleep(Duration::from_secs(1)).await;
//                 continue;
//             }
//         };

//         for o in orders.into_iter() {
//             let tx_clone = tx.clone();
//             let db_clone = db.clone();
//             // process each arrival in its own task to not block
//             tokio::spawn(async move {
//                 if let Err(e) = process_arrival(o, db_clone, tx_clone).await {
//                     tracing::error!("error processing arrival: {:?}", e);
//                 }
//             });
//         }

//         tokio::time::sleep(Duration::from_secs(1)).await;
//     }
// }

// async fn process_arrival(order_row: Unit, db: SqlitePool, tx: Sender<String>) -> anyhow::Result<()> {
//     // Rehydrate fields
//     // Note: We used a query! macro earlier; here we accept a generic row.
//     // But to simplify, re-query the order by id.
//     let id: i64 = order_row.id;
//     let order = sqlx::query_as::<_, MoveOrder>(
//         r#"
//         SELECT id, unit_id, from_x, from_y, to_x, to_y,
//             arrival_time as "arrival_time: DateTime<Utc>"
//         FROM move_orders
//         WHERE id = ?
//         "#
//     )
//     .bind(id)
//     .fetch_one(&db)
//     .await?;

//     // Update unit position inside transaction
//     let mut txn = db.begin().await?;
//     sqlx::query(
//         r#"
//         UPDATE units SET x = ?, y = ? WHERE id = ?
//         "#)
//         .bind(order.to_x)
//         .bind(order.to_y)
//         .bind(order.unit_id)
//     .execute(&mut *txn).await?;

//     txn.commit().await?;

//     // check for encounter: any other unit on same tile
//     let others = sqlx::query_as::<_, Unit>(
//         r#"
//         SELECT id, player_id FROM units WHERE x = ? AND y = ? AND id != ?
//         "#)
//         .bind(order.to_x)
//         .bind(order.to_y)
//         .bind(order.unit_id)
//     .fetch_all(&db)
//     .await?;

//     if !others.is_empty() {
//         // get this unit's player
//         let self_player: (i64,) = sqlx::query_as(
//             "SELECT player_id FROM units WHERE id = ?")
//         .bind(order.unit_id)
//             .fetch_one(&db)
//             .await?;

//         for other in others {
//             let event = EncounterEvent {
//                 r#type: "encounter".to_string(),
//                 player_a: self_player.0,
//                 player_b: other.player_id,
//                 x: order.to_x,
//                 y: order.to_y,
//             };
//             let json = serde_json::to_string(&event)?;
//             // broadcast; ignoring if no listeners
//             let _ = tx.send(json);
//             tracing::info!("encounter event broadcast");
//         }
//     }

//     Ok(())
// }

// // helper to extract column safely in process_arrival (SqliteRow access)
// use sqlx::Row;



// ##########################################  hash generation ##################################
// see --https://chatgpt.com/c/6945cd39-b0e4-8327-bafb-a8d62d309825

// use xxhash_rust::xxh3::xxh3_64;


// const GENERATION_VERSION: u64 = 1;
// const WORLD_SEED: u64 = 1;

// // Galaxy
// const GALAXY_TAG: u64 = 1;

// // Star system
// const SYSTEM_TAG: u64 = 10;
// const SYSTEM_EXISTS_TAG: u64 = 11;
// const STAR_TYPE_TAG: u64 = 12;
// const PLANET_COUNT_TAG: u64 = 13;

// // Planet
// const PLANET_TAG: u64 = 20;
// const PLANET_TYPE_TAG: u64 = 21;

// // Tile
// const TILE_TAG: u64 = 100;
// const TILE_HEIGHT_TAG: u64 = 101;
// const TILE_BIOME_TAG: u64 = 102;



// fn hash64(values: &[u64]) -> u64 {
//     let mut bytes = Vec::new();
//     for v in values {
//         bytes.extend_from_slice(&v.to_le_bytes());
//     }
//     xxh3_64(&bytes)
// }

// planet_count = hash_values(&[
//     GENERATION_VERSION,
//     WORLD_SEED,
//     PLANET_COUNT_TAG,
// ]);

// worldSeed = 12345

// galaxySeed = 12345
// systemSeed = hash(12345, 10, -3, 7)
// planetSeed = hash(systemSeed, "planet", 2)
// tileSeed   = hash(planetSeed, 4, 12, 9)

// height     = noise(hash(tileSeed, "height"))


// system_seed = hash(
//     galaxy_seed,   // parent
//     SYSTEM_TAG,    // what this is
//     gx, gy, gz     // which system
// );

// planet_seed = hash(
//     system_seed,
//     PLANET_TAG,    // same for all planets
//     orbit_index    // distinguishes them
// );

// tile_seed = hash(
//     planet_seed,
//     TILE_TAG,      // same for all tiles
//     face, u, v     // distinguishes them
// );

// system_seed = hash(42, 2, 10, -3, 7)

// let exists = hash(system_seed, EXISTS_TAG) % 100 < 30;
// let star_type = hash(system_seed, STAR_TYPE_TAG) % 5;
// let planet_count = hash(system_seed, PLANET_COUNT_TAG) % 12;

// ######################### hash for Objects in the Star System ###################
//
// const SYSTEM_OBJECT_COUNT_TAG: u64 = 200;
// const SYSTEM_OBJECT_TYPE_TAG:  u64 = 201;
// const SYSTEM_OBJECT_POS_TAG:   u64 = 202;
// Then:

// rust
// Copier le code
// let object_count =
//     hash(system_seed, SYSTEM_OBJECT_COUNT_TAG) % 20;

// for i in 0..object_count {
//     let object_seed =
//         hash(system_seed, SYSTEM_OBJECT_TAG, i as u64);

//     let object_type =
//         hash(object_seed, SYSTEM_OBJECT_TYPE_TAG) % NUM_TYPES;

//     let position =
//         hash(object_seed, SYSTEM_OBJECT_POS_TAG);
// }