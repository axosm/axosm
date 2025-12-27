Yes—what you are running into is a common pain point in Rust when using **SQLx**: you have a database “projection” (`UnitWithPlanetLocationRow`) that doesn’t exactly match your API DTO (`Unit`). You manually implement `From` or `TryFrom` to map between them. There are several ways to reduce or eliminate that boilerplate, each with trade-offs. Let’s go through them.

---

## 1. Use `serde` + SQL query aliasing

If your **SQL query exactly matches your DTO fields**, you can skip the intermediate `Row` type.

Example:

```rust
#[derive(serde::Serialize, sqlx::FromRow)]
pub struct Unit {
    pub id: i64,
    pub player_id: i64,
    pub unit_type: String,
    pub location_type: String,
    pub planet_id: Option<i64>,
    pub face: Option<i32>,
    pub u: Option<i32>,
    pub v: Option<i32>,
}
```

Then your query can be:

```rust
let units: Vec<Unit> = sqlx::query_as(
    r#"
    SELECT
        u.id,
        u.player_id,
        u.unit_type,
        u.location_type,
        pl.planet_id,
        pl.face,
        pl.u,
        pl.v
    FROM units u
    LEFT JOIN unit_planet_locations pl
        ON pl.unit_id = u.id
    "#
)
.fetch_all(&state.db)
.await?;
```

No `From` impl is needed because `sqlx::FromRow` maps directly.

✅ **Limitation**: The struct must exactly match the query column names and types.

---

## 2. Use a procedural macro / mapping library

There are crates like:

* [`derive_more`](https://crates.io/crates/derive_more)
* [`typed-builder`](https://crates.io/crates/typed-builder)
* [`mapper`](https://crates.io/crates/mapper)

They let you auto-generate `From`/`Into` implementations. Example with `derive_more`:

```rust
#[derive(derive_more::From)]
struct Unit(UnitWithPlanetLocationRow);
```

✅ Reduces boilerplate but adds a dependency and you still need a struct per mapping.

---

## 3. Switch to Diesel ORM

Diesel encourages **“domain structs”** that match tables directly:

```rust
#[derive(Queryable, Serialize)]
struct Unit {
    id: i32,
    player_id: i32,
    unit_type: String,
    location_type: String,
}
```

* Diesel automatically maps DB rows to structs
* You can use **associations** to preload related tables
* You can skip manual `From` if your structs match the DB shape

✅ **Pros**: less boilerplate if your DB schema is stable
❌ **Cons**: Less flexible for complex SQL queries or joins; compile times longer

---

## 4. Build DTOs directly from the query (flatten the mapping)

Instead of returning `UnitWithPlanetLocationRow` from the DB layer, return a **fully constructed DTO** from the query function:

```rust
pub async fn find_units(db: &SqlitePool, player_id: i64) -> Result<Vec<Unit>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            u.id,
            u.player_id,
            u.unit_type,
            u.location_type,
            pl.planet_id,
            pl.face,
            pl.u,
            pl.v
        FROM units u
        LEFT JOIN unit_planet_locations pl
            ON pl.unit_id = u.id
        WHERE u.player_id = ?
        "#,
        player_id
    )
    .fetch_all(db)
    .await?;

    let units = rows.into_iter().map(|r| Unit {
        id: r.id,
        player_id: r.player_id,
        unit_type: r.unit_type,
        location: match r.location_type.as_str() {
            "PLANET_SURFACE" => UnitLocation::PLANET_SURFACE {
                planet_id: r.planet_id.unwrap(),
                face: r.face.unwrap(),
                u: r.u.unwrap(),
                v: r.v.unwrap(),
            },
            "ORBIT" => UnitLocation::ORBIT,
            "SPACE" => UnitLocation::SPACE,
            _ => unreachable!(),
        },
    }).collect();

    Ok(units)
}
```

✅ The handler no longer needs any mapping; the DB layer returns `Vec<Unit>` directly.

---

## 5. Summary of options

| Approach                          | Pros                       | Cons                                                   |
| --------------------------------- | -------------------------- | ------------------------------------------------------ |
| `FromRow` on DTO                  | Minimal boilerplate        | Query must exactly match DTO                           |
| Procedural macros (`derive_more`) | Auto-generate `From`       | Adds dependency, less explicit                         |
| Diesel ORM                        | Auto-mapping, associations | Less flexible for complex SQL, compile times           |
| Build DTO in query function       | No mapping in handler      | Slightly less layered; handler cannot reuse DB structs |

---

**Recommendation:**

* If your DTO **mostly matches query columns**, just use `sqlx::FromRow` on the DTO.
* If you have **complex mapping logic** (like `location_type` → enum), either leave a `From` impl or **build the DTO in the DB function**, avoiding the extra mapping step in the handler.

---

If you want, I can rewrite your **`find_units` function** so that it **directly returns `Vec<Unit>`** with the location enum, so you never need `From<UnitWithPlanetLocationRow> for Unit`. This completely removes that boilerplate.

Do you want me to do that?
