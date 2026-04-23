

-- 20251001_create_tables.sql


PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ─────────────────────────────────────────────────────────────
-- 1. USERS & AUTHENTICATION
-- ─────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS players (
  id             INTEGER  PRIMARY KEY AUTOINCREMENT,
  username       TEXT     NOT NULL,
  email          TEXT     NOT NULL UNIQUE,
  password_hash  TEXT     NOT NULL,
  created_at     TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at     TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  last_login_at  TEXT
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
  subdivision     INTEGER  NOT NULL, -- Goldberg resolution (N)
  created_at      TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at      TEXT     NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_planets_system ON planets(star_system_id);




aaaaaa TODO review everything below
see https://claude.ai/share/f3a79932-cf06-4638-acae-f0213bbf423a


CREATE TABLE units (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  unit_type TEXT NOT NULL,
  is_squad BOOLEAN NOT NULL,
  count INTEGER NOT NULL,
  hp INTEGER NOT NULL,
  player_id INTEGER NOT NULL,

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
  unit_id INTEGER NOT NULL,
  move_type         TEXT NOT NULL CHECK(move_type IN (
                      'tile_walk', 'launch_to_orbit', 'orbit_to_space',
                      'space_travel', 'enter_orbit', 'land'
                    )),
  from_planet_id    INTEGER REFERENCES planets(id),
  from_planet_face INTEGER NOT NULL,
  from_planet_u INTEGER NOT NULL,
  from_planet_v INTEGER NOT NULL,
  to_planet_id    INTEGER REFERENCES planets(id),
  to_planet_face INTEGER NOT NULL,
  to_planet_u INTEGER NOT NULL,
  to_planet_v INTEGER NOT NULL,

  start_time INTEGER NOT NULL,
  arrival_time INTEGER NOT NULL,
  --   status            TEXT NOT NULL DEFAULT 'in_transit'
  --                   CHECK(status IN ('in_transit', 'arrived', 'recalled', 'cancelled')),
  -- callback_json     TEXT

  FOREIGN KEY(unit_id) REFERENCES units(id)
);

aaaa see Squad based or single unit control, which one do you prefer ? 
https://www.reddit.com/r/RealTimeStrategy/comments/182q4o3/squad_based_or_single_unit_control_which_one_do/

PostGIS
https://claude.ai/share/94821517-87e4-4601-9537-4ded5073e2ca
learn more about :
- Slash of clan defense/battle system ("how buildings have implicit HP and act as defenders")
- Settlers of Catan rules

schema / city battles / influence
https://claude.ai/share/b25daaf0-c880-4e4d-986a-3717515382ef


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
-- -- CREATE TABLE IF NOT EXISTS units (
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
-- -- CREATE TABLE IF NOT EXISTS move_orders (
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

CREATE TRIGGER trg_users_updated_at
    AFTER UPDATE ON users FOR EACH ROW
    BEGIN UPDATE users SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_galaxies_updated_at
    AFTER UPDATE ON galaxies FOR EACH ROW
    BEGIN UPDATE galaxies SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_star_systems_updated_at
    AFTER UPDATE ON star_systems FOR EACH ROW
    BEGIN UPDATE galastar_systemsxies SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

CREATE TRIGGER trg_planets_updated_at
    AFTER UPDATE ON planets FOR EACH ROW
    BEGIN UPDATE planets SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = OLD.id; END;

