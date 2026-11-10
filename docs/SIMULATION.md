# Ship simulation

The world is a `sim::World`. The server loads it from PostgreSQL, advances it, and saves it. The Bevy app only renders snapshots.

Time is 1:1 with the wall clock. `World::step` covers 60 seconds. On boot the server calls `catch_up_to` for every missed minute since `last_tick_unix`. The step is deterministic: fixed crew order, no unseeded random numbers, so a day of catch-up matches a day of live steps.

## Hull and decks

The plan follows Icon of the Seas' published size and public deck plans: 365 m overall, 213 ft (65 m) beam. Origin is the aft-starboard corner, outside the hull. +X points forward, +Y points to port. The waterline is a fine bow, a parallel mid-body, and a flat transom with rounded quarters. Decks are 3 m apart. Deck index 0 is the lowest. There is no deck 13, matching the ship, so index 13 is Icon deck 14 and index 19 is Icon deck 20.

Rooms are named rectangles that sit fully on that deck's hull. A shared edge longer than a metre is a door. A corridor is a room. Crew walk the door graph at 1.3 m/s (78 m per step), aiming at the doorway and then into the next room. A stair, which has no shared edge, still aims at the room centre. A berth holds a bed. A head holds a toilet and a shower. Press P in the viewer for the top-down plan.

| Index | Icon deck | What the plan contributes |
| --- | --- | --- |
| 0 | (below the guest plans) | Gas generator hall, switchgear, gas plant space |
| 1 | (below the guest plans) | Desalination, wastewater and incinerator, water, recycling, carpentry, metal shop, sewing, laundry, crew mess |
| 2 | 2 Gangway | Gangway alleys, cold and dry stores, timber and metal stores |
| 3 | 3 | Main dining aft, galley, bakery, medical, Music Hall forward, ocean-view cabins |
| 4 | 4 | Main dining aft, Casino Royale amidships, Royal Theater forward, cabins |
| 5 | 5 | Royal Promenade, no cabins, Sorrento's, aft dining |
| 6 | 6 | Royal Promenade, Giovanni's, Adventure Ocean aft, no cabins |
| 7 | 7 | Surfside Eatery and Water's Edge aft, cabins forward |
| 8 | 8 | Central Park (hydroponics beds and paths), Chops Grille, Izumi, Park Cafe, cabins |
| 9–11 | 9–11 | Cabin decks, open over Central Park |
| 12 | 12 | Cabins, Vitality Spa, bridge forward |
| 13 | 14 | Cabins, Vitality Spa, The Overlook forward. The park is still open |
| 14 | 15 | The Hideaway aft, Windjammer midship, AquaDome Market forward. Solid plate |
| 15 | 16 | Category 6, Coastal Kitchen, suites forward |
| 16 | 17 | Cloud 17, sports court, Coastal Kitchen, suites |
| 17–18 | 18–19 | The Grove forward, plus an aft slide-tower island |
| 19 | 20 | Category 6 slide tops, aft only |

Aft and forward atria are the stairs. Side alleys run the stateroom bands on decks 0–16.

The public plans and the sim disagree where a rectangle, one room kind, or an adjacent-deck stair cannot say what the ship does:

- Decks 0 and 1 are not on the guest plans. The gas-generator hall, switchgear, and tank space sit low, in the machinery band. The viewer draws the sets, cabinets, and tank shells; there is no fuel-flow model. Wastewater tanks and an incinerator share the waste room on deck 1. Carpentry, the metal shop, sewing, and laundry are the deck 1 workshops, also below the guest plans.
- Lifeboats hang outboard of Icon deck 7 (the Surfside topsides), eight a side. They are scenery, not rooms, because a room has to sit inside the hull.
- Icon has 2,805 staterooms. The sim samples that band as 4.4 m × 7.2 m modules (real balcony cabins are nearer 3 m; the module is wide enough for a 2 m bed). A 1.2 m gap between modules is a bulkhead, not a door. An en-suite bath is its own Head room, because a room has one kind.
- Casino Royale, the Royal Theater, Music Hall, Absolute Zero, the ice rink, the spa, and the sports court have no room kinds. They keep the real footprint as a named corridor, restaurant, bakery, mess, hydroponics plot, or the bridge. Absolute Zero is not separate from the aft dining room. Fishing is required by the crew rules and is a small starboard room on deck 7; Icon has no fishing venue.
- Pools and the waterpark are scenery on the rooms that already occupy those footprints. Hideaway Pool is aft on Icon deck 15. Water's Edge is the family pool on Icon deck 7. Royal Bay is the starboard side of the Windjammer footprint on Icon deck 15, because that rectangle is also the restaurant. Cloud 17's pool is on Icon deck 17. Category 6 (Icon decks 16 and 20, plus the slide-tower landings) has a splash basin and three flumes. The Cove, the whirlpools, FlowRider, and the mini golf are not separate rooms.
- Central Park on Icon deck 8 is planted with trees, which is where the public plan puts the garden. Icon decks 18 and 19 are the Grove suite sun deck. The starboard half of those rooms is also planted, so half of that top sun deck reads as a park. On the real ship those decks are suite loungers, and deck 20 is only the slide tops, which is too small to hold the park.
- Central Park is a hole on indices 9–13 (Icon decks 9–12 and 14). Deck 8 is the park floor. Index 14 (Icon 15) is a solid plate over that hole. The real park is still open air beside the pools; the sim cannot leave a deck open to the sky and still walk on it.
- Indices 17 and 18 (Icon 18 and 19) add a disconnected aft island so the deck 20 slides can meet a stair on the deck below. Stairs only join adjacent decks that share a footprint. That island is not guest deck.
- Crown's Edge, the running track, escalators, and curved rooms are not in the model. The perimeter passages on decks 5 and 6 stand in for the promenade edge.

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
