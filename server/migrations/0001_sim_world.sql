-- Persistent ship world. The older players / entities / game_state tables are unused.

CREATE TABLE IF NOT EXISTS world_meta (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    seed BIGINT NOT NULL,
    last_tick_unix BIGINT NOT NULL,
    solar_kw DOUBLE PRECISION NOT NULL,
    next_job_id INTEGER NOT NULL,
    power_json TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS decks (
    index SMALLINT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS rooms (
    id INTEGER PRIMARY KEY,
    deck SMALLINT NOT NULL REFERENCES decks(index),
    kind TEXT NOT NULL,
    name TEXT NOT NULL,
    polygon JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS doors (
    id INTEGER PRIMARY KEY,
    from_room INTEGER NOT NULL REFERENCES rooms(id),
    to_room INTEGER NOT NULL REFERENCES rooms(id)
);

CREATE TABLE IF NOT EXISTS stations (
    id INTEGER PRIMARY KEY,
    room_id INTEGER NOT NULL REFERENCES rooms(id),
    recipe TEXT
);

CREATE TABLE IF NOT EXISTS crew (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    department TEXT NOT NULL,
    skill TEXT NOT NULL,
    room_id INTEGER NOT NULL REFERENCES rooms(id),
    x DOUBLE PRECISION NOT NULL,
    y DOUBLE PRECISION NOT NULL,
    deck SMALLINT NOT NULL,
    hunger DOUBLE PRECISION NOT NULL,
    thirst DOUBLE PRECISION NOT NULL,
    fatigue DOUBLE PRECISION NOT NULL,
    health DOUBLE PRECISION NOT NULL,
    activity TEXT NOT NULL,
    berth_id INTEGER NOT NULL REFERENCES rooms(id),
    station_id INTEGER REFERENCES stations(id),
    watch_offset_min INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS stocks (
    room_id INTEGER NOT NULL REFERENCES rooms(id),
    good TEXT NOT NULL,
    qty DOUBLE PRECISION NOT NULL,
    PRIMARY KEY (room_id, good)
);

CREATE TABLE IF NOT EXISTS jobs (
    id INTEGER PRIMARY KEY,
    station_id INTEGER NOT NULL REFERENCES stations(id),
    recipe TEXT NOT NULL,
    elapsed_seconds DOUBLE PRECISION NOT NULL
);

CREATE TABLE IF NOT EXISTS crops (
    room_id INTEGER PRIMARY KEY REFERENCES rooms(id),
    progress_days DOUBLE PRECISION NOT NULL,
    area_m2 DOUBLE PRECISION NOT NULL
);
