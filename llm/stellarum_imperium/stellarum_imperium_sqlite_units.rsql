-- from https://claude.ai/share/106113ec-5bab-4f5f-a680-7e1cd397fd35

-- A few things worth noting

-- army_movements and fleet_movements have no updated_at — they're transient records that get inserted once and deleted on arrival. No update ever happens to them.

-- commanders uses ON DELETE SET NULL on both armies and fleets — if a commander dies you null out the FK, you don't cascade-delete the army.

-- ships.fleet_id is nullable intentionally — a ship being built, docked alone at a planet, or being transported inside another ship has no fleet yet.

-- The game loop query for your tick processor is now trivial:

-- sql-- Ground arrivals
-- SELECT * FROM army_movements WHERE arrives_at <= datetime('now');

-- -- Space arrivals  
-- SELECT * FROM fleet_movements WHERE arrives_at <= datetime('now');

-- What's deliberately left out for later: buildings, technologies, resources, combat_logs, jumpgates. Get the unit/movement loop working first.

-- ─────────────────────────────────────────────────────────────
-- USERS & EMPIRES
-- ─────────────────────────────────────────────────────────────
CREATE TABLE users (
    id            INTEGER  PRIMARY KEY AUTOINCREMENT,
    username      TEXT     NOT NULL UNIQUE,
    email         TEXT     NOT NULL UNIQUE,
    password_hash TEXT     NOT NULL,
    created_at    TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE empires (
    id         INTEGER  PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER  NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name       TEXT     NOT NULL,
    created_at TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

-- ─────────────────────────────────────────────────────────────
-- WORLD
-- ─────────────────────────────────────────────────────────────
CREATE TABLE solar_systems (
    id   INTEGER  PRIMARY KEY AUTOINCREMENT,
    name TEXT     NOT NULL,
    x    REAL     NOT NULL,
    y    REAL     NOT NULL
);

CREATE TABLE planets (
    id              INTEGER  PRIMARY KEY AUTOINCREMENT,
    solar_system_id INTEGER  NOT NULL REFERENCES solar_systems(id),
    name            TEXT     NOT NULL,
    type            TEXT     NOT NULL  -- 'terrestrial', 'gas_giant', 'asteroid', etc.
);

CREATE TABLE planet_tiles (
    id           INTEGER  PRIMARY KEY AUTOINCREMENT,
    planet_id    INTEGER  NOT NULL REFERENCES planets(id),
    face         INTEGER  NOT NULL,   -- Goldberg sphere face index
    u            INTEGER  NOT NULL,
    v            INTEGER  NOT NULL,
    terrain_type TEXT     NOT NULL,   -- 'plains', 'forest', 'mountain', 'river', etc.
    building_id  INTEGER,             -- FK to buildings table (future)
    UNIQUE (planet_id, face, u, v)
);

CREATE INDEX idx_tiles_planet ON planet_tiles(planet_id);

-- ─────────────────────────────────────────────────────────────
-- COMMANDERS  (shared between ground armies and space fleets)
-- ─────────────────────────────────────────────────────────────
CREATE TABLE commanders (
    id         INTEGER  PRIMARY KEY AUTOINCREMENT,
    empire_id  INTEGER  NOT NULL REFERENCES empires(id),
    name       TEXT     NOT NULL,
    xp         INTEGER  NOT NULL DEFAULT 0,
    level      INTEGER  NOT NULL DEFAULT 1,
    hp         INTEGER  NOT NULL,
    hp_max     INTEGER  NOT NULL,
    abilities  TEXT     NOT NULL DEFAULT '[]',  -- JSON array of ability keys
    created_at TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

-- ─────────────────────────────────────────────────────────────
-- GROUND: ARMIES
-- ─────────────────────────────────────────────────────────────
CREATE TABLE armies (
    id             INTEGER  PRIMARY KEY AUTOINCREMENT,
    empire_id      INTEGER  NOT NULL REFERENCES empires(id),
    commander_id   INTEGER  REFERENCES commanders(id) ON DELETE SET NULL,
    planet_tile_id INTEGER  NOT NULL REFERENCES planet_tiles(id),
    morale         INTEGER  NOT NULL DEFAULT 100 CHECK (morale BETWEEN 0 AND 100),
    aggression     TEXT     NOT NULL DEFAULT 'neutral'
                       CHECK (aggression IN ('aggressive','neutral','defensive')),
    created_at     TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at     TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

-- One row per unit type in the army (e.g. 600 swordsmen + 400 archers = 2 rows)
CREATE TABLE army_units (
    id           INTEGER  PRIMARY KEY AUTOINCREMENT,
    army_id      INTEGER  NOT NULL REFERENCES armies(id) ON DELETE CASCADE,
    unit_type    TEXT     NOT NULL,   -- key into your hardcoded UNIT_TYPES config
    quantity     INTEGER  NOT NULL CHECK (quantity > 0),
    quantity_max INTEGER  NOT NULL    -- heals back toward this cap; raise with reinforcements
);

-- Only exists while the army is moving; deleted on arrival
CREATE TABLE army_movements (
    id           INTEGER  PRIMARY KEY AUTOINCREMENT,
    army_id      INTEGER  NOT NULL UNIQUE REFERENCES armies(id) ON DELETE CASCADE,
    from_tile_id INTEGER  NOT NULL REFERENCES planet_tiles(id),
    to_tile_id   INTEGER  NOT NULL REFERENCES planet_tiles(id),
    arrives_at   TEXT     NOT NULL
);

CREATE INDEX idx_armies_empire    ON armies(empire_id);
CREATE INDEX idx_armies_tile      ON armies(planet_tile_id);
CREATE INDEX idx_army_units_army  ON army_units(army_id);
CREATE INDEX idx_army_moves_eta   ON army_movements(arrives_at);

-- ─────────────────────────────────────────────────────────────
-- SPACE: FLEETS & SHIPS
-- ─────────────────────────────────────────────────────────────
CREATE TABLE fleets (
    id              INTEGER  PRIMARY KEY AUTOINCREMENT,
    empire_id       INTEGER  NOT NULL REFERENCES empires(id),
    commander_id    INTEGER  REFERENCES commanders(id) ON DELETE SET NULL,
    name            TEXT,
    location_type   TEXT     NOT NULL CHECK (location_type IN ('orbit','space')),
    planet_id       INTEGER  REFERENCES planets(id),        -- when orbit
    solar_system_id INTEGER  REFERENCES solar_systems(id),
    space_x         REAL,
    space_y         REAL,
    created_at      TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at      TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

-- Each ship is an individual with its own HP bar
CREATE TABLE ships (
    id         INTEGER  PRIMARY KEY AUTOINCREMENT,
    fleet_id   INTEGER  REFERENCES fleets(id) ON DELETE SET NULL,  -- NULL = solo / docked
    empire_id  INTEGER  NOT NULL REFERENCES empires(id),
    ship_type  TEXT     NOT NULL,   -- key into hardcoded SHIP_TYPES config
    hp         INTEGER  NOT NULL,
    cargo      TEXT     NOT NULL DEFAULT '{}',  -- JSON: { "resource_key": amount }
    created_at TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

-- Only exists while the fleet is moving; deleted on arrival
CREATE TABLE fleet_movements (
    id            INTEGER  PRIMARY KEY AUTOINCREMENT,
    fleet_id      INTEGER  NOT NULL UNIQUE REFERENCES fleets(id) ON DELETE CASCADE,
    from_x        REAL     NOT NULL,
    from_y        REAL     NOT NULL,
    to_x          REAL     NOT NULL,
    to_y          REAL     NOT NULL,
    to_planet_id  INTEGER  REFERENCES planets(id),   -- set when heading to orbit
    arrives_at    TEXT     NOT NULL
);

CREATE INDEX idx_fleets_empire   ON fleets(empire_id);
CREATE INDEX idx_fleets_system   ON fleets(solar_system_id);
CREATE INDEX idx_ships_fleet     ON ships(fleet_id);
CREATE INDEX idx_fleet_moves_eta ON fleet_movements(arrives_at);