-- Users table (from Twitter OAuth) - COMMENTED OUT FOR NOW
-- CREATE TABLE IF NOT EXISTS users (
--     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
--     twitter_id VARCHAR(255) UNIQUE NOT NULL,
--     username VARCHAR(255) NOT NULL,
--     display_name VARCHAR(255) NOT NULL,
--     avatar_url TEXT,
--     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
--     updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
-- );

-- Sessions table - COMMENTED OUT FOR NOW
-- CREATE TABLE IF NOT EXISTS sessions (
--     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
--     user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
--     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
--     expires_at TIMESTAMPTZ NOT NULL,
--     UNIQUE(id)
-- );

-- CREATE INDEX idx_sessions_user_id ON sessions(user_id);
-- CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);

-- Players table (game state)
CREATE TABLE IF NOT EXISTS players (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255) NOT NULL,
    position_x FLOAT NOT NULL DEFAULT 0.0,
    position_y FLOAT NOT NULL DEFAULT 0.0,
    position_z FLOAT NOT NULL DEFAULT 0.0,
    rotation FLOAT NOT NULL DEFAULT 0.0,
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers to auto-update updated_at
-- CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
--     FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_players_updated_at BEFORE UPDATE ON players
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

