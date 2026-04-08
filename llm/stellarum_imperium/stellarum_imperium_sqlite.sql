-- ============================================================
-- STELLARUM IMPERIUM — SQLite Schema
-- Version 0.1 (matches GDD v0.1)
--
-- SQLite differences vs PostgreSQL version:
--   • BIGSERIAL → INTEGER PRIMARY KEY AUTOINCREMENT
--   • TIMESTAMPTZ → TEXT (ISO-8601 strings; use strftime / datetime())
--   • FLOAT → REAL
--   • BOOLEAN → INTEGER (0/1)
--   • JSONB → TEXT (JSON stored as text; parse in application layer)
--   • No partial indexes (WHERE clause on CREATE INDEX is dropped)
--   • No stored procedures / DO blocks; updated_at managed in app layer
--   • No gist/PostGIS spatial indexes; spatial queries done in application
--   • CHECK constraints: SQLite enforces them only from version 3.25+;
--     they are included for documentation and future-proofing.
--   • Deferred foreign keys: enable with PRAGMA foreign_keys = ON;
-- ============================================================

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ─────────────────────────────────────────────────────────────
-- 1. USERS & AUTHENTICATION
-- ─────────────────────────────────────────────────────────────

CREATE TABLE users (
    id              INTEGER     PRIMARY KEY AUTOINCREMENT,
    username        TEXT        NOT NULL UNIQUE,
    email           TEXT        NOT NULL UNIQUE,
    password_hash   TEXT        NOT NULL,
    created_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    last_login_at   TEXT
);

-- ─────────────────────────────────────────────────────────────
-- 2. UNIVERSE & SPACE
-- ─────────────────────────────────────────────────────────────

CREATE TABLE galaxies (
    id          INTEGER     PRIMARY KEY AUTOINCREMENT,
    seed        INTEGER     NOT NULL,
    name        TEXT        NOT NULL,
    radius      REAL        NOT NULL,
    created_at  TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at  TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE solar_systems (
    id          INTEGER     PRIMARY KEY AUTOINCREMENT,
    galaxy_id   INTEGER     NOT NULL REFERENCES galaxies(id),
    seed        INTEGER     NOT NULL,
    name        TEXT        NOT NULL,
    x           REAL        NOT NULL,
    y           REAL        NOT NULL,
    star_type   TEXT,
    created_at  TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at  TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_solar_systems_galaxy ON solar_systems(galaxy_id);

CREATE TABLE planets (
    id                INTEGER     PRIMARY KEY AUTOINCREMENT,
    solar_system_id   INTEGER     NOT NULL REFERENCES solar_systems(id),
    seed              INTEGER     NOT NULL,
    name              TEXT        NOT NULL,
    orbital_slot      INTEGER     NOT NULL,
    tile_count        INTEGER     NOT NULL CHECK (tile_count BETWEEN 100 AND 10000),
    planet_type       TEXT,
    created_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_planets_system ON planets(solar_system_id);

CREATE TABLE planet_tiles (
    id                INTEGER     PRIMARY KEY AUTOINCREMENT,
    planet_id         INTEGER     NOT NULL REFERENCES planets(id),
    tile_index        INTEGER     NOT NULL,
    terrain_type      TEXT        NOT NULL
                          CHECK (terrain_type IN ('plains','forest','mountain','water','lava','tundra','desert')),
    is_pentagon       INTEGER     NOT NULL DEFAULT 0 CHECK (is_pentagon IN (0,1)),
    owner_empire_id   INTEGER     REFERENCES empires(id),   -- nullable; FK to empires (defined below)
    created_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    UNIQUE (planet_id, tile_index)
);

CREATE INDEX idx_planet_tiles_planet ON planet_tiles(planet_id);
CREATE INDEX idx_planet_tiles_owner  ON planet_tiles(owner_empire_id);

CREATE TABLE tile_adjacency (
    tile_id       INTEGER     NOT NULL REFERENCES planet_tiles(id),
    neighbor_id   INTEGER     NOT NULL REFERENCES planet_tiles(id),
    PRIMARY KEY (tile_id, neighbor_id),
    CHECK (tile_id <> neighbor_id)
);

CREATE INDEX idx_tile_adj_neighbor ON tile_adjacency(neighbor_id);

CREATE TABLE asteroids (
    id                INTEGER     PRIMARY KEY AUTOINCREMENT,
    solar_system_id   INTEGER     NOT NULL REFERENCES solar_systems(id),
    x                 REAL        NOT NULL,
    y                 REAL        NOT NULL,
    size              TEXT        NOT NULL CHECK (size IN ('small','large')),
    resources         TEXT,       -- JSON: {resource_key: amount}
    created_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_asteroids_system ON asteroids(solar_system_id);

-- ─────────────────────────────────────────────────────────────
-- 3. EMPIRES & DIPLOMACY
-- ─────────────────────────────────────────────────────────────

CREATE TABLE empires (
    id                  INTEGER     PRIMARY KEY AUTOINCREMENT,
    name                TEXT        NOT NULL UNIQUE,
    founder_user_id     INTEGER     NOT NULL REFERENCES users(id),
    capital_planet_id   INTEGER     REFERENCES planets(id),
    treasury            TEXT        NOT NULL DEFAULT '{}',  -- JSON
    created_at          TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at          TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE empire_members (
    id          INTEGER     PRIMARY KEY AUTOINCREMENT,
    empire_id   INTEGER     NOT NULL REFERENCES empires(id),
    user_id     INTEGER     NOT NULL REFERENCES users(id),
    role        TEXT        NOT NULL DEFAULT 'member',
    joined_at   TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    created_at  TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at  TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    UNIQUE (empire_id, user_id)
);

CREATE TABLE diplomatic_relations (
    id              INTEGER     PRIMARY KEY AUTOINCREMENT,
    empire_a_id     INTEGER     NOT NULL REFERENCES empires(id),
    empire_b_id     INTEGER     NOT NULL REFERENCES empires(id),
    status          TEXT        NOT NULL DEFAULT 'neutral'
                        CHECK (status IN ('war','peace','alliance','neutral','hostile')),
    declared_at     TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    created_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    UNIQUE (empire_a_id, empire_b_id),
    CHECK (empire_a_id < empire_b_id)
);

CREATE TABLE alliances (
    id          INTEGER     PRIMARY KEY AUTOINCREMENT,
    name        TEXT,
    formed_at   TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    terms       TEXT,       -- JSON
    created_at  TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at  TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE alliance_members (
    alliance_id   INTEGER     NOT NULL REFERENCES alliances(id),
    empire_id     INTEGER     NOT NULL REFERENCES empires(id),
    joined_at     TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    PRIMARY KEY (alliance_id, empire_id)
);

-- ─────────────────────────────────────────────────────────────
-- 4. RESOURCES
-- ─────────────────────────────────────────────────────────────

CREATE TABLE resource_types (
    id                INTEGER     PRIMARY KEY AUTOINCREMENT,
    key               TEXT        NOT NULL UNIQUE,
    name              TEXT        NOT NULL,
    era               TEXT        NOT NULL,
    is_energy         INTEGER     NOT NULL DEFAULT 0 CHECK (is_energy IN (0,1)),
    required_tech_id  INTEGER     REFERENCES technologies(id),
    created_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE planet_resources (
    id                INTEGER     PRIMARY KEY AUTOINCREMENT,
    planet_id         INTEGER     NOT NULL REFERENCES planets(id),
    empire_id         INTEGER     NOT NULL REFERENCES empires(id),
    resource_type_id  INTEGER     NOT NULL REFERENCES resource_types(id),
    amount            INTEGER     NOT NULL DEFAULT 0 CHECK (amount >= 0),
    capacity          INTEGER     NOT NULL DEFAULT 0 CHECK (capacity >= 0),
    created_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    UNIQUE (planet_id, empire_id, resource_type_id)
);

CREATE INDEX idx_planet_resources_planet ON planet_resources(planet_id);
CREATE INDEX idx_planet_resources_empire ON planet_resources(empire_id);

-- ─────────────────────────────────────────────────────────────
-- 5. BUILDINGS
-- ─────────────────────────────────────────────────────────────

CREATE TABLE building_types (
    id                INTEGER     PRIMARY KEY AUTOINCREMENT,
    key               TEXT        NOT NULL UNIQUE,
    name              TEXT        NOT NULL,
    category          TEXT        NOT NULL,
    base_build_time   INTEGER     NOT NULL,
    base_cost         TEXT        NOT NULL DEFAULT '{}',    -- JSON
    base_output       TEXT        NOT NULL DEFAULT '{}',    -- JSON
    max_level         INTEGER     NOT NULL DEFAULT 10,
    allowed_terrains  TEXT        NOT NULL DEFAULT '[]',    -- JSON array of terrain strings
    required_tech_id  INTEGER     REFERENCES technologies(id),
    created_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE tile_buildings (
    id                INTEGER     PRIMARY KEY AUTOINCREMENT,
    planet_tile_id    INTEGER     NOT NULL UNIQUE REFERENCES planet_tiles(id),
    empire_id         INTEGER     NOT NULL REFERENCES empires(id),
    building_type_id  INTEGER     NOT NULL REFERENCES building_types(id),
    level             INTEGER     NOT NULL DEFAULT 1 CHECK (level >= 1),
    hp                INTEGER     NOT NULL,
    max_hp            INTEGER     NOT NULL,
    workers_assigned  INTEGER     NOT NULL DEFAULT 0 CHECK (workers_assigned >= 0),
    created_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_tile_buildings_empire ON tile_buildings(empire_id);
CREATE INDEX idx_tile_buildings_type   ON tile_buildings(building_type_id);

-- ─────────────────────────────────────────────────────────────
-- 6. RESEARCH / TECHNOLOGY
-- ─────────────────────────────────────────────────────────────

CREATE TABLE technologies (
    id                  INTEGER     PRIMARY KEY AUTOINCREMENT,
    key                 TEXT        NOT NULL UNIQUE,
    name                TEXT        NOT NULL,
    era                 TEXT        NOT NULL,
    category            TEXT,
    base_research_time  INTEGER     NOT NULL,
    base_cost           TEXT        NOT NULL DEFAULT '{}',  -- JSON
    server_age_gate     INTEGER     NOT NULL DEFAULT 0,     -- seconds
    effects             TEXT        NOT NULL DEFAULT '{}',  -- JSON
    created_at          TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at          TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE technology_prerequisites (
    tech_id       INTEGER     NOT NULL REFERENCES technologies(id),
    requires_id   INTEGER     NOT NULL REFERENCES technologies(id),
    PRIMARY KEY (tech_id, requires_id),
    CHECK (tech_id <> requires_id)
);

CREATE TABLE player_research (
    id            INTEGER     PRIMARY KEY AUTOINCREMENT,
    user_id       INTEGER     NOT NULL REFERENCES users(id),
    tech_id       INTEGER     NOT NULL REFERENCES technologies(id),
    status        TEXT        NOT NULL DEFAULT 'in_progress'
                      CHECK (status IN ('in_progress','completed')),
    started_at    TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    completes_at  TEXT        NOT NULL,
    completed_at  TEXT,
    created_at    TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at    TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    UNIQUE (user_id, tech_id)
);

CREATE INDEX idx_player_research_user   ON player_research(user_id);
CREATE INDEX idx_player_research_status ON player_research(status);

-- ─────────────────────────────────────────────────────────────
-- 7. UNITS
-- ─────────────────────────────────────────────────────────────

CREATE TABLE unit_types (
    id                INTEGER     PRIMARY KEY AUTOINCREMENT,
    key               TEXT        NOT NULL UNIQUE,
    name              TEXT        NOT NULL,
    unit_class        TEXT        NOT NULL CHECK (unit_class IN ('ground','ship')),
    ship_class        TEXT,
    attack            INTEGER     NOT NULL DEFAULT 0,
    defense           INTEGER     NOT NULL DEFAULT 0,
    hp_per_unit       INTEGER     NOT NULL DEFAULT 1,
    speed             INTEGER     NOT NULL DEFAULT 1,
    cargo             INTEGER     NOT NULL DEFAULT 0,
    cloak_rating      INTEGER     NOT NULL DEFAULT 0,
    scan_power        INTEGER     NOT NULL DEFAULT 0,
    orbit_capable     INTEGER     NOT NULL DEFAULT 0 CHECK (orbit_capable IN (0,1)),
    required_tech_id  INTEGER     REFERENCES technologies(id),
    created_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE unit_stacks (
    id                INTEGER     PRIMARY KEY AUTOINCREMENT,
    unit_type_id      INTEGER     NOT NULL REFERENCES unit_types(id),
    owner_empire_id   INTEGER     NOT NULL REFERENCES empires(id),
    owner_user_id     INTEGER     NOT NULL REFERENCES users(id),
    quantity          INTEGER     NOT NULL DEFAULT 1 CHECK (quantity > 0),
    location_type     TEXT        NOT NULL
                          CHECK (location_type IN ('planet_tile','orbit_low','orbit_high','space')),
    planet_tile_id    INTEGER     REFERENCES planet_tiles(id),
    planet_id         INTEGER     REFERENCES planets(id),
    space_x           REAL,
    space_y           REAL,
    solar_system_id   INTEGER     REFERENCES solar_systems(id),
    aggression        TEXT        NOT NULL DEFAULT 'neutral'
                          CHECK (aggression IN ('aggressive','neutral','defensive')),
    hp                INTEGER     NOT NULL,
    cargo_contents    TEXT        NOT NULL DEFAULT '{}',  -- JSON
    created_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_unit_stacks_empire ON unit_stacks(owner_empire_id);
CREATE INDEX idx_unit_stacks_tile   ON unit_stacks(planet_tile_id);
CREATE INDEX idx_unit_stacks_planet ON unit_stacks(planet_id);

-- ─────────────────────────────────────────────────────────────
-- 8. MOVEMENT ORDERS
-- ─────────────────────────────────────────────────────────────

CREATE TABLE movement_orders (
    id              INTEGER     PRIMARY KEY AUTOINCREMENT,
    unit_stack_id   INTEGER     NOT NULL REFERENCES unit_stacks(id),
    origin_type     TEXT        NOT NULL
                        CHECK (origin_type IN ('planet_tile','orbit_low','orbit_high','space')),
    dest_type       TEXT        NOT NULL
                        CHECK (dest_type IN ('planet_tile','orbit_low','orbit_high','space')),
    dest_tile_id    INTEGER     REFERENCES planet_tiles(id),
    dest_planet_id  INTEGER     REFERENCES planets(id),
    dest_x          REAL,
    dest_y          REAL,
    dest_system_id  INTEGER     REFERENCES solar_systems(id),
    started_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    arrives_at      TEXT        NOT NULL,
    status          TEXT        NOT NULL DEFAULT 'in_transit'
                        CHECK (status IN ('in_transit','arrived','cancelled')),
    created_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_movement_orders_stack   ON movement_orders(unit_stack_id);
CREATE INDEX idx_movement_orders_arrives ON movement_orders(arrives_at);

-- ─────────────────────────────────────────────────────────────
-- 9. COMBAT
-- ─────────────────────────────────────────────────────────────

CREATE TABLE battles (
    id                INTEGER     PRIMARY KEY AUTOINCREMENT,
    arena_type        TEXT        NOT NULL
                          CHECK (arena_type IN ('planet_tile','orbit_low','orbit_high','space')),
    location_ref      INTEGER,
    solar_system_id   INTEGER     REFERENCES solar_systems(id),
    space_x           REAL,
    space_y           REAL,
    status            TEXT        NOT NULL DEFAULT 'active'
                          CHECK (status IN ('active','resolved','ceasefire')),
    started_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    resolved_at       TEXT,
    round_number      INTEGER     NOT NULL DEFAULT 0,
    created_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_battles_status ON battles(status);

CREATE TABLE battle_participants (
    id              INTEGER     PRIMARY KEY AUTOINCREMENT,
    battle_id       INTEGER     NOT NULL REFERENCES battles(id),
    unit_stack_id   INTEGER     NOT NULL REFERENCES unit_stacks(id),
    empire_id       INTEGER     NOT NULL REFERENCES empires(id),
    side            TEXT        NOT NULL CHECK (side IN ('attacker','defender')),
    initial_qty     INTEGER     NOT NULL,
    current_qty     INTEGER     NOT NULL,
    current_hp      INTEGER     NOT NULL,
    created_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_battle_participants_battle ON battle_participants(battle_id);

CREATE TABLE battle_rounds (
    id            INTEGER     PRIMARY KEY AUTOINCREMENT,
    battle_id     INTEGER     NOT NULL REFERENCES battles(id),
    round_number  INTEGER     NOT NULL,
    resolved_at   TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    damage_log    TEXT        NOT NULL DEFAULT '{}',  -- JSON
    losses_log    TEXT        NOT NULL DEFAULT '{}',  -- JSON
    created_at    TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at    TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    UNIQUE (battle_id, round_number)
);

CREATE INDEX idx_battle_rounds_battle ON battle_rounds(battle_id);

-- ─────────────────────────────────────────────────────────────
-- 10. SPACE STATIONS & STARGATES
-- ─────────────────────────────────────────────────────────────

CREATE TABLE space_stations (
    id                INTEGER     PRIMARY KEY AUTOINCREMENT,
    owner_empire_id   INTEGER     NOT NULL REFERENCES empires(id),
    solar_system_id   INTEGER     NOT NULL REFERENCES solar_systems(id),
    asteroid_id       INTEGER     REFERENCES asteroids(id),
    x                 REAL,
    y                 REAL,
    hp                INTEGER     NOT NULL,
    max_hp            INTEGER     NOT NULL,
    modules           TEXT        NOT NULL DEFAULT '[]',  -- JSON array
    created_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_space_stations_empire ON space_stations(owner_empire_id);
CREATE INDEX idx_space_stations_system ON space_stations(solar_system_id);

CREATE TABLE stargates (
    id                INTEGER     PRIMARY KEY AUTOINCREMENT,
    owner_empire_id   INTEGER     NOT NULL REFERENCES empires(id),
    solar_system_id   INTEGER     NOT NULL REFERENCES solar_systems(id),
    x                 REAL        NOT NULL,
    y                 REAL        NOT NULL,
    linked_gate_id    INTEGER     REFERENCES stargates(id),
    hp                INTEGER     NOT NULL,
    is_active         INTEGER     NOT NULL DEFAULT 0 CHECK (is_active IN (0,1)),
    created_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at        TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_stargates_empire ON stargates(owner_empire_id);
CREATE INDEX idx_stargates_system ON stargates(solar_system_id);

-- ─────────────────────────────────────────────────────────────
-- 11. CONSTRUCTION QUEUE
-- ─────────────────────────────────────────────────────────────

CREATE TABLE construction_queue (
    id                  INTEGER     PRIMARY KEY AUTOINCREMENT,
    empire_id           INTEGER     NOT NULL REFERENCES empires(id),
    queue_type          TEXT        NOT NULL
                            CHECK (queue_type IN ('building','unit','ship','repair')),
    reference_id        INTEGER     NOT NULL,
    location_tile_id    INTEGER     REFERENCES planet_tiles(id),
    location_planet_id  INTEGER     REFERENCES planets(id),
    quantity            INTEGER     NOT NULL DEFAULT 1 CHECK (quantity > 0),
    started_at          TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    completes_at        TEXT        NOT NULL,
    status              TEXT        NOT NULL DEFAULT 'queued'
                            CHECK (status IN ('queued','in_progress','done','cancelled')),
    created_at          TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at          TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_construction_queue_empire    ON construction_queue(empire_id);
CREATE INDEX idx_construction_queue_completes ON construction_queue(completes_at);

-- ─────────────────────────────────────────────────────────────
-- 12. SERVER CONFIGURATION
-- ─────────────────────────────────────────────────────────────

CREATE TABLE server_config (
    key           TEXT    PRIMARY KEY,
    value         TEXT    NOT NULL,
    description   TEXT,
    created_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

INSERT INTO server_config (key, value, description)
VALUES ('server_start_time',
        CAST(strftime('%s','now') AS TEXT),
        'Unix timestamp of server launch; used to compute all tech gate windows');

-- ─────────────────────────────────────────────────────────────
-- 13. updated_at TRIGGERS (SQLite does not have stored procs)
-- ─────────────────────────────────────────────────────────────

CREATE TRIGGER trg_users_updated_at
    AFTER UPDATE ON users FOR EACH ROW
    BEGIN UPDATE users SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_galaxies_updated_at
    AFTER UPDATE ON galaxies FOR EACH ROW
    BEGIN UPDATE galaxies SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_solar_systems_updated_at
    AFTER UPDATE ON solar_systems FOR EACH ROW
    BEGIN UPDATE solar_systems SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_planets_updated_at
    AFTER UPDATE ON planets FOR EACH ROW
    BEGIN UPDATE planets SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_planet_tiles_updated_at
    AFTER UPDATE ON planet_tiles FOR EACH ROW
    BEGIN UPDATE planet_tiles SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_asteroids_updated_at
    AFTER UPDATE ON asteroids FOR EACH ROW
    BEGIN UPDATE asteroids SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_empires_updated_at
    AFTER UPDATE ON empires FOR EACH ROW
    BEGIN UPDATE empires SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_empire_members_updated_at
    AFTER UPDATE ON empire_members FOR EACH ROW
    BEGIN UPDATE empire_members SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_diplomatic_relations_updated_at
    AFTER UPDATE ON diplomatic_relations FOR EACH ROW
    BEGIN UPDATE diplomatic_relations SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_alliances_updated_at
    AFTER UPDATE ON alliances FOR EACH ROW
    BEGIN UPDATE alliances SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_resource_types_updated_at
    AFTER UPDATE ON resource_types FOR EACH ROW
    BEGIN UPDATE resource_types SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_planet_resources_updated_at
    AFTER UPDATE ON planet_resources FOR EACH ROW
    BEGIN UPDATE planet_resources SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_building_types_updated_at
    AFTER UPDATE ON building_types FOR EACH ROW
    BEGIN UPDATE building_types SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_tile_buildings_updated_at
    AFTER UPDATE ON tile_buildings FOR EACH ROW
    BEGIN UPDATE tile_buildings SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_technologies_updated_at
    AFTER UPDATE ON technologies FOR EACH ROW
    BEGIN UPDATE technologies SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_player_research_updated_at
    AFTER UPDATE ON player_research FOR EACH ROW
    BEGIN UPDATE player_research SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_unit_types_updated_at
    AFTER UPDATE ON unit_types FOR EACH ROW
    BEGIN UPDATE unit_types SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_unit_stacks_updated_at
    AFTER UPDATE ON unit_stacks FOR EACH ROW
    BEGIN UPDATE unit_stacks SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_movement_orders_updated_at
    AFTER UPDATE ON movement_orders FOR EACH ROW
    BEGIN UPDATE movement_orders SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_battles_updated_at
    AFTER UPDATE ON battles FOR EACH ROW
    BEGIN UPDATE battles SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_battle_participants_updated_at
    AFTER UPDATE ON battle_participants FOR EACH ROW
    BEGIN UPDATE battle_participants SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_battle_rounds_updated_at
    AFTER UPDATE ON battle_rounds FOR EACH ROW
    BEGIN UPDATE battle_rounds SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_space_stations_updated_at
    AFTER UPDATE ON space_stations FOR EACH ROW
    BEGIN UPDATE space_stations SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_stargates_updated_at
    AFTER UPDATE ON stargates FOR EACH ROW
    BEGIN UPDATE stargates SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_construction_queue_updated_at
    AFTER UPDATE ON construction_queue FOR EACH ROW
    BEGIN UPDATE construction_queue SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_server_config_updated_at
    AFTER UPDATE ON server_config FOR EACH ROW
    BEGIN UPDATE server_config SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE key = OLD.key; END;
