// // services/planet.rs
// pub async fn get_planet(pool: &SqlitePool, player_id: i64, planet_id: i64)
//     -> anyhow::Result<PlanetResponse>
// {
//     let planet = db::planet::find(pool, planet_id).await?;
//     let resources = db::resources::find(pool, planet_id).await?;
//     let computed = production::compute(&resources);  // pure game logic

//     Ok(PlanetResponse::from_parts(planet, computed)) // db → dto
// }
//
//
//
//
// here is another exemple for Tick Production
//
// // services/resource_tick.rs
// pub async fn process_production_tick(db_pool: &PgPool) -> Result<(), Error> {
//     // 1. Service layer handles the DB I/O
//     let mut transaction = db_pool.begin().await?;
//     let active_planets = repository::get_all_active_planets(&mut transaction).await?;

//     for mut db_planet in active_planets {
//         // 2. Convert DB model to pure Game model if necessary,
//         //    then pass it to the pure engine for the math
//         let elapsed_ticks = 1;
//         let production_result = game::production::calculate_yield(&db_planet, elapsed_ticks);

//         // 3. Service applies the result to our tracking and updates the DB
//         db_planet.metal += production_result.metal_gained;
//         repository::save_planet_resources(&mut transaction, &db_planet).await?;
//     }

//     transaction.commit().await?;
//     Ok(())
// }
