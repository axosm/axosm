

// Option A — Feature flags (cleanest for your case)

// # Cargo.toml
// [features]
// local = ["sqlx/sqlite"]
// prod = ["sqlx/postgres"]

#[cfg(feature = "local")]
async fn find_units_near(pos: Vec3) -> Vec<Unit> {
    sqlx::query_as!(Unit, "SELECT * FROM units WHERE ...bounding box...")
}

#[cfg(feature = "prod")]
async fn find_units_near(pos: Vec3) -> Vec<Unit> {
    sqlx::query_as!(Unit, "SELECT * FROM units WHERE ST_DWithin(...)")
    // WHERE system_id = ? AND ST_DWithin(space_pos, ...) AND location_mode = 'in_space'
}

// Option B — Runtime DB abstraction via trait

// async fn process_combat(repo: &dyn UnitRepository, attacker_id: i64) {
//     let nearby = repo.find_near(system_id, pos, radius).await?;
    
//     // combat logic...
// }

#[async_trait]
pub trait UnitRepository {
    async fn find_near(&self, system_id: i64, pos: Vec3, radius: f64) -> Result<Vec<Unit>>;
    async fn find_by_player(&self, player_id: i64) -> Result<Vec<Unit>>;
}

pub struct SqliteUnitRepo(SqlitePool);
pub struct PostgresUnitRepo(PgPool);

impl UnitRepository for SqliteUnitRepo { ... }
impl UnitRepository for PostgresUnitRepo { ... }





// For unit tests

// struct MockUnitRepo {
//     units: Vec<Unit>
// }

// impl UnitRepository for MockUnitRepo {
//     async fn find_near(&self, ...) -> Result<Vec<Unit>> {
//         Ok(self.units.clone()) // just return whatever you want
//     }
// }

// // In tests — no DB needed at all
// #[tokio::test]
// async fn test_combat() {
//     let repo = MockUnitRepo { units: vec![...] };
//     process_combat(&repo, 1).await;
// }