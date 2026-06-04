-- Cell-grid simulation schema (replaces legacy players/entities tables).

-- Material lookup tables
CREATE TABLE wall_material (
    id SMALLSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE floor_material (
    id SMALLSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE ceiling_material (
    id SMALLSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO wall_material (name) VALUES
    ('open'),
    ('marine_panel'),
    ('door'),
    ('window');

INSERT INTO floor_material (name) VALUES
    ('carpet'),
    ('wood');

INSERT INTO ceiling_material (name) VALUES
    ('open'),
    ('marine_panel');

-- Singleton game clock (starts at 0 on deploy)
CREATE TABLE game_state (
    id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    game_time_seconds BIGINT NOT NULL DEFAULT 0,
    last_tick_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO game_state (id, game_time_seconds) VALUES (1, 0)
ON CONFLICT (id) DO NOTHING;

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_game_state_updated_at
    BEFORE UPDATE ON game_state
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Sea cells (sparse: only occupied cells in the ship)
CREATE TABLE cell (
    x INT NOT NULL,
    y INT NOT NULL,
    z INT NOT NULL,
    bow_wall SMALLINT NOT NULL REFERENCES wall_material (id),
    stern_wall SMALLINT NOT NULL REFERENCES wall_material (id),
    port_wall SMALLINT NOT NULL REFERENCES wall_material (id),
    starboard_wall SMALLINT NOT NULL REFERENCES wall_material (id),
    floor SMALLINT NOT NULL REFERENCES floor_material (id),
    ceiling SMALLINT NOT NULL REFERENCES ceiling_material (id),
    PRIMARY KEY (x, y, z),
    CHECK (x BETWEEN 0 AND 359),
    CHECK (y BETWEEN 0 AND 59),
    CHECK (z BETWEEN 0 AND 19)
);

CREATE INDEX idx_cell_z ON cell (z);

-- Entity types and entities
CREATE TABLE entity_type (
    name TEXT PRIMARY KEY
);

INSERT INTO entity_type (name) VALUES ('human'), ('sim_human');

CREATE SEQUENCE entity_id_seq START WITH 1;

CREATE TABLE entity (
    id BIGINT PRIMARY KEY DEFAULT nextval('entity_id_seq'),
    x INT NOT NULL,
    y INT NOT NULL,
    z INT NOT NULL,
    entity_type TEXT NOT NULL REFERENCES entity_type (name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (x, y, z) REFERENCES cell (x, y, z)
);

CREATE INDEX idx_entity_xyz ON entity (x, y, z);
CREATE INDEX idx_entity_type ON entity (entity_type);

CREATE TRIGGER update_entity_updated_at
    BEFORE UPDATE ON entity
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
