// // services/planet.rs
// pub async fn get_planet(pool: &SqlitePool, player_id: i64, planet_id: i64) 
//     -> anyhow::Result<PlanetResponse> 
// {
//     let planet = db::planet::find(pool, planet_id).await?;
//     let resources = db::resources::find(pool, planet_id).await?;
//     let computed = production::compute(&resources);  // pure game logic

//     Ok(PlanetResponse::from_parts(planet, computed)) // db → dto
// }