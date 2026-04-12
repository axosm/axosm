-- https://claude.ai/share/f3a79932-cf06-4638-acae-f0213bbf423a

-- This maps cleanly onto two distinct concepts:

-- Squads — a headcount. No individual HP. Attrition reduces the number directly. Marines, tanks — you just track how many are left.
-- Vessels — a constructed object with HP, armor, modules. Zocs, frigates, GSVs. Takes damage, needs repair, can be customized.

-- Ground-only unit types never need the vessel machinery, so there's no point mixing them in the same table.


-- ─────────────────────────────────────────────────────────────
-- 1. The split
-- ─────────────────────────────────────────────────────────────

-- A squad: pure headcount, no construction, no modules
-- unit_type limited to ground-only types
CREATE TABLE squad (
  id            INTEGER PRIMARY KEY,
  player_id     INTEGER NOT NULL REFERENCES player(id),
  unit_type     TEXT NOT NULL CHECK(unit_type IN ('marine', 'tank', 'artillery')),
  count         INTEGER NOT NULL DEFAULT 100,  -- this IS the hp
  planet_id     INTEGER REFERENCES planet(id),
  tile_x        INTEGER,
  tile_y        INTEGER,
  carried_by    INTEGER REFERENCES vessel(id)  -- can be loaded into a vessel
);

-- A vessel: has HP, has modules, can be designed by the player
-- unit_type limited to space-capable types
CREATE TABLE vessel (
  id              INTEGER PRIMARY KEY,
  player_id       INTEGER NOT NULL REFERENCES player(id),
  unit_type       TEXT NOT NULL CHECK(unit_type IN ('zoc', 'frigate', 'gsv')),
  name            TEXT NOT NULL,
  location_mode   TEXT NOT NULL DEFAULT 'planet_surface'
                  CHECK(location_mode IN (
                    'planet_surface', 'in_orbit', 'in_space', 'embarked'
                  )),

  -- Surface position (when location_mode = 'planet_surface')
  planet_id       INTEGER REFERENCES planet(id),
  tile_x          INTEGER,
  tile_y          INTEGER,

  -- Orbit (when location_mode = 'in_orbit')
  orbit_planet_id INTEGER REFERENCES planet(id),

  -- Deep space (when location_mode = 'in_space')
  space_x         REAL,
  space_y         REAL,
  space_z         REAL DEFAULT 0,

  -- Nesting: a Zoc inside a GSV
  carried_by      INTEGER REFERENCES vessel(id),

  -- HP as a proper stat, not a headcount
  hp_max          INTEGER NOT NULL,
  hp_current      INTEGER NOT NULL,

  -- Base stats from type, modified by modules
  base_speed      REAL NOT NULL DEFAULT 0,
  base_armor      INTEGER NOT NULL DEFAULT 0
);


-- ─────────────────────────────────────────────────────────────
-- 2. Ship design — the module system:
-- ─────────────────────────────────────────────────────────────

-- This is the Eve-like part. A vessel_design is a blueprint a player saves. A vessel references a design. Fitting a module changes the vessel's effective stats.

-- A player-saved blueprint, separate from the actual built vessel
CREATE TABLE vessel_design (
  id          INTEGER PRIMARY KEY,
  player_id   INTEGER NOT NULL REFERENCES player(id),
  unit_type   TEXT NOT NULL,
  name        TEXT NOT NULL,  -- "My fast Zoc", "Brick frigate"
  created_at  TEXT DEFAULT (datetime('now'))
);

-- Available module definitions (hardcoded in app, mirrored here for queries)
CREATE TABLE module_type (
  id          INTEGER PRIMARY KEY,
  name        TEXT NOT NULL,  -- 'shield_booster', 'warp_drive', 'cargo_bay'
  slot_type   TEXT NOT NULL CHECK(slot_type IN ('active', 'passive', 'utility')),
  hp_bonus    INTEGER DEFAULT 0,
  armor_bonus INTEGER DEFAULT 0,
  speed_bonus REAL    DEFAULT 0,
  carry_bonus INTEGER DEFAULT 0  -- extra cargo slots
);

-- Modules fitted onto a design
CREATE TABLE design_module (
  id          INTEGER PRIMARY KEY,
  design_id   INTEGER NOT NULL REFERENCES vessel_design(id),
  module_type_id INTEGER NOT NULL REFERENCES module_type(id),
  slot_index  INTEGER NOT NULL,  -- position in the fitting layout
  UNIQUE(design_id, slot_index)
);

-- When a vessel is built from a design, it gets its own module instances
-- (damage/repairs affect the instance, not the design)
CREATE TABLE vessel_module (
  id             INTEGER PRIMARY KEY,
  vessel_id      INTEGER NOT NULL REFERENCES vessel(id),
  module_type_id INTEGER NOT NULL REFERENCES module_type(id),
  slot_index     INTEGER NOT NULL,
  condition      REAL NOT NULL DEFAULT 1.0,  -- 1.0 = intact, 0 = destroyed
  UNIQUE(vessel_id, slot_index)
);


-- ─────────────────────────────────────────────────────────────
-- 3. Reading a vessel's effective stats (base + all fitted modules):
-- ─────────────────────────────────────────────────────────────

SELECT
  v.id,
  v.name,
  v.hp_current,
  v.hp_max + COALESCE(SUM(mt.hp_bonus), 0)    AS effective_hp_max,
  v.base_armor + COALESCE(SUM(mt.armor_bonus), 0) AS effective_armor,
  v.base_speed + COALESCE(SUM(mt.speed_bonus), 0) AS effective_speed
FROM vessel v
LEFT JOIN vessel_module vm ON vm.vessel_id = v.id AND vm.condition > 0
LEFT JOIN module_type mt   ON mt.id = vm.module_type_id
WHERE v.id = ?
GROUP BY v.id;


-- The condition > 0 filter means a destroyed module stops contributing its bonuses — a ship that takes a hit to its shield booster actually gets weaker, like in Eve.

-- ─────────────────────────────────────────────────────────────
-- 4. Movement queue — two entity types now:
-- ─────────────────────────────────────────────────────────────


CREATE TABLE movement_queue (
  id                INTEGER PRIMARY KEY,
  entity_type       TEXT NOT NULL CHECK(entity_type IN ('squad', 'vessel')),
  entity_id         INTEGER NOT NULL,
  move_type         TEXT NOT NULL CHECK(move_type IN (
                      'tile_walk', 'launch_to_orbit', 'orbit_to_space',
                      'space_travel', 'enter_orbit', 'land'
                    )),
  from_planet_id    INTEGER REFERENCES planet(id),
  from_tile_x       INTEGER,
  from_tile_y       INTEGER,
  from_space_x      REAL,
  from_space_y      REAL,
  to_planet_id      INTEGER REFERENCES planet(id),
  to_tile_x         INTEGER,
  to_tile_y         INTEGER,
  to_space_x        REAL,
  to_space_y        REAL,
  departure_time    TEXT NOT NULL DEFAULT (datetime('now')),
  scheduled_arrival TEXT NOT NULL,
  status            TEXT NOT NULL DEFAULT 'in_transit'
                    CHECK(status IN ('in_transit', 'arrived', 'recalled', 'cancelled')),
  callback_json     TEXT
);

CREATE INDEX idx_mq_poll ON movement_queue(status, scheduled_arrival);


-- Squads can only ever have move_type IN ('tile_walk') unless they are inside a vessel — in that case the vessel moves and drags the squad along. Your app enforces this: when a vessel moves, you don't queue individual moves for its passengers, they just inherit the vessel's new position on arrival.


-- ─────────────────────────────────────────────────────────────
-- 5. Summary of the final shape:
-- ─────────────────────────────────────────────────────────────

-- ┌─────────────────┬──────────────────────────────┬───────────┬──────────────┐
-- │ Concept         │ Table                        │ "HP"      │ Customizable │
-- ├─────────────────┼──────────────────────────────┼───────────┼──────────────┤
-- │ Marines, tanks  │ squad                        │ count     │ No           │
-- │ Zoc, frigate,   │ vessel                       │ hp_current│ Yes (modules)│
-- │ GSV             │                              │ hp_max    │              │
-- │ Ship blueprint  │ vessel_design + design_module│ —         │ Yes          │
-- │ Fitted instance │ vessel_module                │ condition │ —            │
-- └─────────────────┴──────────────────────────────┴───────────┴──────────────┘