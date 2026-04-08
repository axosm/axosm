-- ============================================================
-- STELLARUM IMPERIUM — PostgreSQL Schema
-- Version 0.1 (matches GDD v0.1)
-- ============================================================

-- ─────────────────────────────────────────────────────────────
-- 1. USERS & AUTHENTICATION
-- ─────────────────────────────────────────────────────────────

CREATE TABLE users (
    id              BIGSERIAL       PRIMARY KEY,
    username        VARCHAR(64)     NOT NULL UNIQUE,
    email           VARCHAR(255)    NOT NULL UNIQUE,
    password_hash   VARCHAR(255)    NOT NULL,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    last_login_at   TIMESTAMPTZ
);

-- ─────────────────────────────────────────────────────────────
-- 2. UNIVERSE & SPACE
-- ─────────────────────────────────────────────────────────────

CREATE TABLE galaxies (
    id          BIGSERIAL       PRIMARY KEY,
    seed        BIGINT          NOT NULL,
    name        VARCHAR(128)    NOT NULL,
    radius      FLOAT           NOT NULL,
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE solar_systems (
    id          BIGSERIAL       PRIMARY KEY,
    galaxy_id   BIGINT          NOT NULL REFERENCES galaxies(id),
    seed        BIGINT          NOT NULL,
    name        VARCHAR(128)    NOT NULL,
    x           FLOAT           NOT NULL,
    y           FLOAT           NOT NULL,
    star_type   VARCHAR(32),                    -- G, K, M, etc.
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_solar_systems_galaxy ON solar_systems(galaxy_id);
CREATE INDEX idx_solar_systems_coords ON solar_systems USING gist (point(x, y));

CREATE TABLE planets (
    id                BIGSERIAL       PRIMARY KEY,
    solar_system_id   BIGINT          NOT NULL REFERENCES solar_systems(id),
    seed              BIGINT          NOT NULL,
    name              VARCHAR(128)    NOT NULL,
    orbital_slot      INT             NOT NULL,
    tile_count        INT             NOT NULL CHECK (tile_count BETWEEN 100 AND 10000),
    planet_type       VARCHAR(32),                -- terrestrial, gas, lava, ice
    created_at        TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_planets_system ON planets(solar_system_id);

CREATE TABLE planet_tiles (
    id                BIGSERIAL       PRIMARY KEY,
    planet_id         BIGINT          NOT NULL REFERENCES planets(id),
    tile_index        INT             NOT NULL,
    terrain_type      VARCHAR(32)     NOT NULL
                          CHECK (terrain_type IN ('plains','forest','mountain','water','lava','tundra','desert')),
    is_pentagon       BOOLEAN         NOT NULL DEFAULT FALSE,
    owner_empire_id   BIGINT,                     -- FK set after empires table; see ALTER below
    created_at        TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    UNIQUE (planet_id, tile_index)
);

CREATE INDEX idx_planet_tiles_planet   ON planet_tiles(planet_id);
CREATE INDEX idx_planet_tiles_owner    ON planet_tiles(owner_empire_id);

-- Goldberg sphere adjacency graph (undirected edges stored as directed pairs)
CREATE TABLE tile_adjacency (
    tile_id       BIGINT  NOT NULL REFERENCES planet_tiles(id),
    neighbor_id   BIGINT  NOT NULL REFERENCES planet_tiles(id),
    PRIMARY KEY (tile_id, neighbor_id),
    CHECK (tile_id <> neighbor_id)
);

CREATE INDEX idx_tile_adj_neighbor ON tile_adjacency(neighbor_id);

CREATE TABLE asteroids (
    id                BIGSERIAL       PRIMARY KEY,
    solar_system_id   BIGINT          NOT NULL REFERENCES solar_systems(id),
    x                 FLOAT           NOT NULL,
    y                 FLOAT           NOT NULL,
    size              VARCHAR(16)     NOT NULL CHECK (size IN ('small','large')),
    resources         JSONB,                       -- {resource_key: amount, ...}
    created_at        TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_asteroids_system  ON asteroids(solar_system_id);
CREATE INDEX idx_asteroids_coords  ON asteroids USING gist (point(x, y));

-- ─────────────────────────────────────────────────────────────
-- 3. EMPIRES & DIPLOMACY
-- ─────────────────────────────────────────────────────────────

CREATE TABLE empires (
    id                  BIGSERIAL       PRIMARY KEY,
    name                VARCHAR(128)    NOT NULL UNIQUE,
    founder_user_id     BIGINT          NOT NULL REFERENCES users(id),
    capital_planet_id   BIGINT,                    -- FK set after planets exist
    treasury            JSONB           NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

-- Now add FKs that required empires to exist first
ALTER TABLE planet_tiles
    ADD CONSTRAINT fk_planet_tiles_empire
    FOREIGN KEY (owner_empire_id) REFERENCES empires(id);

ALTER TABLE empires
    ADD CONSTRAINT fk_empires_capital
    FOREIGN KEY (capital_planet_id) REFERENCES planets(id);

CREATE TABLE empire_members (
    id            BIGSERIAL       PRIMARY KEY,
    empire_id     BIGINT          NOT NULL REFERENCES empires(id),
    user_id       BIGINT          NOT NULL REFERENCES users(id),
    role          VARCHAR(64)     NOT NULL DEFAULT 'member',  -- emperor, general, diplomat, member
    joined_at     TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    created_at    TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    UNIQUE (empire_id, user_id)
);

CREATE TABLE diplomatic_relations (
    id                BIGSERIAL       PRIMARY KEY,
    empire_a_id       BIGINT          NOT NULL REFERENCES empires(id),
    empire_b_id       BIGINT          NOT NULL REFERENCES empires(id),
    status            VARCHAR(32)     NOT NULL DEFAULT 'neutral'
                          CHECK (status IN ('war','peace','alliance','neutral','hostile')),
    declared_at       TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    created_at        TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    UNIQUE (empire_a_id, empire_b_id),
    CHECK (empire_a_id < empire_b_id)   -- canonical ordering avoids duplicate pairs
);

CREATE TABLE alliances (
    id            BIGSERIAL       PRIMARY KEY,
    name          VARCHAR(128),
    formed_at     TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    terms         JSONB,                               -- non-aggression, mutual_defense, resource_sharing, etc.
    created_at    TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE alliance_members (
    alliance_id   BIGINT  NOT NULL REFERENCES alliances(id),
    empire_id     BIGINT  NOT NULL REFERENCES empires(id),
    joined_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (alliance_id, empire_id)
);

-- ─────────────────────────────────────────────────────────────
-- 4. RESOURCES
-- ─────────────────────────────────────────────────────────────

CREATE TABLE resource_types (
    id                BIGSERIAL       PRIMARY KEY,
    key               VARCHAR(64)     NOT NULL UNIQUE,   -- e.g. 'wood', 'uranium'
    name              VARCHAR(128)    NOT NULL,
    era               VARCHAR(32)     NOT NULL,          -- prehistoric … interstellar
    is_energy         BOOLEAN         NOT NULL DEFAULT FALSE,
    required_tech_id  BIGINT,                            -- FK added after technologies table
    created_at        TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE planet_resources (
    id                BIGSERIAL   PRIMARY KEY,
    planet_id         BIGINT      NOT NULL REFERENCES planets(id),
    empire_id         BIGINT      NOT NULL REFERENCES empires(id),
    resource_type_id  BIGINT      NOT NULL REFERENCES resource_types(id),
    amount            BIGINT      NOT NULL DEFAULT 0 CHECK (amount >= 0),
    capacity          BIGINT      NOT NULL DEFAULT 0 CHECK (capacity >= 0),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (planet_id, empire_id, resource_type_id)
);

CREATE INDEX idx_planet_resources_planet ON planet_resources(planet_id);
CREATE INDEX idx_planet_resources_empire ON planet_resources(empire_id);

-- ─────────────────────────────────────────────────────────────
-- 5. BUILDINGS
-- ─────────────────────────────────────────────────────────────

CREATE TABLE building_types (
    id                    BIGSERIAL       PRIMARY KEY,
    key                   VARCHAR(64)     NOT NULL UNIQUE,
    name                  VARCHAR(128)    NOT NULL,
    category              VARCHAR(64)     NOT NULL,
        -- extraction | storage | population | production | research | defense | infrastructure | special
    base_build_time       INT             NOT NULL,   -- seconds
    base_cost             JSONB           NOT NULL DEFAULT '{}',
    base_output           JSONB           NOT NULL DEFAULT '{}',
    max_level             INT             NOT NULL DEFAULT 10,
    allowed_terrains      JSONB           NOT NULL DEFAULT '[]',  -- list of terrain_type strings
    required_tech_id      BIGINT,
    created_at            TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE tile_buildings (
    id                BIGSERIAL   PRIMARY KEY,
    planet_tile_id    BIGINT      NOT NULL UNIQUE REFERENCES planet_tiles(id),  -- one building per tile
    empire_id         BIGINT      NOT NULL REFERENCES empires(id),
    building_type_id  BIGINT      NOT NULL REFERENCES building_types(id),
    level             INT         NOT NULL DEFAULT 1 CHECK (level >= 1),
    hp                INT         NOT NULL,
    max_hp            INT         NOT NULL,
    workers_assigned  INT         NOT NULL DEFAULT 0 CHECK (workers_assigned >= 0),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tile_buildings_empire ON tile_buildings(empire_id);
CREATE INDEX idx_tile_buildings_type   ON tile_buildings(building_type_id);

-- ─────────────────────────────────────────────────────────────
-- 6. RESEARCH / TECHNOLOGY
-- ─────────────────────────────────────────────────────────────

CREATE TABLE technologies (
    id                  BIGSERIAL       PRIMARY KEY,
    key                 VARCHAR(64)     NOT NULL UNIQUE,   -- e.g. 'nuclear_fission'
    name                VARCHAR(128)    NOT NULL,
    era                 VARCHAR(32)     NOT NULL,
    category            VARCHAR(64),    -- military, economic, science, diplomatic
    base_research_time  INT             NOT NULL,          -- seconds at base research power
    base_cost           JSONB           NOT NULL DEFAULT '{}',
    server_age_gate     INT             NOT NULL DEFAULT 0, -- seconds of server runtime required
    effects             JSONB           NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

-- Now back-fill the FK from resource_types
ALTER TABLE resource_types
    ADD CONSTRAINT fk_resource_types_tech
    FOREIGN KEY (required_tech_id) REFERENCES technologies(id);

ALTER TABLE building_types
    ADD CONSTRAINT fk_building_types_tech
    FOREIGN KEY (required_tech_id) REFERENCES technologies(id);

CREATE TABLE technology_prerequisites (
    tech_id       BIGINT  NOT NULL REFERENCES technologies(id),
    requires_id   BIGINT  NOT NULL REFERENCES technologies(id),
    PRIMARY KEY (tech_id, requires_id),
    CHECK (tech_id <> requires_id)
);

CREATE TABLE player_research (
    id            BIGSERIAL   PRIMARY KEY,
    user_id       BIGINT      NOT NULL REFERENCES users(id),
    tech_id       BIGINT      NOT NULL REFERENCES technologies(id),
    status        VARCHAR(16) NOT NULL DEFAULT 'in_progress'
                      CHECK (status IN ('in_progress','completed')),
    started_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completes_at  TIMESTAMPTZ NOT NULL,
    completed_at  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, tech_id)
);

CREATE INDEX idx_player_research_user   ON player_research(user_id);
CREATE INDEX idx_player_research_status ON player_research(status);

-- ─────────────────────────────────────────────────────────────
-- 7. UNITS
-- ─────────────────────────────────────────────────────────────

CREATE TABLE unit_types (
    id                BIGSERIAL       PRIMARY KEY,
    key               VARCHAR(64)     NOT NULL UNIQUE,
    name              VARCHAR(128)    NOT NULL,
    unit_class        VARCHAR(32)     NOT NULL,  -- ground | ship
    ship_class        VARCHAR(32),               -- scout | fighter | frigate | destroyer | cruiser | battleship | carrier | transport | colony | station
    attack            INT             NOT NULL DEFAULT 0,
    defense           INT             NOT NULL DEFAULT 0,
    hp_per_unit       INT             NOT NULL DEFAULT 1,
    speed             INT             NOT NULL DEFAULT 1,
    cargo             INT             NOT NULL DEFAULT 0,
    cloak_rating      INT             NOT NULL DEFAULT 0,
    scan_power        INT             NOT NULL DEFAULT 0,
    orbit_capable     BOOLEAN         NOT NULL DEFAULT FALSE,
    required_tech_id  BIGINT          REFERENCES technologies(id),
    created_at        TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE unit_stacks (
    id                BIGSERIAL   PRIMARY KEY,
    unit_type_id      BIGINT      NOT NULL REFERENCES unit_types(id),
    owner_empire_id   BIGINT      NOT NULL REFERENCES empires(id),
    owner_user_id     BIGINT      NOT NULL REFERENCES users(id),
    quantity          INT         NOT NULL DEFAULT 1 CHECK (quantity > 0),
    location_type     VARCHAR(16) NOT NULL
                          CHECK (location_type IN ('planet_tile','orbit_low','orbit_high','space')),
    planet_tile_id    BIGINT      REFERENCES planet_tiles(id),
    planet_id         BIGINT      REFERENCES planets(id),
    space_x           FLOAT,
    space_y           FLOAT,
    solar_system_id   BIGINT      REFERENCES solar_systems(id),
    aggression        VARCHAR(16) NOT NULL DEFAULT 'neutral'
                          CHECK (aggression IN ('aggressive','neutral','defensive')),
    hp                INT         NOT NULL,
    cargo_contents    JSONB       NOT NULL DEFAULT '{}',
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_unit_stacks_empire  ON unit_stacks(owner_empire_id);
CREATE INDEX idx_unit_stacks_tile    ON unit_stacks(planet_tile_id);
CREATE INDEX idx_unit_stacks_planet  ON unit_stacks(planet_id);
CREATE INDEX idx_unit_stacks_coords  ON unit_stacks USING gist (point(space_x, space_y))
    WHERE space_x IS NOT NULL;

-- ─────────────────────────────────────────────────────────────
-- 8. MOVEMENT ORDERS
-- ─────────────────────────────────────────────────────────────

CREATE TABLE movement_orders (
    id              BIGSERIAL   PRIMARY KEY,
    unit_stack_id   BIGINT      NOT NULL REFERENCES unit_stacks(id),
    origin_type     VARCHAR(16) NOT NULL
                        CHECK (origin_type IN ('planet_tile','orbit_low','orbit_high','space')),
    dest_type       VARCHAR(16) NOT NULL
                        CHECK (dest_type IN ('planet_tile','orbit_low','orbit_high','space')),
    dest_tile_id    BIGINT      REFERENCES planet_tiles(id),
    dest_planet_id  BIGINT      REFERENCES planets(id),
    dest_x          FLOAT,
    dest_y          FLOAT,
    dest_system_id  BIGINT      REFERENCES solar_systems(id),
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    arrives_at      TIMESTAMPTZ NOT NULL,
    status          VARCHAR(16) NOT NULL DEFAULT 'in_transit'
                        CHECK (status IN ('in_transit','arrived','cancelled')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_movement_orders_stack   ON movement_orders(unit_stack_id);
CREATE INDEX idx_movement_orders_arrives ON movement_orders(arrives_at) WHERE status = 'in_transit';

-- ─────────────────────────────────────────────────────────────
-- 9. COMBAT
-- ─────────────────────────────────────────────────────────────

CREATE TABLE battles (
    id                BIGSERIAL   PRIMARY KEY,
    arena_type        VARCHAR(16) NOT NULL
                          CHECK (arena_type IN ('planet_tile','orbit_low','orbit_high','space')),
    location_ref      BIGINT,                         -- tile_id, planet_id, or spatial object id
    solar_system_id   BIGINT      REFERENCES solar_systems(id),
    space_x           FLOAT,
    space_y           FLOAT,
    status            VARCHAR(16) NOT NULL DEFAULT 'active'
                          CHECK (status IN ('active','resolved','ceasefire')),
    started_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at       TIMESTAMPTZ,
    round_number      INT         NOT NULL DEFAULT 0,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_battles_status ON battles(status) WHERE status = 'active';

CREATE TABLE battle_participants (
    id              BIGSERIAL   PRIMARY KEY,
    battle_id       BIGINT      NOT NULL REFERENCES battles(id),
    unit_stack_id   BIGINT      NOT NULL REFERENCES unit_stacks(id),
    empire_id       BIGINT      NOT NULL REFERENCES empires(id),
    side            VARCHAR(8)  NOT NULL CHECK (side IN ('attacker','defender')),
    initial_qty     INT         NOT NULL,
    current_qty     INT         NOT NULL,
    current_hp      INT         NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_battle_participants_battle ON battle_participants(battle_id);

CREATE TABLE battle_rounds (
    id            BIGSERIAL   PRIMARY KEY,
    battle_id     BIGINT      NOT NULL REFERENCES battles(id),
    round_number  INT         NOT NULL,
    resolved_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    damage_log    JSONB       NOT NULL DEFAULT '{}',
    losses_log    JSONB       NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (battle_id, round_number)
);

CREATE INDEX idx_battle_rounds_battle ON battle_rounds(battle_id);

-- ─────────────────────────────────────────────────────────────
-- 10. SPACE STATIONS & STARGATES
-- ─────────────────────────────────────────────────────────────

CREATE TABLE space_stations (
    id                BIGSERIAL   PRIMARY KEY,
    owner_empire_id   BIGINT      NOT NULL REFERENCES empires(id),
    solar_system_id   BIGINT      NOT NULL REFERENCES solar_systems(id),
    asteroid_id       BIGINT      REFERENCES asteroids(id),
    x                 FLOAT,
    y                 FLOAT,
    hp                INT         NOT NULL,
    max_hp            INT         NOT NULL,
    modules           JSONB       NOT NULL DEFAULT '[]',
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_space_stations_empire ON space_stations(owner_empire_id);
CREATE INDEX idx_space_stations_system ON space_stations(solar_system_id);

CREATE TABLE stargates (
    id                BIGSERIAL   PRIMARY KEY,
    owner_empire_id   BIGINT      NOT NULL REFERENCES empires(id),
    solar_system_id   BIGINT      NOT NULL REFERENCES solar_systems(id),
    x                 FLOAT       NOT NULL,
    y                 FLOAT       NOT NULL,
    linked_gate_id    BIGINT      REFERENCES stargates(id),   -- self-referential
    hp                INT         NOT NULL,
    is_active         BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_stargates_empire ON stargates(owner_empire_id);
CREATE INDEX idx_stargates_system ON stargates(solar_system_id);

-- ─────────────────────────────────────────────────────────────
-- 11. CONSTRUCTION QUEUE
-- ─────────────────────────────────────────────────────────────

CREATE TABLE construction_queue (
    id                    BIGSERIAL   PRIMARY KEY,
    empire_id             BIGINT      NOT NULL REFERENCES empires(id),
    queue_type            VARCHAR(32) NOT NULL
                              CHECK (queue_type IN ('building','unit','ship','repair')),
    reference_id          BIGINT      NOT NULL,     -- ID of building_type, unit_type, etc.
    location_tile_id      BIGINT      REFERENCES planet_tiles(id),
    location_planet_id    BIGINT      REFERENCES planets(id),
    quantity              INT         NOT NULL DEFAULT 1 CHECK (quantity > 0),
    started_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completes_at          TIMESTAMPTZ NOT NULL,
    status                VARCHAR(16) NOT NULL DEFAULT 'queued'
                              CHECK (status IN ('queued','in_progress','done','cancelled')),
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_construction_queue_empire   ON construction_queue(empire_id);
CREATE INDEX idx_construction_queue_completes ON construction_queue(completes_at)
    WHERE status IN ('queued','in_progress');

-- ─────────────────────────────────────────────────────────────
-- 12. SERVER CONFIGURATION
-- ─────────────────────────────────────────────────────────────

CREATE TABLE server_config (
    key           VARCHAR(128)    PRIMARY KEY,
    value         TEXT            NOT NULL,
    description   TEXT,
    created_at    TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

-- Seed the authoritative server start time (set on first boot)
INSERT INTO server_config (key, value, description)
VALUES ('server_start_time', extract(epoch from NOW())::TEXT,
        'Unix timestamp of server launch; used to compute all tech gate windows');

-- ─────────────────────────────────────────────────────────────
-- 13. HELPER: updated_at trigger function (applies to all tables)
-- ─────────────────────────────────────────────────────────────

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

-- Generate triggers for every table with an updated_at column
DO $$
DECLARE
    t TEXT;
BEGIN
    FOR t IN
        SELECT table_name FROM information_schema.columns
        WHERE table_schema = 'public' AND column_name = 'updated_at'
        GROUP BY table_name
    LOOP
        EXECUTE format(
            'CREATE TRIGGER trg_%s_updated_at
             BEFORE UPDATE ON %I
             FOR EACH ROW EXECUTE FUNCTION set_updated_at()',
            t, t
        );
    END LOOP;
END;
$$;
