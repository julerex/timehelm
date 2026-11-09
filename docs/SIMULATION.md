# Ship simulation

The world is a `sim::World`. The server loads it from PostgreSQL, advances it, and saves it. The Bevy app only renders snapshots.

Time is 1:1 with the wall clock. `World::step` covers 60 seconds. On boot the server calls `catch_up_to` for every missed minute since `last_tick_unix`. The step is deterministic: fixed crew order, no unseeded random numbers, so a day of catch-up matches a day of live steps.

## Hull and decks

The plan is 318 m long and 60 m across. Origin is the aft-starboard corner, +X points forward, +Y points to port. Decks are 3 m apart. Deck index 0 is the lowest. Decks 9 and above keep the upper-deck courtyard void.

Rooms are named rectangles clipped to that deck's hull. A shared edge is a door. A corridor is a room. Crew walk the door graph at 1.3 m/s, in a straight line between room centroids (78 m per step).

| Deck | Name | Program |
| --- | --- | --- |
| 0 | Engine Deck | Engineering, electrical, fuel store |
| 1 | Tank Deck | Desalination, water store, waste treatment, recycling |
| 2 | Hold | Cold store, dry store, timber store, metal store |
| 3–4 | Garden decks | Hydroponics |
| 5 | Galley Deck | Galley, bakery |
| 6 | Mess Deck | Restaurant, mess |
| 7 | Hospital Deck | Clinic, infirmary |
| 8 | Carpentry Deck | Carpentry |
| 9 | Metal Deck | Metalwork |
| 10 | Textile Deck | Sewing, laundry |
| 11 | Fishing Deck | Fishing station |
| 12 | Bridge Deck | Bridge |
| 13–19 | Berth decks | Berths and heads |

Every deck has a stair core on the same footprint, plus a longitudinal corridor. Upper decks link port and starboard forward of the courtyard. Filler rooms of the deck's main kind pack the remaining hull.

`RoomKind` is an exhaustive enum: corridor, stair, berth, head, hydroponics, fishing, cold store, dry store, galley, bakery, restaurant, mess, desalination, water store, waste treatment, recycling, carpentry, sewing, metalwork, laundry, timber store, metal store, clinic, infirmary, engineering, electrical, fuel store, bridge.

## Crew

Eighty named people are seeded from a stored seed. They share 4-berth cabins. Departments and counts:

| Department | People |
| --- | --- |
| Engineering | 6 |
| Water | 4 |
| Waste | 4 |
| Galley | 6 |
| Bakery | 4 |
| Service | 8 |
| Hydroponics | 10 |
| Fishing | 4 |
| Clinic | 6 |
| Carpentry | 4 |
| Metalwork | 4 |
| Sewing | 4 |
| Laundry | 3 |
| Bridge | 4 |
| Relief | 9 |

A watch is 8 hours. Department `i` is offset by `i * 45` minutes, so the watches stagger. On shift, a person walks to their assigned station. Off shift, hunger above 0.6 sends them to the mess; otherwise they return to their berth. Health below 0.55 sends them to the clinic first.

Each minute, hunger and thirst rise by `1/1440` (capped at 1.5). Fatigue rises by `1/480` on shift and falls by `1/480` while sleeping in a berth. Health falls slowly when hunger exceeds 0.85 or thirst exceeds 0.85.

Eating requires hunger of at least 0.5 and a mess, restaurant, or galley. A portion is 0.45 kg of meals and relieves hunger by 0.4. Drinking takes 0.12 L from a water store when thirst is at least 0.35 and relieves thirst by 0.3. In the clinic, health below 0.9 consumes 0.02 kg of medical supplies and restores 0.05 health.

## Goods and recipes

Quantities are kilograms. Litres of water are treated as kilograms. Environmental sources (seawater) are not taken from a tank. Environmental sinks (brine, vapor, wastewater, process loss) are not stored.

Recipes are fixed data in `sim::recipes`. A job consumes its inputs when it starts and advances only while it is staffed and receives power. One batch:

| Recipe | Station | Time | Energy | Balance |
| --- | --- | --- | --- | --- |
| Bake bread | Bakery | 3.5 h | 2.0 kWh | 1.00 flour + 0.60 water + 0.02 salt + 0.01 yeast = 1.40 bread + 0.23 vapor |
| Cook meal | Galley | 30 min | 0.6 kWh | 0.25 bread + 0.20 vegetables + 0.15 fish + 0.05 water = 0.55 meal + 0.10 compost |
| Desalinate | Desalination | 30 min | 1.575 kWh | 1000 L seawater = 450 L fresh + 550 L brine (3.5 kWh per m³ fresh) |
| Spin cloth | Sewing | 2 h | 0.4 kWh | 1.00 fibre = 0.92 cloth + 0.08 scrap |
| Sew garment | Sewing | 4 h | 0.2 kWh | 1.20 cloth = 1.05 garment + 0.15 scrap |
| Saw planks | Carpentry | 1 h | 0.3 kWh | 2.00 timber = 1.70 planks + 0.30 compost |
| Repair tools | Carpentry | 2 h | 0.4 kWh | 0.30 planks + 0.10 metal = 0.40 tools |
| Recycle metal | Recycling | 3 h | 1.5 kWh | 1.00 scrap = 0.90 metal + 0.10 process loss |
| Forge fitting | Metalwork | 2 h | 0.8 kWh | 0.50 metal = 0.45 tools + 0.05 scrap |
| Compost nutrients | Waste treatment | 2 h | 0.1 kWh | 1.00 compost = 0.55 nutrients + 0.45 process loss |
| Wash clothes | Laundry | 1 h | 0.3 kWh | 8 L water + 1 garment = 1 garment + 8 L wastewater |

Hydroponics is not a job. Each crop room advances `1/1440` of a day per minute, times the power fraction, and only while a hydroponics crew member is in the room on shift. The cycle is 28 days. Harvest is 0.12 kg of vegetables per m², plus compost equal to 25% of that yield, and it consumes 0.01 kg of nutrients per m². If nutrients are short, the crop waits at day 28. Light, air, and irrigation water are inflows, so crop mass is not a closed balance.

Fishing adds 8 kg per on-shift fisher per day into the cold store, and only while that person is in the fishing room. That catch is a renewable inflow.

Once a day, at minute 0, 0.2% of stored garments become scrap.

The opening endowment is meant to last weeks: 2000 kg flour, 2000 kg vegetables, 600 kg fish, 120 kg bread, 80 kg meals, 30 000 L water, 2500 kg timber, 900 kg metal, plus smaller stocks of salt, yeast, fibre, cloth, garments, planks, tools, scrap, medical supplies, nutrients, and compost. Timber and metal are finite. Food and water can be renewed by the galley, hydroponics, fishing, and desalination. Food days are edible kilograms divided by crew × 0.6. Water days are litres divided by crew × 3.

## Power

The solar plant is a constant 90 kW, so each minute has 1.5 kWh available. `allocate_power` serves lower `PowerClass` values first:

1. Clinic (plus a 0.03 kWh/min base load)
2. Galley
3. Bakery
4. Essential workshops
5. Hydroponics (0.0008 kWh per m² per minute, about 48 W/m²)
6. Desalination

A job asks for `energy_kwh * 60 / duration` only when it is already running, or when it is staffed and its inputs are in stock. Progress equals 60 seconds times the fraction of requested power it received. Hydroponic lights and desalination therefore drop before the clinic and the galley.

## Authority

```rust
pub enum Role {
    Viewer,
}

pub enum Command {
    SetStationRecipe { station: u32, recipe: u16 },
}

pub fn authorize(role: Role, command: &Command) -> Result<(), ViewOnly> {
    match (role, command) {
        (Role::Viewer, Command::SetStationRecipe { .. }) => Err(ViewOnly),
    }
}
```

The WebSocket accepts a `command` message and always answers `command_rejected` for the current role. Stations already store a recipe id. The viewer registers an `Interact` action in leafwing and leaves it unbound, so a later participant role can enable it without a new input stack.

There is no participant role yet. `authorize` has nowhere to return `Ok(())`.

## Persistence

`server/migrations` creates `world_meta`, `decks`, `rooms`, `doors`, `stations`, `crew`, `stocks`, `jobs`, and `crops`. Older `players`, `entities`, and `game_state` tables are left in place and unused.

The server applies migrations on startup when the database user can create tables. When `_sqlx_migrations` already matches this binary, startup skips DDL. It then loads the world or seeds it, catches up, and saves. It writes again at each minute boundary and on shutdown. Polygons are JSON. Enums are serde snake_case strings.

## Messages

`/ws` and `GET /api/world` use the same JSON shapes (`type` tag, snake_case):

- `snapshot` — rooms and polygons, plus crew, stocks, totals, power, and the clock. Sent once on connect.
- `delta` — clock, crew, stocks, totals, and power. Sent when a minute step commits.
- `unavailable` — persistence is missing or failed. No world is attached.
- `command_rejected` — the answer to a client `command`.

The viewer keeps the snapshot and patches it with deltas. Crew glide between the positions in successive deltas. Recipes do not run on the client.
