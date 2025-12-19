--https://chatgpt.com/c/6945cd39-b0e4-8327-bafb-a8d62d309825

-- 20251001_create_tables.sql
CREATE TABLE IF NOT EXISTS players (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS units (
  id INTEGER PRIMARY KEY,
  player_id INTEGER NOT NULL,
  x INTEGER NOT NULL,
  y INTEGER NOT NULL,
  FOREIGN KEY(player_id) REFERENCES players(id)
);

CREATE TABLE IF NOT EXISTS move_orders (
  id INTEGER PRIMARY KEY,
  unit_id INTEGER NOT NULL,
  from_x INTEGER NOT NULL,
  from_y INTEGER NOT NULL,
  to_x INTEGER NOT NULL,
  to_y INTEGER NOT NULL,
  arrival_time INTEGER NOT NULL,
  FOREIGN KEY(unit_id) REFERENCES units(id)
);

-- seed two players + units (for prototype)
INSERT OR IGNORE INTO players (id, name) VALUES (1, 'Player 1');
INSERT OR IGNORE INTO players (id, name) VALUES (2, 'Player 2');

-- place player 1 at (2,2) and player 2 at (5,5)
INSERT OR IGNORE INTO units (id, player_id, x, y) VALUES (1, 1, 2, 2);
INSERT OR IGNORE INTO units (id, player_id, x, y) VALUES (2, 2, 5, 5);
