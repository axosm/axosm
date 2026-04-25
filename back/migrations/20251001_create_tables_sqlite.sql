

-- 20251001_create_tables.sql


PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ─────────────────────────────────────────────────────────────
-- 1. USERS & AUTHENTICATION
-- ─────────────────────────────────────────────────────────────

CREATE TABLE players (
  id             INTEGER  PRIMARY KEY AUTOINCREMENT,
  username       TEXT     NOT NULL,
  email          TEXT     NOT NULL UNIQUE,
  password_hash  TEXT     NOT NULL,
  created_at     TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at     TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  last_login_at  TEXT
);

CREATE TABLE empires (
  id          INTEGER  PRIMARY KEY AUTOINCREMENT,
  name        TEXT     NOT NULL UNIQUE,
  created_by  INTEGER  NOT NULL REFERENCES players(id),
  created_at  TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE empire_members (
  empire_id   INTEGER  NOT NULL REFERENCES empires(id),
  player_id   INTEGER  NOT NULL REFERENCES players(id),
  role        TEXT     NOT NULL DEFAULT 'member'
              CHECK(role IN ('leader','officer','member')),
  joined_at   TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  PRIMARY KEY (empire_id, player_id)
);

-- ─────────────────────────────────────────────────────────────
-- 2. UNIVERSE & SPACE
-- ─────────────────────────────────────────────────────────────

CREATE TABLE galaxies (
  id          INTEGER  PRIMARY KEY AUTOINCREMENT,
  seed        INTEGER  NOT NULL,
  x           REAL     NOT NULL,
  y           REAL     NOT NULL,
  z           REAL     NOT NULL,
  created_at  TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at  TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE(x, y, z)
);

CREATE TABLE star_systems (
  id          INTEGER  PRIMARY KEY AUTOINCREMENT,
  galaxy_id   INTEGER  NOT NULL REFERENCES galaxies(id),
  seed        INTEGER  NOT NULL,
  x           INTEGER  NOT NULL,
  y           INTEGER  NOT NULL,
  z           INTEGER  NOT NULL,
  created_at  TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at  TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE(galaxy_id, x, y, z)
);

CREATE INDEX idx_star_systems_galaxy ON star_systems(galaxy_id);

CREATE TABLE planets (
  id              INTEGER  PRIMARY KEY AUTOINCREMENT,
  star_system_id  INTEGER  NOT NULL REFERENCES star_systems(id),
  seed            INTEGER  NOT NULL,
  x               REAL     NOT NULL,
  y               REAL     NOT NULL,
  subdivision     INTEGER  NOT NULL, -- Goldberg polyhedron resolution (N)
  created_at      TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at      TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_planets_system ON planets(star_system_id);

-- Dynamic space objects (asteroids, pirate bases, events — not seeded)
CREATE TABLE space_objects (
  id             INTEGER  PRIMARY KEY AUTOINCREMENT,
  star_system_id INTEGER  NOT NULL REFERENCES star_systems(id),
  object_type    TEXT     NOT NULL
                 CHECK(object_type IN ('asteroid','pirate_base','anomaly','wreck','event')),
  x              REAL     NOT NULL,
  y              REAL     NOT NULL,
  z              REAL     NOT NULL DEFAULT 0,
  properties     TEXT,                          -- JSON: hp, loot table, difficulty, etc.
  spawned_at     TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  despawned_at   TEXT                           -- NULL = still active
);

CREATE INDEX idx_space_objects_system ON space_objects(star_system_id);



-- ─────────────────────────────────────────────────────────────
-- 3. TILES
-- ─────────────────────────────────────────────────────────────

-- One row per tile on a planet surface.
-- Tiles are generated lazily as players explore — not all at startup.
CREATE TABLE planet_tiles (
  id                       INTEGER  PRIMARY KEY AUTOINCREMENT,
  planet_id                INTEGER  NOT NULL REFERENCES planets(id),
  face                     INTEGER  NOT NULL, -- Goldberg face index
  u                        INTEGER  NOT NULL,
  v                        INTEGER  NOT NULL,

  -- Terrain
  tile_type                TEXT     NOT NULL
                           CHECK(tile_type IN (
                             'plains','forest','mountain','desert',
                             'snow','lava','water','ocean'
                           )),
  -- Seeded yield quality: 0.0 (poor) to 1.0 (rich).
  -- Multiplied against building base output to get actual production.
  yield_quality            REAL     NOT NULL DEFAULT 0.5,

  -- Rare deposit present on this tile (NULL = none).
  -- Unlocked by tech era — a coal deposit does nothing until industrial era.
  rare_deposit             TEXT
                           CHECK(rare_deposit IN (
                             'coal','iron','gold','gems','petrol','uranium',
                             'rare_earths','silicon','deuterium','dark_matter',
                             NULL
                           )),

  -- Ownership driven by influence recalc — NULL = unclaimed wilderness
  owner_player_id          INTEGER  REFERENCES players(id),

  -- Dirty flag: set to 1 when a nearby building is created/destroyed/repaired.
  -- Background job recalculates influence for all flagged tiles.
  influence_recalc_needed  INTEGER  NOT NULL DEFAULT 0,

  created_at               TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at               TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),

  UNIQUE(planet_id, face, u, v)
);

CREATE INDEX idx_tiles_owner        ON planet_tiles(owner_player_id) WHERE owner_player_id IS NOT NULL;
CREATE INDEX idx_tiles_recalc       ON planet_tiles(influence_recalc_needed) WHERE influence_recalc_needed = 1;


-- ─────────────────────────────────────────────────────────────
-- 4. INFLUENCE
-- ─────────────────────────────────────────────────────────────

-- Per-tile per-player influence score.
-- Score = SUM over all player's buildings of (influence_power / distance)
-- capped at influence_radius.
-- Owner of a tile = player with highest score.
-- Recalculated by background job when influence_recalc_needed = 1.
CREATE TABLE tile_influence (
  tile_id    INTEGER  NOT NULL REFERENCES planet_tiles(id) ON DELETE CASCADE,
  player_id  INTEGER  NOT NULL REFERENCES players(id)      ON DELETE CASCADE,
  score      REAL     NOT NULL DEFAULT 0,
  updated_at TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  PRIMARY KEY (tile_id, player_id)
);

CREATE INDEX idx_influence_tile   ON tile_influence(tile_id);
CREATE INDEX idx_influence_player ON tile_influence(player_id);



-- ─────────────────────────────────────────────────────────────
-- 5. BUILDINGS
-- ─────────────────────────────────────────────────────────────


-- I server code
-- -- Static config per building type. Stored in DB so game logic can query it.
-- CREATE TABLE building_types (
--   unit_type                TEXT     PRIMARY KEY,
--   era                      TEXT     NOT NULL
--                            CHECK(era IN ('stone','industrial','modern','space')),

--   -- Tile placement constraints
--   valid_tile_types         TEXT     NOT NULL, -- JSON array e.g. ["forest","plains"]

--   -- Construction
--   build_cost_json          TEXT     NOT NULL, -- JSON: {"wood":100,"stone":50}
--   build_time_ticks         INTEGER  NOT NULL,

--   -- HP & combat
--   base_hp                  INTEGER  NOT NULL,
--   defence_value            REAL     NOT NULL DEFAULT 0, -- damage dealt to attacker per tick
--   city_wide_defence_bonus  REAL     NOT NULL DEFAULT 0, -- added to all friendly tile battles in city
--   auto_repair_hp_per_tick  REAL     NOT NULL DEFAULT 0.1,

--   -- Resource production (passive, per tick, at level 1 — scales with level)
--   produces_resource        TEXT,              -- NULL = no passive output
--   base_output_per_tick     REAL     NOT NULL DEFAULT 0,

--   -- Storage (for warehouse/storage buildings)
--   storage_capacity_json    TEXT,             -- JSON: {"wood":5000,"stone":5000} or NULL

--   -- Influence
--   influence_power          REAL     NOT NULL DEFAULT 0,
--   influence_radius         INTEGER  NOT NULL DEFAULT 0,

--   -- Flying buildings (Starcraft-style liftoff)
--   can_fly                  INTEGER  NOT NULL DEFAULT 0,
--   fly_speed_tiles_per_tick REAL,
--   liftoff_ticks            INTEGER  NOT NULL DEFAULT 0,
--   land_cooldown_ticks      INTEGER  NOT NULL DEFAULT 0  -- ticks before usable after landing
-- );

-- Per-instance building on a tile.
-- One building per tile enforced by UNIQUE(tile_id).
CREATE TABLE buildings (
  id                   INTEGER  PRIMARY KEY AUTOINCREMENT,
  player_id            INTEGER  NOT NULL REFERENCES players(id),
  building_type        TEXT     NOT NULL,
  tile_id              INTEGER  NOT NULL REFERENCES planet_tiles(id),
  level                INTEGER  NOT NULL DEFAULT 1,

  -- HP
  hp                   INTEGER  NOT NULL,
  max_hp               INTEGER  NOT NULL, -- = building_types.base_hp × level (denormalised for perf)
  under_attack         INTEGER  NOT NULL DEFAULT 0,  -- 1 = repair paused
  destroyed_at         TEXT,                         -- NULL = standing

  -- Flying buildings
  can_fly              INTEGER  NOT NULL DEFAULT 0,
  flight_state         TEXT
                       CHECK(flight_state IN ('grounded','lifting_off','flying','landing')),

  -- Construction queue (NULL = already built)
  construction_done_at TEXT,

  created_at           TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at           TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),

  -- One building per tile
  UNIQUE(tile_id)
);

CREATE INDEX idx_buildings_player      ON buildings(player_id);
CREATE INDEX idx_buildings_tile        ON buildings(tile_id);
CREATE INDEX idx_buildings_under_attack ON buildings(under_attack) WHERE under_attack = 1;
CREATE INDEX idx_buildings_destroyed   ON buildings(destroyed_at)  WHERE destroyed_at IS NOT NULL;
CREATE INDEX idx_buildings_flying      ON buildings(flight_state)  WHERE flight_state != 'grounded';


-- ─────────────────────────────────────────────────────────────
-- 6. UNITS & MOVEMENT
-- ─────────────────────────────────────────────────────────────


-- aaaaaa TODO review everything below
-- see https://claude.ai/share/f3a79932-cf06-4638-acae-f0213bbf423a

-- In server code
-- Static config per unit type
-- CREATE TABLE unit_types (
--   unit_type              TEXT  PRIMARY KEY,
--   era                    TEXT  NOT NULL
--                          CHECK(era IN ('stone','industrial','modern','space')),
--   attack                 REAL  NOT NULL,
--   defence                REAL  NOT NULL,
--   hp_per_individual      REAL  NOT NULL,
--   speed_tiles_per_tick   REAL  NOT NULL,
--   carry_capacity         INTEGER NOT NULL DEFAULT 0, -- resources this unit can carry when looting
--   -- Type matchup bonuses: JSON map of target unit_type → multiplier
--   -- e.g. {"swordsman": 1.5, "cavalry": 0.7}
--   combat_bonuses_json    TEXT  NOT NULL DEFAULT '{}',
--   recruit_cost_json      TEXT  NOT NULL,             -- JSON: {"wood":10,"stone":5}
--   recruit_time_ticks     INTEGER NOT NULL
-- );


CREATE TABLE units (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  unit_type TEXT NOT NULL,
  is_squad       INTEGER  NOT NULL DEFAULT 0,  -- 1 = multiple individuals grouped
  count          INTEGER  NOT NULL DEFAULT 1,  -- number of individuals in formation
  hp INTEGER NOT NULL,
  player_id INTEGER NOT NULL,

  -- Combat state
  in_battle      INTEGER  NOT NULL DEFAULT 0,  -- 1 = currently fighting on a tile

  -- Location
  location_mode   TEXT NOT NULL DEFAULT 'planet_surface'
                  CHECK(location_mode IN (
                    'planet_surface', 'in_orbit', 'in_space', 'embarked'
                  )),

  -- Surface position (when location_mode = 'planet_surface')
  planet_id       INTEGER REFERENCES planets(id),
  planet_face     INTEGER,
  planet_u          INTEGER,
  planet_v          INTEGER,

  -- Orbit (when location_mode = 'in_orbit')
  orbit_planet_id INTEGER REFERENCES planets(id),

  -- Deep space (when location_mode = 'in_space')
  star_system_id     INTEGER REFERENCES star_systems(id),
  -- star_system_posZ GEOMETRY(PointZ, 0), -- need Postgres extension PostGIS
  star_system_x         REAL,
  star_system_y         REAL,
  star_system_z         REAL DEFAULT 0,

  customization JSON,
  FOREIGN KEY (player_id) REFERENCES players(id)
);

CREATE INDEX idx_units_player ON units(player_id);
CREATE INDEX idx_units_planet ON units(planet_id, planet_face, planet_u, planet_v)  WHERE location_mode = 'planet_surface';
CREATE INDEX idx_units_orbit ON units(orbit_planet_id)  WHERE location_mode = 'in_orbit';
CREATE INDEX idx_units_star_system ON units(star_system_x, star_system_y, star_system_z);


-- -- This stores all three coordinates in one column
-- UPDATE units SET space_pos = ST_MakePoint(12.5, 34.2, 7.8)
-- WHERE id = ?;

-- -- "find all units within radius 100 of position (10, 30, 5)"
-- SELECT * FROM units
-- WHERE system_id = ? AND ST_DWithin(space_pos, ST_MakePoint(10, 30, 5), 100);


CREATE TABLE move_orders (
  id INTEGER PRIMARY KEY,

  -- Exactly one of these must be set

  unit_id         INTEGER  REFERENCES units(id),
  building_id     INTEGER  REFERENCES buildings(id),
  mover_type      TEXT     NOT NULL CHECK(mover_type IN ('unit','building')),

  move_type       TEXT     NOT NULL CHECK(move_type IN (
                    'tile_walk',
                    'launch_to_orbit',
                    'orbit_to_space',
                    'space_travel',
                    'enter_orbit',
                    'land',
                    'building_liftoff',
                    'building_land',
                    'loot_and_retreat'   -- triggers loot then auto-generates retreat move_order
                  )),
  -- Surface origin
  from_planet_id   INTEGER  REFERENCES planets(id),
  from_planet_face INTEGER,
  from_planet_u    INTEGER,
  from_planet_v    INTEGER,

  -- Surface destination
  to_planet_id     INTEGER  REFERENCES planets(id),
  to_planet_face   INTEGER,
  to_planet_u      INTEGER,
  to_planet_v      INTEGER,

  -- Space origin / destination (for space_travel)
  from_star_system_id  INTEGER  REFERENCES star_systems(id),
  from_space_x         REAL,
  from_space_y         REAL,
  from_space_z         REAL,

  to_star_system_id    INTEGER  REFERENCES star_systems(id),
  to_space_x           REAL,
  to_space_y           REAL,
  to_space_z           REAL,

  start_time INTEGER NOT NULL,
  arrival_time INTEGER NOT NULL,
  --   status            TEXT NOT NULL DEFAULT 'in_transit'
  --                   CHECK(status IN ('in_transit', 'arrived', 'recalled', 'cancelled')),
  -- callback_json     TEXT

  FOREIGN KEY(unit_id) REFERENCES units(id)
);


CREATE INDEX idx_move_orders_unit     ON move_orders(unit_id)     WHERE unit_id IS NOT NULL;
CREATE INDEX idx_move_orders_building ON move_orders(building_id) WHERE building_id IS NOT NULL;
CREATE INDEX idx_move_orders_arrival  ON move_orders(arrival_time);

-- Retreat orders: created by player during combat, consumed next tick.
CREATE TABLE retreat_orders (
  id              INTEGER  PRIMARY KEY AUTOINCREMENT,
  unit_id         INTEGER  NOT NULL REFERENCES units(id) ON DELETE CASCADE,
  player_id       INTEGER  NOT NULL REFERENCES players(id),
  retreat_to_face INTEGER  NOT NULL,
  retreat_to_u    INTEGER  NOT NULL,
  retreat_to_v    INTEGER  NOT NULL,
  created_at      TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_retreat_unit   ON retreat_orders(unit_id);
CREATE INDEX idx_retreat_player ON retreat_orders(player_id);


-- ─────────────────────────────────────────────────────────────
-- 7. COMBAT
-- ─────────────────────────────────────────────────────────────

-- Active battles — one row per contested tile (or space coordinate).
-- Created when opposing units meet. Deleted when resolved.
CREATE TABLE battles (
  id               INTEGER  PRIMARY KEY AUTOINCREMENT,

  -- Location: exactly one of planet tile or space coordinate
  tile_id          INTEGER  REFERENCES planet_tiles(id),
  star_system_id   INTEGER  REFERENCES star_systems(id),
  space_x          REAL,
  space_y          REAL,
  space_z          REAL,

  -- Participants (attacker = player who moved onto the tile)
  attacker_id      INTEGER  NOT NULL REFERENCES players(id),
  defender_id      INTEGER  NOT NULL REFERENCES players(id),

  -- Phase
  phase            TEXT     NOT NULL DEFAULT 'vs_units'
                   CHECK(phase IN ('vs_units','vs_building')),
                   -- vs_units:    attacker fights defender garrison
                   -- vs_building: garrison dead, attacker fights building HP

  started_at       TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  last_tick_at     TEXT
);

CREATE INDEX idx_battles_tile      ON battles(tile_id)        WHERE tile_id IS NOT NULL;
CREATE INDEX idx_battles_attacker  ON battles(attacker_id);
CREATE INDEX idx_battles_defender  ON battles(defender_id);

-- Permanent record of every resolved battle.
CREATE TABLE battle_reports (
  id               INTEGER  PRIMARY KEY AUTOINCREMENT,
  battle_id        INTEGER  NOT NULL, -- original battles.id (kept after deletion)

  -- Location
  tile_id          INTEGER  REFERENCES planet_tiles(id),
  star_system_id   INTEGER  REFERENCES star_systems(id),
  space_x          REAL,
  space_y          REAL,
  space_z          REAL,

  attacker_id      INTEGER  NOT NULL REFERENCES players(id),
  defender_id      INTEGER  NOT NULL REFERENCES players(id),

  outcome          TEXT     NOT NULL
                   CHECK(outcome IN (
                     'attacker_victory',
                     'defender_victory',
                     'attacker_retreated',
                     'defender_retreated',
                     'attacker_looted',   -- garrison killed, loot taken, attacker left
                     'draw'
                   )),

  -- Snapshot of units involved (JSON) — preserved even after units are deleted
  -- e.g. [{"unit_type":"archer","sent":50,"lost":12}, ...]
  attacker_units_snapshot  TEXT  NOT NULL,
  defender_units_snapshot  TEXT  NOT NULL,

  -- Resources looted (only set when outcome = 'attacker_looted')
  resources_looted_json    TEXT,

  started_at       TEXT     NOT NULL,
  ended_at         TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),

  -- Notifications: 0 = not yet read by that player
  attacker_read    INTEGER  NOT NULL DEFAULT 0,
  defender_read    INTEGER  NOT NULL DEFAULT 0
);

CREATE INDEX idx_battle_reports_attacker ON battle_reports(attacker_id, attacker_read);
CREATE INDEX idx_battle_reports_defender ON battle_reports(defender_id, defender_read);
CREATE INDEX idx_battle_reports_tile     ON battle_reports(tile_id) WHERE tile_id IS NOT NULL;

-- ─────────────────────────────────────────────────────────────
-- 8. RESOURCES
-- ─────────────────────────────────────────────────────────────

-- Pooled resource inventory per player.
-- All buildings on all planets feed into this single pool (OGame-style).
-- Add planet_id FK here later if you want per-planet pools.
CREATE TABLE player_resources (
  player_id      INTEGER  NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  resource_type  TEXT     NOT NULL
                 CHECK(resource_type IN (
                   -- Stone age
                   'wood','stone','food','water',
                   -- Industrial
                   'coal','iron','petrol','copper',
                   -- Modern
                   'silicon','uranium','rare_earths','electricity',
                   -- Space
                   'deuterium','dark_matter','titanium','antimatter'
                 )),
  amount         REAL     NOT NULL DEFAULT 0,
  -- Hard cap — enforced by game logic, not DB constraint.
  -- Determined by sum of player's storage buildings.
  cap            REAL     NOT NULL DEFAULT 1000,
  updated_at     TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  PRIMARY KEY (player_id, resource_type)
);

CREATE INDEX idx_player_resources_player ON player_resources(player_id);

-- Resources carried by a unit formation (set during loot_and_retreat)
CREATE TABLE unit_cargo (
  unit_id        INTEGER  NOT NULL REFERENCES units(id) ON DELETE CASCADE,
  resource_type  TEXT     NOT NULL,
  amount         REAL     NOT NULL DEFAULT 0,
  PRIMARY KEY (unit_id, resource_type)
);

-- aaaa see Squad based or single unit control, which one do you prefer ? 
-- https://www.reddit.com/r/RealTimeStrategy/comments/182q4o3/squad_based_or_single_unit_control_which_one_do/

-- PostGIS
-- https://claude.ai/share/94821517-87e4-4601-9537-4ded5073e2ca
-- learn more about :
-- - Slash of clan defense/battle system ("how buildings have implicit HP and act as defenders")
-- - Settlers of Catan rules

-- schema / city battles / influence
-- https://claude.ai/share/b25daaf0-c880-4e4d-986a-3717515382ef


-- CREATE TABLE formations (
--   id INTEGER PRIMARY KEY AUTOINCREMENT,
--   name TEXT,  -- e.g., "Formation Alpha"
--   player_id INTEGER NOT NULL,
--   transport_id INTEGER REFERENCES units(id),  -- Carrier ship or transport vehicle
--   FOREIGN KEY (player_id) REFERENCES players(id)
-- );

-- CREATE TABLE formation_units (
--   formation_id INTEGER NOT NULL,
--   unit_id INTEGER NOT NULL,
--   PRIMARY KEY (formation_id, unit_id),
--   FOREIGN KEY (formation_id) REFERENCES formations(id),
--   FOREIGN KEY (unit_id) REFERENCES units(id)
-- );

-- CREATE TABLE unit_transport (
--   transport_id INTEGER NOT NULL,  -- ID of the carrier (individual unit)
--   unit_id INTEGER NOT NULL,        -- ID of the transported unit/squad
--   PRIMARY KEY (transport_id, unit_id),
--   FOREIGN KEY (transport_id) REFERENCES units(id),
--   FOREIGN KEY (unit_id) REFERENCES units(id)
-- );






-- -- A squad: pure headcount, no construction, no modules
-- -- unit_type limited to ground-only types
-- CREATE TABLE squad (
--   id            INTEGER PRIMARY KEY,
--   player_id     INTEGER NOT NULL REFERENCES player(id),
--   unit_type     TEXT NOT NULL CHECK(unit_type IN ('marine', 'tank', 'artillery')),
--   count         INTEGER NOT NULL DEFAULT 100,  -- this IS the hp
--   planet_id     INTEGER REFERENCES planet(id),
--   tile_x        INTEGER,
--   tile_y        INTEGER,
--   carried_by    INTEGER REFERENCES vessel(id)  -- can be loaded into a vessel
-- );








-- CREATE TABLE units (
--   id INTEGER PRIMARY KEY,
--   player_id INTEGER NOT NULL,
--   unit_type TEXT NOT NULL,
--   location_type TEXT NOT NULL, -- 'PLANET_SURFACE', 'ORBIT', 'SPACE'
--   FOREIGN KEY(player_id) REFERENCES players(id)
-- );






-- -- One table for everything: troops, ships, hybrids like the Zoc
-- -- location_mode: 'planet_surface' | 'in_orbit' | 'in_space' | 'embarked'
-- CREATE TABLE unit (
--   id               INTEGER PRIMARY KEY,
--   player_id        INTEGER NOT NULL REFERENCES player(id),
--   template_id      INTEGER NOT NULL REFERENCES entity_template(id),
--   name             TEXT,

--   -- Current location mode
--   location_mode    TEXT NOT NULL DEFAULT 'planet_surface',

--   -- Planet surface coords (location_mode = 'planet_surface')
--   planet_id        INTEGER REFERENCES planet(id),
--   tile_x           INTEGER,
--   tile_y           INTEGER,

--   -- Orbit (location_mode = 'in_orbit')
--   orbit_planet_id  INTEGER REFERENCES planet(id),

--   -- Deep space coords (location_mode = 'in_space')
--   space_x          REAL,
--   space_y          REAL,
--   space_z          REAL DEFAULT 0,

--   -- Embarked inside a carrier (location_mode = 'embarked')
--   carried_by       INTEGER REFERENCES unit(id),  -- self-referential!

--   hp               INTEGER NOT NULL DEFAULT 100
-- );






-- -- old schema
-- -- CREATE TABLE units (
-- --   id INTEGER PRIMARY KEY,
-- --   player_id INTEGER NOT NULL,
-- --   x INTEGER NOT NULL,
-- --   y INTEGER NOT NULL,
-- --   FOREIGN KEY(player_id) REFERENCES players(id)
-- -- );
-- CREATE TABLE units (
--   id INTEGER PRIMARY KEY,
--   player_id INTEGER NOT NULL,
--   unit_type TEXT NOT NULL,
--   location_type TEXT NOT NULL, -- 'PLANET_SURFACE', 'ORBIT', 'SPACE'
--   FOREIGN KEY(player_id) REFERENCES players(id)
-- );

-- CREATE TABLE unit_planet_locations (
--   unit_id INTEGER PRIMARY KEY,
--   planet_id INTEGER NOT NULL,
--   face INTEGER NOT NULL,
--   u INTEGER NOT NULL,
--   v INTEGER NOT NULL,
--   FOREIGN KEY(unit_id) REFERENCES units(id),
--   FOREIGN KEY(planet_id) REFERENCES planets(id)
-- );

-- -- old schema
-- -- CREATE TABLE move_orders (
-- --   id INTEGER PRIMARY KEY,
-- --   unit_id INTEGER NOT NULL,
-- --   from_x INTEGER NOT NULL,
-- --   from_y INTEGER NOT NULL,
-- --   to_x INTEGER NOT NULL,
-- --   to_y INTEGER NOT NULL,
-- --   arrival_time INTEGER NOT NULL,
-- --   FOREIGN KEY(unit_id) REFERENCES units(id)
-- -- );
-- CREATE TABLE move_orders (
--   id INTEGER PRIMARY KEY,
--   unit_id INTEGER NOT NULL,
--   from_face INTEGER NOT NULL,
--   from_u INTEGER NOT NULL,
--   from_v INTEGER NOT NULL,
--   to_face INTEGER NOT NULL,
--   to_u INTEGER NOT NULL,
--   to_v INTEGER NOT NULL,
--   start_time INTEGER NOT NULL,
--   arrival_time INTEGER NOT NULL,
--   FOREIGN KEY(unit_id) REFERENCES units(id)
-- );



-- -- Future: Orbital / Space Location (DO NOT USE YET)
-- -- When you are ready, you add this without touching existing tables.
-- -- CREATE TABLE unit_space_locations (
-- --   unit_id INTEGER PRIMARY KEY,
-- --   star_system_id INTEGER NOT NULL,
-- --   x REAL NOT NULL,
-- --   y REAL NOT NULL,
-- --   z REAL NOT NULL,
-- --   FOREIGN KEY(unit_id) REFERENCES units(id),
-- --   FOREIGN KEY(star_system_id) REFERENCES star_systems(id)
-- -- );


-- -- Why This Schema Works Long-Term
-- -- Planet → Space Transition
-- -- DELETE FROM unit_planet_locations
-- -- INSERT INTO unit_space_locations
-- -- UPDATE units.location_type = 'SPACE'


-- -- Interstellar Travel Later
-- -- You will add:
-- -- CREATE TABLE unit_ftl_travel (
-- --   unit_id INTEGER PRIMARY KEY,
-- --   from_system INTEGER NOT NULL,
-- --   to_system INTEGER NOT NULL,
-- --   progress REAL NOT NULL
-- -- );


-- -- Indexes You Should Add Early
-- -- CREATE INDEX idx_units_player ON units(player_id);
-- -- CREATE INDEX idx_planet_location_planet ON unit_planet_locations(planet_id);
-- -- CREATE INDEX idx_move_orders_unit ON move_orders(unit_id);


-- -- seed two players + units (for prototype)
-- -- INSERT OR IGNORE INTO players (id, name) VALUES (1, 'Player 1');
-- -- INSERT OR IGNORE INTO players (id, name) VALUES (2, 'Player 2');

-- -- place player 1 at (2,2) and player 2 at (5,5)
-- -- INSERT OR IGNORE INTO units (id, player_id, x, y) VALUES (1, 1, 2, 2);
-- -- INSERT OR IGNORE INTO units (id, player_id, x, y) VALUES (2, 2, 5, 5);


-- -- https://chatgpt.com/c/694dd80b-260c-8331-8f03-ce07eb2c7307
-- -- problem : If the client receives canonical coordinates (face, u, v), the client can reconstruct absolute planetary geography and infer the seed.
-- -- solution 1 (recommended): player-local coordinate frames
-- -- For each player–planet pair, define a local frame:
-- -- An arbitrary origin (face₀, u₀, v₀)
-- -- An arbitrary rotation / orientation
-- -- CREATE TABLE player_planet_frames (
-- --   player_id INTEGER NOT NULL,
-- --   planet_id INTEGER NOT NULL,
-- --   origin_face INTEGER NOT NULL,
-- --   origin_u INTEGER NOT NULL,
-- --   origin_v INTEGER NOT NULL,
-- --   rotation INTEGER NOT NULL, -- 0..5 for face orientation, or quaternion index
-- --   PRIMARY KEY (player_id, planet_id)
-- -- );


-- INSERT INTO players (id, name) VALUES
--   (1, 'Player One'),
--   (2, 'Player Two');


-- INSERT INTO galaxies (id, seed) VALUES
--   (1, 123456),
--   (2, 987654);


-- INSERT INTO star_systems (id, galaxy_id, gx, gy, gz, seed) VALUES
--   (1, 1, 0, 0, 0, 111),
--   (2, 1, 1, 0, 0, 222),
--   (3, 2, 0, 1, 0, 333);


-- INSERT INTO planets (
--   id,
--   star_system_id,
--   orbit_index,
--   radius,
--   subdivision,
--   seed
-- ) VALUES
--   (1, 1, 0, 6371, 3, 1001),
--   (2, 1, 1, 3390, 3, 1002),
--   (3, 2, 0, 7000, 4, 2001);


-- INSERT INTO units (id, player_id, unit_type, location_type) VALUES
--   (1, 1, 'INFANTRY', 'PLANET_SURFACE'),
--   (2, 1, 'TANK',     'PLANET_SURFACE'),
--   (3, 1, 'FLEET',    'ORBIT'),
--   (4, 1, 'SCOUT',    'SPACE');


-- INSERT INTO unit_planet_locations (unit_id, planet_id, face, u, v) VALUES
--   (1, 1, 0, 10, 5),
--   (2, 1, 2, 3, 7);



-- ─────────────────────────────────────────────────────────────
-- 13. updated_at TRIGGERS (SQLite does not have stored procs)
-- ─────────────────────────────────────────────────────────────

-- CREATE TRIGGER trg_users_updated_at
--     AFTER UPDATE ON users FOR EACH ROW
--     BEGIN UPDATE users SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_galaxies_updated_at
    AFTER UPDATE ON galaxies FOR EACH ROW
    BEGIN UPDATE galaxies SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_star_systems_updated_at
    AFTER UPDATE ON star_systems FOR EACH ROW
    BEGIN UPDATE galastar_systemsxies SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_planets_updated_at
    AFTER UPDATE ON planets FOR EACH ROW
    BEGIN UPDATE planets SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

-- // TODO add missing triggers