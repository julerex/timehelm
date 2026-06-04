# Deploy cell-grid simulation (PostgreSQL)

## Local database reset

```bash
docker run --name timehelm-db -e POSTGRES_PASSWORD=password -e POSTGRES_DB=timehelm -p 5432:5432 -d postgres:15
export DATABASE_URL=postgresql://postgres:password@localhost:5432/timehelm
cd server && env -u ARGV0 cargo run
```

Migrations run automatically on server start (`001`, `002`, `003_seed_cells`).

Regenerate seed SQL after layout changes:

```bash
make seed-cells
```

## Fly.io production reset

Requires `schema_admin` / `fly mpg connect`:

```sql
DROP SCHEMA public CASCADE;
CREATE SCHEMA public;
GRANT ALL ON SCHEMA public TO PUBLIC;
```

Then deploy the app (migrations run on boot). Seed migration `003_seed_cells.sql` is large (~220k INSERTs); first boot may take several minutes.

```bash
make deploy
make fly-logs
```

Verify:

- `curl https://timehelm.net/api/entities`
- `curl https://timehelm.net/api/decks/4/cells` (first lines)
- Open https://timehelm.net/3d/ — WASM loads cells from `/api/cells` and WebSocket `/ws`

`game_time_seconds` starts at **0** on each fresh deploy.
