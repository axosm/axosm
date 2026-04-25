20 tables, 44 indexes, validated clean through SQLite. Here's a quick map of what's in there:
Universe — galaxies → star_systems → planets → planet_tiles, plus space_objects for dynamic asteroid/pirate spawns.
Influence & territory — planet_tiles owns the tile with owner_player_id and a dirty flag. tile_influence stores the per-player score per tile. Background job recalculates when influence_recalc_needed = 1.
Buildings — building_types (static config: HP, defence, influence power/radius, flying, loot, production) and buildings (instances with HP, flight_state, destroyed_at, under_attack).
Units & movement — unit_types (stats + combat bonuses JSON), units (instances with location), move_orders (shared by units and flying buildings), retreat_orders.
Combat — battles (active fights, tracks phase: vs_units → vs_building), battle_reports (permanent log with unit snapshots and loot).
Resources — player_resources (pooled per player), unit_cargo (what a formation carries during loot_and_retreat).
Players — players, empires, empire_members.
When you're ready to port to Postgres the main changes will be: INTEGER → SERIAL/BIGSERIAL for PKs, TEXT timestamps → TIMESTAMPTZ, partial indexes stay identical, and you can replace the JSON columns with JSONB. Nothing structural needs to change.