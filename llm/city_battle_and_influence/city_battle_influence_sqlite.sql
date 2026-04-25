-- =============================================================
-- GAME SCHEMA — SQLite
-- =============================================================
-- Sections:
--   1. Players & Authentication
--   2. Universe & Space
--   3. Tiles
--   4. Influence
--   5. Buildings
--   6. Units & Movement
--   7. Combat
--   8. Resources
-- =============================================================

PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

-- ─────────────────────────────────────────────────────────────
-- 1. PLAYERS & AUTHENTICATION
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

CREATE INDEX idx_tiles_planet       ON planet_tiles(planet_id);
CREATE INDEX idx_tiles_owner        ON planet_tiles(owner_player_id) WHERE owner_player_id IS NOT NULL;
CREATE INDEX idx_tiles_recalc       ON planet_tiles(influence_recalc_needed) WHERE influence_recalc_needed = 1;
CREATE INDEX idx_tiles_rare_deposit ON planet_tiles(rare_deposit) WHERE rare_deposit IS NOT NULL;

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

-- Static config per building type. Stored in DB so game logic can query it.
CREATE TABLE building_types (
  unit_type                TEXT     PRIMARY KEY,
  era                      TEXT     NOT NULL
                           CHECK(era IN ('stone','industrial','modern','space')),

  -- Tile placement constraints
  valid_tile_types         TEXT     NOT NULL, -- JSON array e.g. ["forest","plains"]

  -- Construction
  build_cost_json          TEXT     NOT NULL, -- JSON: {"wood":100,"stone":50}
  build_time_ticks         INTEGER  NOT NULL,

  -- HP & combat
  base_hp                  INTEGER  NOT NULL,
  defence_value            REAL     NOT NULL DEFAULT 0, -- damage dealt to attacker per tick
  city_wide_defence_bonus  REAL     NOT NULL DEFAULT 0, -- added to all friendly tile battles in city
  auto_repair_hp_per_tick  REAL     NOT NULL DEFAULT 0.1,

  -- Resource production (passive, per tick, at level 1 — scales with level)
  produces_resource        TEXT,              -- NULL = no passive output
  base_output_per_tick     REAL     NOT NULL DEFAULT 0,

  -- Storage (for warehouse/storage buildings)
  storage_capacity_json    TEXT,             -- JSON: {"wood":5000,"stone":5000} or NULL

  -- Influence
  influence_power          REAL     NOT NULL DEFAULT 0,
  influence_radius         INTEGER  NOT NULL DEFAULT 0,

  -- Flying buildings (Starcraft-style liftoff)
  can_fly                  INTEGER  NOT NULL DEFAULT 0,
  fly_speed_tiles_per_tick REAL,
  liftoff_ticks            INTEGER  NOT NULL DEFAULT 0,
  land_cooldown_ticks      INTEGER  NOT NULL DEFAULT 0  -- ticks before usable after landing
);

-- Per-instance building on a tile.
-- One building per tile enforced by UNIQUE(tile_id).
CREATE TABLE buildings (
  id                   INTEGER  PRIMARY KEY AUTOINCREMENT,
  player_id            INTEGER  NOT NULL REFERENCES players(id),
  building_type        TEXT     NOT NULL REFERENCES building_types(unit_type),
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
CREATE INDEX idx_buildings_type        ON buildings(building_type);
CREATE INDEX idx_buildings_under_attack ON buildings(under_attack) WHERE under_attack = 1;
CREATE INDEX idx_buildings_destroyed   ON buildings(destroyed_at)  WHERE destroyed_at IS NOT NULL;
CREATE INDEX idx_buildings_flying      ON buildings(flight_state)  WHERE flight_state != 'grounded';

-- ─────────────────────────────────────────────────────────────
-- 6. UNITS & MOVEMENT
-- ─────────────────────────────────────────────────────────────

-- Static config per unit type
CREATE TABLE unit_types (
  unit_type              TEXT  PRIMARY KEY,
  era                    TEXT  NOT NULL
                         CHECK(era IN ('stone','industrial','modern','space')),
  attack                 REAL  NOT NULL,
  defence                REAL  NOT NULL,
  hp_per_individual      REAL  NOT NULL,
  speed_tiles_per_tick   REAL  NOT NULL,
  carry_capacity         INTEGER NOT NULL DEFAULT 0, -- resources this unit can carry when looting
  -- Type matchup bonuses: JSON map of target unit_type → multiplier
  -- e.g. {"swordsman": 1.5, "cavalry": 0.7}
  combat_bonuses_json    TEXT  NOT NULL DEFAULT '{}',
  recruit_cost_json      TEXT  NOT NULL,             -- JSON: {"wood":10,"stone":5}
  recruit_time_ticks     INTEGER NOT NULL
);

-- Per-instance unit / formation
CREATE TABLE units (
  id             INTEGER  PRIMARY KEY AUTOINCREMENT,
  unit_type      TEXT     NOT NULL REFERENCES unit_types(unit_type),
  is_squad       INTEGER  NOT NULL DEFAULT 0,  -- 1 = multiple individuals grouped
  count          INTEGER  NOT NULL DEFAULT 1,  -- number of individuals in formation
  hp             INTEGER  NOT NULL,
  player_id      INTEGER  NOT NULL REFERENCES players(id),

  -- Combat state
  in_battle      INTEGER  NOT NULL DEFAULT 0,  -- 1 = currently fighting on a tile

  -- Location
  location_mode  TEXT     NOT NULL DEFAULT 'planet_surface'
                 CHECK(location_mode IN (
                   'planet_surface','in_orbit','in_space','embarked'
                 )),

  -- Surface position (when location_mode = 'planet_surface')
  planet_id      INTEGER  REFERENCES planets(id),
  planet_face    INTEGER,
  planet_u       INTEGER,
  planet_v       INTEGER,

  -- Orbit (when location_mode = 'in_orbit')
  orbit_planet_id INTEGER  REFERENCES planets(id),

  -- Deep space (when location_mode = 'in_space')
  star_system_id  INTEGER  REFERENCES star_systems(id),
  star_system_x   REAL,
  star_system_y   REAL,
  star_system_z   REAL     DEFAULT 0,

  customization   TEXT,    -- JSON: cosmetic options

  created_at      TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at      TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),

  FOREIGN KEY (player_id) REFERENCES players(id)
);

CREATE INDEX idx_units_player      ON units(player_id);
CREATE INDEX idx_units_in_battle   ON units(in_battle) WHERE in_battle = 1;
CREATE INDEX idx_units_planet      ON units(planet_id, planet_face, planet_u, planet_v)
                                                  WHERE location_mode = 'planet_surface';
CREATE INDEX idx_units_orbit       ON units(orbit_planet_id)
                                                  WHERE location_mode = 'in_orbit';
CREATE INDEX idx_units_space       ON units(star_system_id, star_system_x, star_system_y)
                                                  WHERE location_mode = 'in_space';

-- All movement orders: units and flying buildings share this table.
CREATE TABLE move_orders (
  id              INTEGER  PRIMARY KEY AUTOINCREMENT,

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

  start_time     INTEGER  NOT NULL, -- game tick
  arrival_time   INTEGER  NOT NULL, -- game tick

  created_at     TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
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