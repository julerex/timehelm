-- PL/pgSQL simulation: movement, wander, and 1 Hz tick.
-- Included after schema migration (material seeds must exist).

CREATE OR REPLACE FUNCTION wall_passable(material_id SMALLINT)
RETURNS BOOLEAN
LANGUAGE sql
STABLE
AS $$
    SELECT EXISTS (
        SELECT 1 FROM wall_material WHERE id = material_id AND name = 'open'
    );
$$;

CREATE OR REPLACE FUNCTION can_step(
    from_x INT,
    from_y INT,
    from_z INT,
    to_x INT,
    to_y INT,
    to_z INT
)
RETURNS BOOLEAN
LANGUAGE plpgsql
STABLE
AS $$
DECLARE
    dx INT := to_x - from_x;
    dy INT := to_y - from_y;
    dz INT := to_z - from_z;
    fc RECORD;
    tc RECORD;
BEGIN
    IF dz <> 0 OR ABS(dx) + ABS(dy) <> 1 THEN
        RETURN FALSE;
    END IF;

    SELECT bow_wall, stern_wall, port_wall, starboard_wall
    INTO fc
    FROM cell
    WHERE x = from_x AND y = from_y AND z = from_z;

    IF NOT FOUND THEN
        RETURN FALSE;
    END IF;

    SELECT bow_wall, stern_wall, port_wall, starboard_wall
    INTO tc
    FROM cell
    WHERE x = to_x AND y = to_y AND z = to_z;

    IF NOT FOUND THEN
        RETURN FALSE;
    END IF;

    -- side1=bow(+x), side2=port(+y), side3=stern(-x), side4=starboard(-y)
    IF dx = 1 AND dy = 0 THEN
        RETURN wall_passable(fc.bow_wall) AND wall_passable(tc.stern_wall);
    ELSIF dx = -1 AND dy = 0 THEN
        RETURN wall_passable(fc.stern_wall) AND wall_passable(tc.bow_wall);
    ELSIF dx = 0 AND dy = 1 THEN
        RETURN wall_passable(fc.port_wall) AND wall_passable(tc.starboard_wall);
    ELSIF dx = 0 AND dy = -1 THEN
        RETURN wall_passable(fc.starboard_wall) AND wall_passable(tc.port_wall);
    END IF;

    RETURN FALSE;
END;
$$;

CREATE OR REPLACE FUNCTION move_entity(
    p_entity_id BIGINT,
    to_x INT,
    to_y INT,
    to_z INT
)
RETURNS BOOLEAN
LANGUAGE plpgsql
AS $$
DECLARE
    from_x INT;
    from_y INT;
    from_z INT;
BEGIN
    SELECT x, y, z INTO from_x, from_y, from_z
    FROM entity
    WHERE id = p_entity_id
    FOR UPDATE;

    IF NOT FOUND THEN
        RETURN FALSE;
    END IF;

    IF from_x = to_x AND from_y = to_y AND from_z = to_z THEN
        RETURN TRUE;
    END IF;

    IF NOT can_step(from_x, from_y, from_z, to_x, to_y, to_z) THEN
        RETURN FALSE;
    END IF;

    UPDATE entity
    SET x = to_x, y = to_y, z = to_z, updated_at = NOW()
    WHERE id = p_entity_id;

    RETURN TRUE;
END;
$$;

CREATE OR REPLACE FUNCTION wander_sim_humans()
RETURNS VOID
LANGUAGE plpgsql
AS $$
DECLARE
    rec RECORD;
    dirs CONSTANT INT[][] := ARRAY[
        ARRAY[1, 0],
        ARRAY[-1, 0],
        ARRAY[0, 1],
        ARRAY[0, -1]
    ];
    candidates INT[][];
    d INT[];
    i INT;
    nx INT;
    ny INT;
    chosen INT[];
BEGIN
    FOR rec IN
        SELECT id, x, y, z FROM entity WHERE entity_type = 'sim_human'
    LOOP
        candidates := ARRAY[]::INT[][];

        FOR i IN 1..4 LOOP
            d := dirs[i];
            nx := rec.x + d[1];
            ny := rec.y + d[2];
            IF can_step(rec.x, rec.y, rec.z, nx, ny, rec.z) THEN
                candidates := candidates || ARRAY[ARRAY[nx, ny]];
            END IF;
        END LOOP;

        IF array_length(candidates, 1) IS NULL THEN
            CONTINUE;
        END IF;

        chosen := candidates[1 + floor(random() * array_length(candidates, 1))::INT];
        PERFORM move_entity(rec.id, chosen[1], chosen[2], rec.z);
    END LOOP;
END;
$$;

CREATE OR REPLACE FUNCTION sim_tick()
RETURNS BIGINT
LANGUAGE plpgsql
AS $$
DECLARE
    new_time BIGINT;
BEGIN
    UPDATE game_state
    SET
        game_time_seconds = game_time_seconds + 1,
        last_tick_at = NOW(),
        updated_at = NOW()
    WHERE id = 1
    RETURNING game_time_seconds INTO new_time;

    PERFORM wander_sim_humans();

    RETURN new_time;
END;
$$;

CREATE OR REPLACE FUNCTION get_game_time_seconds()
RETURNS BIGINT
LANGUAGE sql
STABLE
AS $$
    SELECT game_time_seconds FROM game_state WHERE id = 1;
$$;
