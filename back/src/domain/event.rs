pub async fn check_and_emit(
    db: &SqlitePool,
    tx: &Sender<String>,
    order: &MoveOrder,
) -> anyhow::Result<()> {
    let encounters = db::queries::find_encounters(db, order).await?;
    for e in encounters {
        let json = serde_json::to_string(&e)?;
        let _ = tx.send(json);
    }
    Ok(())
}