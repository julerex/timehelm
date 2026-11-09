# Time Helm

Time Helm is a persistent simulation of a self-sustaining community ship.

The ship is a set of rooms on a 318 m × 60 m hull, not a lattice of 1 m cells. About 80 named crew work watches, walk between rooms, and run a closed material economy: bread, meals, fresh water, crops, fishing, cloth, timber, metal, and a small clinic. Solar power is a fixed average. When it runs short, grow lights and desalination throttle before the galley and the clinic.

Time runs at **1 real second = 1 sim second**. The server keeps the world in PostgreSQL and catches up any missed minutes when it starts, including while nobody is watching. The Bevy client is a viewer. It can orbit the ship, change decks, and read the keymap. It cannot change crew, stocks, or recipes. The command path is in place so a later participant role can be added without a second world model.

Fidelity here means conserved masses, real durations, and labour. The picture stays low-poly.

Production: https://timehelm.net/

## Run

Requirements: Rust 1.92, the `wasm32-unknown-unknown` target, `wasm-bindgen-cli` 0.2.130, and PostgreSQL.

```bash
# server/.env
DATABASE_URL=postgresql://postgres:password@localhost:5432/timehelm
```

```bash
make run-ship
```

Open http://localhost:8080/ . The root redirects to `/3d/`. `/2d` and `/seacells` redirect there as well.

Without `DATABASE_URL` the server still starts and the viewer says the world is unavailable. It does not invent an unsaved ship.

Native window, talking to the same server:

```bash
make run-ship-native
```

## Look around

| Action | Binding |
| --- | --- |
| Pan | W A S D |
| Orbit | Q / E, or right-drag |
| Lift | R / F |
| Dolly | Z / X, or the scroll wheel |
| Pan with the mouse | Middle-drag |
| Deck | Page Up / Page Down |
| Plan | P |
| Keymap | K |

Interaction is listed in the keymap and is not available to viewers.

The top bar shows the simulation clock in UTC, the deck, crew count, food and water in days, and power for the current minute. P switches to a top-down plan of the current deck, where berths show a bed and heads show a toilet and a shower. Hover a room for its name, stocks, and who is inside.

## Layout

| Path | Role |
| --- | --- |
| `sim/` | Hull, rooms, crew, recipes, and the 60-second step. No Bevy. |
| `server/` | Load, catch up, save, and stream the world. |
| `ship-game/` | Bevy 0.18 viewer. |
| `docs/SIMULATION.md` | Rooms, goods, recipes, power priority, and the `Role` / `Command` seam. |
| `client/public/` | Pages and the built WASM viewer. |

The repository is one Cargo workspace. `make test` and `make lint` cover `sim`, `server`, and `ship-game`.

## Later

A participant will be another `Role` on the same `Command` enum. Stations already have recipe slots, and the viewer already has an unbound interaction action. Indirect control — schedules and conditions, rather than direct puppeting — can land on that seam. The current build does not do it.
