pub async fn run(state: Arc<AppState>) {
    loop {
        let orders = db::queries::due_orders(&state.db).await;
        for order in orders {
            let state = state.clone();
            tokio::spawn(async move {
                let _ = arrivals::process(order, state).await;
            });
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}