-- Game state table for global game data
-- Stores singleton game state including game time
CREATE TABLE IF NOT EXISTS game_state (
    id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),  -- Ensures only one row exists
    game_time_minutes INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert the initial game state row
INSERT INTO game_state (id, game_time_minutes) 
VALUES (1, 0)
ON CONFLICT (id) DO NOTHING;

-- Trigger to auto-update updated_at
CREATE TRIGGER update_game_state_updated_at BEFORE UPDATE ON game_state
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

