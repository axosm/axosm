
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS btree_gist;

CREATE TABLE units (
  ...,
  star_system_id     INTEGER REFERENCES star_systems(id),
  star_system_pos GEOMETRY(PointZ, 0),
  ...
)


CREATE INDEX idx_units_space_pos ON units USING GIST(system_id, space_pos)
  WHERE location_mode = 'in_space';

  -- usage WHERE system_id = ? AND ST_DWithin(space_pos, ...) AND location_mode = 'in_space'