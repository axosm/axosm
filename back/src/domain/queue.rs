 
 
//  pub async fn check_and_emit(state: Arc<AppState>) {
//  // check for arrived move orders
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

//         }