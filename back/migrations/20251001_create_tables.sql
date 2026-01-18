

-- 20251001_create_tables.sql


CREATE TABLE galaxies (
  id INTEGER PRIMARY KEY,
  seed INTEGER NOT NULL
);

CREATE TABLE star_systems (
  id INTEGER PRIMARY KEY,
  galaxy_id INTEGER NOT NULL,
  gx INTEGER NOT NULL,
  gy INTEGER NOT NULL,
  gz INTEGER NOT NULL,
  seed INTEGER NOT NULL,
  UNIQUE(galaxy_id, gx, gy, gz),
  FOREIGN KEY(galaxy_id) REFERENCES galaxies(id)
);

CREATE TABLE planets (
  id INTEGER PRIMARY KEY,
  star_system_id INTEGER NOT NULL,
  orbit_index INTEGER NOT NULL,
  radius INTEGER NOT NULL,
  subdivision INTEGER NOT NULL, -- Goldberg resolution (N)
  seed INTEGER NOT NULL,
  FOREIGN KEY(star_system_id) REFERENCES star_systems(id)
);

CREATE TABLE IF NOT EXISTS players (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL
);

-- old schema
-- CREATE TABLE IF NOT EXISTS units (
--   id INTEGER PRIMARY KEY,
--   player_id INTEGER NOT NULL,
--   x INTEGER NOT NULL,
--   y INTEGER NOT NULL,
--   FOREIGN KEY(player_id) REFERENCES players(id)
-- );
CREATE TABLE units (
  id INTEGER PRIMARY KEY,
  player_id INTEGER NOT NULL,
  unit_type TEXT NOT NULL,
  location_type TEXT NOT NULL, -- 'PLANET_SURFACE', 'ORBIT', 'SPACE'
  FOREIGN KEY(player_id) REFERENCES players(id)
);

CREATE TABLE unit_planet_locations (
  unit_id INTEGER PRIMARY KEY,
  planet_id INTEGER NOT NULL,
  face INTEGER NOT NULL,
  u INTEGER NOT NULL,
  v INTEGER NOT NULL,
  FOREIGN KEY(unit_id) REFERENCES units(id),
  FOREIGN KEY(planet_id) REFERENCES planets(id)
);

-- old schema
-- CREATE TABLE IF NOT EXISTS move_orders (
--   id INTEGER PRIMARY KEY,
--   unit_id INTEGER NOT NULL,
--   from_x INTEGER NOT NULL,
--   from_y INTEGER NOT NULL,
--   to_x INTEGER NOT NULL,
--   to_y INTEGER NOT NULL,
--   arrival_time INTEGER NOT NULL,
--   FOREIGN KEY(unit_id) REFERENCES units(id)
-- );
CREATE TABLE move_orders (
  id INTEGER PRIMARY KEY,
  unit_id INTEGER NOT NULL,
  from_face INTEGER NOT NULL,
  from_u INTEGER NOT NULL,
  from_v INTEGER NOT NULL,
  to_face INTEGER NOT NULL,
  to_u INTEGER NOT NULL,
  to_v INTEGER NOT NULL,
  start_time INTEGER NOT NULL,
  arrival_time INTEGER NOT NULL,
  FOREIGN KEY(unit_id) REFERENCES units(id)
);



-- Future: Orbital / Space Location (DO NOT USE YET)
-- When you are ready, you add this without touching existing tables.
-- CREATE TABLE unit_space_locations (
--   unit_id INTEGER PRIMARY KEY,
--   star_system_id INTEGER NOT NULL,
--   x REAL NOT NULL,
--   y REAL NOT NULL,
--   z REAL NOT NULL,
--   FOREIGN KEY(unit_id) REFERENCES units(id),
--   FOREIGN KEY(star_system_id) REFERENCES star_systems(id)
-- );


-- Why This Schema Works Long-Term
-- Planet → Space Transition
-- DELETE FROM unit_planet_locations
-- INSERT INTO unit_space_locations
-- UPDATE units.location_type = 'SPACE'


-- Interstellar Travel Later
-- You will add:
-- CREATE TABLE unit_ftl_travel (
--   unit_id INTEGER PRIMARY KEY,
--   from_system INTEGER NOT NULL,
--   to_system INTEGER NOT NULL,
--   progress REAL NOT NULL
-- );


-- Indexes You Should Add Early
-- CREATE INDEX idx_units_player ON units(player_id);
-- CREATE INDEX idx_planet_location_planet ON unit_planet_locations(planet_id);
-- CREATE INDEX idx_move_orders_unit ON move_orders(unit_id);


-- seed two players + units (for prototype)
-- INSERT OR IGNORE INTO players (id, name) VALUES (1, 'Player 1');
-- INSERT OR IGNORE INTO players (id, name) VALUES (2, 'Player 2');

-- place player 1 at (2,2) and player 2 at (5,5)
-- INSERT OR IGNORE INTO units (id, player_id, x, y) VALUES (1, 1, 2, 2);
-- INSERT OR IGNORE INTO units (id, player_id, x, y) VALUES (2, 2, 5, 5);


-- https://chatgpt.com/c/694dd80b-260c-8331-8f03-ce07eb2c7307
-- problem : If the client receives canonical coordinates (face, u, v), the client can reconstruct absolute planetary geography and infer the seed.
-- solution 1 (recommended): player-local coordinate frames
-- For each player–planet pair, define a local frame:
-- An arbitrary origin (face₀, u₀, v₀)
-- An arbitrary rotation / orientation
-- CREATE TABLE player_planet_frames (
--   player_id INTEGER NOT NULL,
--   planet_id INTEGER NOT NULL,
--   origin_face INTEGER NOT NULL,
--   origin_u INTEGER NOT NULL,
--   origin_v INTEGER NOT NULL,
--   rotation INTEGER NOT NULL, -- 0..5 for face orientation, or quaternion index
--   PRIMARY KEY (player_id, planet_id)
-- );


INSERT INTO players (id, name) VALUES
  (1, 'Player One'),
  (2, 'Player Two');


INSERT INTO galaxies (id, seed) VALUES
  (1, 123456),
  (2, 987654);


INSERT INTO star_systems (id, galaxy_id, gx, gy, gz, seed) VALUES
  (1, 1, 0, 0, 0, 111),
  (2, 1, 1, 0, 0, 222),
  (3, 2, 0, 1, 0, 333);


INSERT INTO planets (
  id,
  star_system_id,
  orbit_index,
  radius,
  subdivision,
  seed
) VALUES
  (1, 1, 0, 6371, 3, 1001),
  (2, 1, 1, 3390, 3, 1002),
  (3, 2, 0, 7000, 4, 2001);


INSERT INTO units (id, player_id, unit_type, location_type) VALUES
  (1, 1, 'INFANTRY', 'PLANET_SURFACE'),
  (2, 1, 'TANK',     'PLANET_SURFACE'),
  (3, 1, 'FLEET',    'ORBIT'),
  (4, 1, 'SCOUT',    'SPACE');


INSERT INTO unit_planet_locations (unit_id, planet_id, face, u, v) VALUES
  (1, 1, 0, 10, 5),
  (2, 1, 2, 3, 7);