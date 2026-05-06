// // game/production.rs
// pub fn compute_resources(snapshot: &PlanetResources, now: i64) -> ResourceState {
//     let elapsed_hours = (now - snapshot.last_updated) as f64 / 3600.0;
//     ResourceState {
//         metal: snapshot.metal + snapshot.metal_rate * elapsed_hours,
//         // ...
//     }
// }