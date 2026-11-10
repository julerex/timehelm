//! Icon of the Seas deck arrangement, as far as rectangular rooms allow.
//!
//! Public plans (Royal Caribbean's Icon deck plan, CruiseMapper, and the
//! deck-by-deck guides) place the neighborhoods like this. X is aft→fore.
//!
//! Decks 0–1 are below the guest plans: the gas generator hall, then workshops and waste.
//! Deck 2 is the gangway. Decks 3–4 are dining aft with cabins outboard.
//! Decks 5–6 are the Royal Promenade and have no staterooms. Deck 7 is
//! Surfside aft. Deck 8 is Central Park. Decks 9–12 and 14 are cabin decks
//! open over the park; the bridge and spa are forward. There is no deck 13.
//! Deck 15 is AquaDome forward, Chill Island midship, Hideaway aft. Deck 16
//! is Thrill Island with suites forward. Deck 17 is Cloud 17. Decks 18–19
//! are the Grove. Deck 20 is the slide tops.
//!
//! A room is one rectangle of one kind, and a shared edge is a door. That is
//! why private bathrooms are their own rooms, why a casino is a named
//! corridor, and why the 2,805 staterooms are a 4.4 m module along the real
//! cabin bands. Fishing has no Icon venue; the sim still needs one station.

use crate::geom::{rect_polygon, shared_edge_span};
use crate::hull::rect_on_deck;
use crate::kinds::RoomKind;
use crate::recipes::RecipeId;
use crate::world::{Deck, Door, Room, Station};

pub const DECK_COUNT: usize = 20;
pub const DECK_HEIGHT_M: f32 = 3.0;

pub const DECK_NAMES: [&str; DECK_COUNT] = [
    "Machinery",
    "Utility",
    "Deck 2 Gangway",
    "Deck 3 Dining",
    "Deck 4 Casino",
    "Deck 5 Promenade",
    "Deck 6 Promenade",
    "Deck 7 Surfside",
    "Deck 8 Central Park",
    "Deck 9 Cabins",
    "Deck 10 Cabins",
    "Deck 11 Cabins",
    "Deck 12 Bridge",
    "Deck 14 Spa",
    "Deck 15 Chill Island",
    "Deck 16 Thrill Island",
    "Deck 17 Cloud 17",
    "Deck 18 Grove",
    "Deck 19 Grove",
    "Deck 20 Slides",
];

const STBD_CABIN_Y0: f32 = 1.8;
const STBD_CABIN_Y1: f32 = 9.0;
const STBD_HALL_Y0: f32 = 9.0;
const STBD_HALL_Y1: f32 = 12.4;
const IN_STBD_Y0: f32 = 12.4;
const IN_STBD_Y1: f32 = 19.2;
const SPINE_Y0: f32 = 26.0;
const SPINE_Y1: f32 = 36.0;
const IN_PORT_Y0: f32 = 45.8;
const IN_PORT_Y1: f32 = 52.6;
const PORT_HALL_Y0: f32 = 52.6;
const PORT_HALL_Y1: f32 = 56.0;
const PORT_CABIN_Y0: f32 = 56.0;
const PORT_CABIN_Y1: f32 = 63.2;

const AFT_X0: f32 = 72.0;
const AFT_X1: f32 = 82.0;
const FWD_X0: f32 = 258.0;
const FWD_X1: f32 = 268.0;
const HALL_X0: f32 = 36.0;
const HALL_X1: f32 = 292.0;

const CABIN_W: f32 = 4.4;
const CABIN_STEP: f32 = 5.6;

#[derive(Clone, Copy)]
struct Draft {
    deck: u8,
    kind: RoomKind,
    name: &'static str,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

pub struct BuiltShip {
    pub decks: Vec<Deck>,
    pub rooms: Vec<Room>,
    pub doors: Vec<Door>,
    pub stations: Vec<Station>,
}

pub fn build_ship() -> Result<BuiltShip, String> {
    let mut drafts = Vec::new();
    for deck in 0..DECK_COUNT {
        build_deck(&mut drafts, deck as u8)?;
    }

    let rooms = drafts
        .iter()
        .enumerate()
        .map(|(id, draft)| Room {
            id: id as u32,
            deck: draft.deck,
            kind: draft.kind,
            name: room_name(draft, id),
            polygon: rect_polygon(draft.x0, draft.y0, draft.x1, draft.y1),
        })
        .collect::<Vec<_>>();
    let doors = doors_for(&drafts);
    let stations = stations_for(&rooms);
    let decks = DECK_NAMES
        .iter()
        .enumerate()
        .map(|(index, name)| Deck {
            index: index as u8,
            name: (*name).to_string(),
        })
        .collect();
    let built = BuiltShip {
        decks,
        rooms,
        doors,
        stations,
    };
    let missing = missing_kinds(&built.rooms);
    if !missing.is_empty() {
        return Err(format!("layout is missing {missing:?}"));
    }
    Ok(built)
}

fn room_name(draft: &Draft, id: usize) -> String {
    if draft.name.is_empty() {
        format!("{} {}-{}", draft.kind.label(), draft.deck, id)
    } else {
        draft.name.to_string()
    }
}

fn missing_kinds(rooms: &[Room]) -> Vec<RoomKind> {
    RoomKind::ALL
        .into_iter()
        .filter(|kind| !rooms.iter().any(|room| room.kind == *kind))
        .collect()
}

fn build_deck(drafts: &mut Vec<Draft>, deck: u8) -> Result<(), String> {
    place_stairs(drafts, deck)?;
    place_alleys(drafts, deck)?;
    if deck <= 16 {
        place_atria(drafts, deck)?;
    }
    place_program(drafts, deck)?;
    if deck <= 16 {
        place_spine(drafts, deck);
    }
    place_cabins(drafts, deck);
    Ok(())
}

fn place_stairs(drafts: &mut Vec<Draft>, deck: u8) -> Result<(), String> {
    must(
        drafts,
        Draft {
            deck,
            kind: RoomKind::Stair,
            name: "Aft atrium",
            x0: AFT_X0,
            y0: SPINE_Y0,
            x1: AFT_X1,
            y1: SPINE_Y1,
        },
    )?;
    if deck < 19 {
        must(
            drafts,
            Draft {
                deck,
                kind: RoomKind::Stair,
                name: "Forward atrium",
                x0: FWD_X0,
                y0: SPINE_Y0,
                x1: FWD_X1,
                y1: SPINE_Y1,
            },
        )?;
    }
    Ok(())
}

fn place_alleys(drafts: &mut Vec<Draft>, deck: u8) -> Result<(), String> {
    if deck >= 17 {
        return Ok(());
    }
    let name = match deck {
        2 => "Gangway",
        5 | 6 => "Perimeter passage",
        0 | 1 => "Service passage",
        _ => "Stateroom alley",
    };
    must(
        drafts,
        Draft {
            deck,
            kind: RoomKind::Corridor,
            name,
            x0: HALL_X0,
            y0: STBD_HALL_Y0,
            x1: HALL_X1,
            y1: STBD_HALL_Y1,
        },
    )?;
    must(
        drafts,
        Draft {
            deck,
            kind: RoomKind::Corridor,
            name,
            x0: HALL_X0,
            y0: PORT_HALL_Y0,
            x1: HALL_X1,
            y1: PORT_HALL_Y1,
        },
    )?;
    Ok(())
}

fn place_atria(drafts: &mut Vec<Draft>, deck: u8) -> Result<(), String> {
    for (name, x0, x1) in [
        ("Aft atrium", AFT_X0, AFT_X1),
        ("Forward atrium", FWD_X0, FWD_X1),
    ] {
        must(
            drafts,
            Draft {
                deck,
                kind: RoomKind::Corridor,
                name,
                x0,
                y0: STBD_HALL_Y1,
                x1,
                y1: SPINE_Y0,
            },
        )?;
        must(
            drafts,
            Draft {
                deck,
                kind: RoomKind::Corridor,
                name,
                x0,
                y0: SPINE_Y1,
                x1,
                y1: PORT_HALL_Y0,
            },
        )?;
    }
    Ok(())
}

fn place_spine(drafts: &mut Vec<Draft>, deck: u8) {
    let name = match deck {
        2 => "Gangway",
        7 => "Surfside",
        14 => "Chill Island",
        15 => "Thrill Island",
        _ => "Corridor",
    };
    for (x0, x1) in [
        (HALL_X0, AFT_X0),
        (AFT_X1, 118.0),
        (130.0, FWD_X0),
        (242.0, FWD_X0),
        (AFT_X1, FWD_X0),
        (FWD_X1, HALL_X1),
    ] {
        if x1 - x0 < 8.0 {
            continue;
        }
        let _ = try_add(
            drafts,
            Draft {
                deck,
                kind: RoomKind::Corridor,
                name,
                x0,
                y0: SPINE_Y0,
                x1,
                y1: SPINE_Y1,
            },
        );
    }
}

fn place_program(drafts: &mut Vec<Draft>, deck: u8) -> Result<(), String> {
    match deck {
        0 => machinery(drafts),
        1 => utility(drafts),
        2 => stores(drafts),
        3 => dining_deck(drafts, true),
        4 => dining_deck(drafts, false),
        5 => promenade(drafts, false),
        6 => promenade(drafts, true),
        7 => surfside(drafts),
        8 => central_park(drafts),
        12 => bridge_deck(drafts),
        13 => spa_deck(drafts),
        14 => pool_deck(drafts),
        15 => thrill_deck(drafts),
        16 => cloud_deck(drafts),
        17 | 18 => grove(drafts, deck),
        19 => slides(drafts),
        _ => Ok(()),
    }
}

fn machinery(drafts: &mut Vec<Draft>) -> Result<(), String> {
    must(
        drafts,
        room(
            0,
            RoomKind::Engineering,
            "Gas generator hall",
            90.0,
            12.4,
            160.0,
            26.0,
        ),
    )?;
    must(
        drafts,
        room(
            0,
            RoomKind::Electrical,
            "Switchgear",
            168.0,
            12.4,
            230.0,
            26.0,
        ),
    )?;
    must(
        drafts,
        room(
            0,
            RoomKind::FuelStore,
            "Gas plant space",
            90.0,
            36.0,
            155.0,
            52.6,
        ),
    )
}

fn utility(drafts: &mut Vec<Draft>) -> Result<(), String> {
    must(
        drafts,
        room(
            1,
            RoomKind::Desalination,
            "Desalination",
            90.0,
            12.4,
            145.0,
            26.0,
        ),
    )?;
    must(
        drafts,
        room(
            1,
            RoomKind::WasteTreatment,
            "Wastewater and incinerator",
            150.0,
            12.4,
            205.0,
            26.0,
        ),
    )?;
    must(
        drafts,
        room(
            1,
            RoomKind::WaterStore,
            "Water store",
            210.0,
            12.4,
            252.0,
            26.0,
        ),
    )?;
    must(
        drafts,
        room(1, RoomKind::Recycling, "Recycling", 90.0, 36.0, 140.0, 52.6),
    )?;
    must(
        drafts,
        room(
            1,
            RoomKind::Carpentry,
            "Carpentry",
            145.0,
            36.0,
            190.0,
            52.6,
        ),
    )?;
    must(
        drafts,
        room(
            1,
            RoomKind::Metalwork,
            "Metalwork",
            195.0,
            36.0,
            245.0,
            52.6,
        ),
    )?;
    must(
        drafts,
        room(
            1,
            RoomKind::Sewing,
            "Sewing workshop",
            90.0,
            1.8,
            140.0,
            9.0,
        ),
    )?;
    must(
        drafts,
        room(1, RoomKind::Laundry, "Laundry", 148.0, 1.8, 198.0, 9.0),
    )?;
    must(
        drafts,
        room(1, RoomKind::Mess, "Crew mess", 206.0, 1.8, 252.0, 9.0),
    )
}

fn stores(drafts: &mut Vec<Draft>) -> Result<(), String> {
    must(
        drafts,
        room(
            2,
            RoomKind::ColdStore,
            "Cold store",
            90.0,
            12.4,
            145.0,
            26.0,
        ),
    )?;
    must(
        drafts,
        room(2, RoomKind::DryStore, "Dry store", 150.0, 12.4, 205.0, 26.0),
    )?;
    must(
        drafts,
        room(
            2,
            RoomKind::TimberStore,
            "Timber store",
            210.0,
            12.4,
            252.0,
            26.0,
        ),
    )?;
    must(
        drafts,
        room(
            2,
            RoomKind::MetalStore,
            "Metal store",
            90.0,
            36.0,
            150.0,
            52.6,
        ),
    )
}

fn dining_deck(drafts: &mut Vec<Draft>, lower: bool) -> Result<(), String> {
    let deck = if lower { 3 } else { 4 };
    must(
        drafts,
        room(
            deck,
            RoomKind::Restaurant,
            "Main Dining Room",
            36.0,
            12.4,
            AFT_X0,
            52.6,
        ),
    )?;
    if lower {
        must(
            drafts,
            room(deck, RoomKind::Galley, "Galley", 90.0, 12.4, 148.0, 26.0),
        )?;
        must(
            drafts,
            room(deck, RoomKind::Bakery, "Bakery", 90.0, 36.0, 148.0, 52.6),
        )?;
        must(
            drafts,
            room(
                deck,
                RoomKind::Clinic,
                "Medical Center",
                160.0,
                12.4,
                214.0,
                26.0,
            ),
        )?;
        must(
            drafts,
            room(
                deck,
                RoomKind::Infirmary,
                "Infirmary",
                160.0,
                36.0,
                214.0,
                52.6,
            ),
        )?;
        forward_block(drafts, deck, RoomKind::Corridor, "Music Hall")?;
    } else {
        wide_public(drafts, deck, "Casino Royale", AFT_X1, FWD_X0)?;
        forward_block(drafts, deck, RoomKind::Corridor, "Royal Theater")?;
    }
    Ok(())
}

fn promenade(drafts: &mut Vec<Draft>, upper: bool) -> Result<(), String> {
    let deck = if upper { 6 } else { 5 };
    wide_public(drafts, deck, "Royal Promenade", AFT_X1, FWD_X0)?;
    if upper {
        must(
            drafts,
            room(
                deck,
                RoomKind::Restaurant,
                "Giovanni's",
                200.0,
                47.0,
                248.0,
                PORT_HALL_Y0,
            ),
        )?;
        must(
            drafts,
            room(
                deck,
                RoomKind::Corridor,
                "Adventure Ocean",
                36.0,
                16.0,
                AFT_X0,
                50.0,
            ),
        )?;
    } else {
        must(
            drafts,
            room(
                deck,
                RoomKind::Restaurant,
                "Main Dining Room",
                36.0,
                16.0,
                AFT_X0,
                50.0,
            ),
        )?;
        must(
            drafts,
            room(
                deck,
                RoomKind::Bakery,
                "Sorrento's",
                200.0,
                STBD_HALL_Y1,
                244.0,
                18.0,
            ),
        )?;
    }
    Ok(())
}

fn surfside(drafts: &mut Vec<Draft>) -> Result<(), String> {
    must(
        drafts,
        room(
            7,
            RoomKind::Mess,
            "Surfside Eatery",
            36.0,
            14.0,
            AFT_X0,
            52.0,
        ),
    )?;
    must(
        drafts,
        room(
            7,
            RoomKind::Corridor,
            "Water's Edge",
            AFT_X1,
            14.0,
            130.0,
            52.0,
        ),
    )?;
    must(
        drafts,
        room(
            7,
            RoomKind::Fishing,
            "Fishing station",
            36.0,
            STBD_CABIN_Y0,
            56.0,
            STBD_CABIN_Y1,
        ),
    )
}

fn central_park(drafts: &mut Vec<Draft>) -> Result<(), String> {
    must(
        drafts,
        room(
            8,
            RoomKind::Hydroponics,
            "Central Park",
            122.0,
            21.0,
            176.0,
            31.5,
        ),
    )?;
    must(
        drafts,
        room(
            8,
            RoomKind::Hydroponics,
            "Central Park",
            122.0,
            35.5,
            176.0,
            44.0,
        ),
    )?;
    must(
        drafts,
        room(
            8,
            RoomKind::Hydroponics,
            "Central Park",
            184.0,
            21.0,
            238.0,
            31.5,
        ),
    )?;
    must(
        drafts,
        room(
            8,
            RoomKind::Hydroponics,
            "Central Park",
            184.0,
            35.5,
            238.0,
            44.0,
        ),
    )?;
    must(
        drafts,
        room(8, RoomKind::Corridor, "Park path", 176.0, 21.0, 184.0, 44.0),
    )?;
    must(
        drafts,
        room(8, RoomKind::Corridor, "Park path", 118.0, 31.5, 176.0, 35.5),
    )?;
    must(
        drafts,
        room(8, RoomKind::Corridor, "Park path", 184.0, 31.5, 242.0, 35.5),
    )?;
    must(
        drafts,
        room(
            8,
            RoomKind::Restaurant,
            "Chops Grille",
            148.0,
            STBD_HALL_Y1,
            198.0,
            21.0,
        ),
    )?;
    must(
        drafts,
        room(
            8,
            RoomKind::Restaurant,
            "Izumi",
            150.0,
            44.0,
            200.0,
            PORT_HALL_Y0,
        ),
    )?;
    must(
        drafts,
        room(
            8,
            RoomKind::Bakery,
            "Park Cafe",
            206.0,
            44.0,
            242.0,
            PORT_HALL_Y0,
        ),
    )
}

fn bridge_deck(drafts: &mut Vec<Draft>) -> Result<(), String> {
    forward_block(drafts, 12, RoomKind::Bridge, "Bridge")?;
    must(
        drafts,
        room(
            12,
            RoomKind::Corridor,
            "Vitality Spa",
            188.0,
            IN_STBD_Y0,
            242.0,
            IN_STBD_Y1,
        ),
    )
}

fn spa_deck(drafts: &mut Vec<Draft>) -> Result<(), String> {
    forward_block(drafts, 13, RoomKind::Corridor, "The Overlook")?;
    must(
        drafts,
        room(
            13,
            RoomKind::Corridor,
            "Vitality Spa",
            188.0,
            IN_STBD_Y0,
            242.0,
            IN_STBD_Y1,
        ),
    )
}

fn pool_deck(drafts: &mut Vec<Draft>) -> Result<(), String> {
    must(
        drafts,
        room(
            14,
            RoomKind::Corridor,
            "The Hideaway",
            36.0,
            STBD_HALL_Y1,
            AFT_X0,
            PORT_HALL_Y0,
        ),
    )?;
    must(
        drafts,
        room(
            14,
            RoomKind::Restaurant,
            "Windjammer Marketplace",
            130.0,
            STBD_HALL_Y1,
            214.0,
            PORT_HALL_Y0,
        ),
    )?;
    forward_block(drafts, 14, RoomKind::Restaurant, "AquaDome Market")
}

fn thrill_deck(drafts: &mut Vec<Draft>) -> Result<(), String> {
    must(
        drafts,
        room(
            15,
            RoomKind::Restaurant,
            "Coastal Kitchen",
            188.0,
            36.0,
            250.0,
            PORT_HALL_Y0,
        ),
    )?;
    must(
        drafts,
        room(
            15,
            RoomKind::Corridor,
            "Category 6",
            90.0,
            STBD_HALL_Y1,
            176.0,
            SPINE_Y0,
        ),
    )
}

fn cloud_deck(drafts: &mut Vec<Draft>) -> Result<(), String> {
    must(
        drafts,
        room(
            16,
            RoomKind::Corridor,
            "Cloud 17",
            140.0,
            STBD_HALL_Y1,
            186.0,
            SPINE_Y0,
        ),
    )?;
    must(
        drafts,
        room(
            16,
            RoomKind::Restaurant,
            "Coastal Kitchen",
            186.0,
            STBD_HALL_Y1,
            248.0,
            24.0,
        ),
    )?;
    must(
        drafts,
        room(
            16,
            RoomKind::Corridor,
            "Sports Court",
            140.0,
            SPINE_Y1,
            230.0,
            PORT_HALL_Y0,
        ),
    )
}

fn grove(drafts: &mut Vec<Draft>, deck: u8) -> Result<(), String> {
    tower_landings(drafts, deck, "Slide tower")?;
    must(
        drafts,
        room(
            deck,
            RoomKind::Corridor,
            "The Grove",
            214.0,
            20.0,
            FWD_X0,
            46.0,
        ),
    )?;
    must(
        drafts,
        room(
            deck,
            RoomKind::Corridor,
            "The Grove",
            FWD_X1,
            22.0,
            308.0,
            44.0,
        ),
    )?;
    if deck == 17 {
        must(
            drafts,
            room(deck, RoomKind::Berth, "Suite", 220.0, 12.0, 232.0, 20.0),
        )?;
        must(
            drafts,
            room(deck, RoomKind::Berth, "Suite", 220.0, 46.0, 232.0, 54.0),
        )?;
    }
    Ok(())
}

fn slides(drafts: &mut Vec<Draft>) -> Result<(), String> {
    tower_landings(drafts, 19, "Category 6")?;
    must(
        drafts,
        room(19, RoomKind::Corridor, "Category 6", 32.0, 20.0, 64.0, 46.0),
    )?;
    must(
        drafts,
        room(
            19,
            RoomKind::Corridor,
            "Category 6",
            102.0,
            20.0,
            118.0,
            46.0,
        ),
    )
}

fn tower_landings(drafts: &mut Vec<Draft>, deck: u8, name: &'static str) -> Result<(), String> {
    must(
        drafts,
        room(deck, RoomKind::Corridor, name, 64.0, 22.0, AFT_X0, 46.0),
    )?;
    must(
        drafts,
        room(deck, RoomKind::Corridor, name, AFT_X1, 22.0, 102.0, 46.0),
    )
}

fn wide_public(
    drafts: &mut Vec<Draft>,
    deck: u8,
    name: &'static str,
    x0: f32,
    x1: f32,
) -> Result<(), String> {
    must(
        drafts,
        room(deck, RoomKind::Corridor, name, x0, 18.0, x1, 47.0),
    )
}

fn forward_block(
    drafts: &mut Vec<Draft>,
    deck: u8,
    kind: RoomKind,
    name: &'static str,
) -> Result<(), String> {
    must(
        drafts,
        room(
            deck,
            RoomKind::Corridor,
            "Forward atrium",
            FWD_X1,
            SPINE_Y0,
            274.0,
            SPINE_Y1,
        ),
    )?;
    must(drafts, room(deck, kind, name, 274.0, 24.0, 322.0, 42.0))
}

fn place_cabins(drafts: &mut Vec<Draft>, deck: u8) {
    match deck {
        3 | 4 | 8 | 9 | 10 | 11 | 12 | 13 => {
            place_heads(drafts, deck);
            berth_row(drafts, deck, 40.0, 290.0, STBD_CABIN_Y0, STBD_CABIN_Y1);
            berth_row(drafts, deck, 40.0, 290.0, PORT_CABIN_Y0, PORT_CABIN_Y1);
        }
        7 => {
            place_heads(drafts, deck);
            berth_row(drafts, deck, 140.0, 290.0, STBD_CABIN_Y0, STBD_CABIN_Y1);
            berth_row(drafts, deck, 140.0, 290.0, PORT_CABIN_Y0, PORT_CABIN_Y1);
        }
        _ => {}
    }
    if matches!(deck, 8..=13) {
        berth_row(drafts, deck, 40.0, 290.0, IN_STBD_Y0, IN_STBD_Y1);
        berth_row(drafts, deck, 40.0, 290.0, IN_PORT_Y0, IN_PORT_Y1);
    }
    if matches!(deck, 15 | 16) {
        for x0 in [276.0, 290.0] {
            let _ = try_add(
                drafts,
                room(
                    deck,
                    RoomKind::Berth,
                    "Suite",
                    x0,
                    IN_STBD_Y0,
                    x0 + 8.0,
                    IN_STBD_Y1,
                ),
            );
            let _ = try_add(
                drafts,
                room(
                    deck,
                    RoomKind::Berth,
                    "Suite",
                    x0,
                    IN_PORT_Y0,
                    x0 + 8.0,
                    IN_PORT_Y1,
                ),
            );
        }
        let _ = try_add(
            drafts,
            room(
                deck,
                RoomKind::Head,
                "Head",
                276.0,
                STBD_CABIN_Y0,
                282.0,
                STBD_CABIN_Y1,
            ),
        );
    }
}

fn place_heads(drafts: &mut Vec<Draft>, deck: u8) {
    // On the cabin-module grid, so a bath replaces one berth instead of
    // overlapping two and leaving a bare patch of hull.
    for x0 in [40.0 + CABIN_STEP, 40.0 + CABIN_STEP * 34.0] {
        let _ = try_add(
            drafts,
            room(
                deck,
                RoomKind::Head,
                "Head",
                x0,
                STBD_CABIN_Y0,
                x0 + CABIN_W,
                STBD_CABIN_Y1,
            ),
        );
        let _ = try_add(
            drafts,
            room(
                deck,
                RoomKind::Head,
                "Head",
                x0,
                PORT_CABIN_Y0,
                x0 + CABIN_W,
                PORT_CABIN_Y1,
            ),
        );
    }
}

fn berth_row(drafts: &mut Vec<Draft>, deck: u8, x0: f32, x1: f32, y0: f32, y1: f32) {
    let mut x = x0;
    while x + CABIN_W <= x1 {
        let _ = try_add(
            drafts,
            room(deck, RoomKind::Berth, "", x, y0, x + CABIN_W, y1),
        );
        x += CABIN_STEP;
    }
}

fn room(deck: u8, kind: RoomKind, name: &'static str, x0: f32, y0: f32, x1: f32, y1: f32) -> Draft {
    Draft {
        deck,
        kind,
        name,
        x0,
        y0,
        x1,
        y1,
    }
}

fn try_add(drafts: &mut Vec<Draft>, draft: Draft) -> bool {
    if draft.x1 - draft.x0 < 2.0 || draft.y1 - draft.y0 < 2.0 {
        return false;
    }
    if !rect_on_deck(draft.deck as usize, draft.x0, draft.y0, draft.x1, draft.y1) {
        return false;
    }
    if overlapping(drafts, &draft).is_some() {
        return false;
    }
    drafts.push(draft);
    true
}

fn must(drafts: &mut Vec<Draft>, draft: Draft) -> Result<(), String> {
    if draft.x1 - draft.x0 < 2.0 || draft.y1 - draft.y0 < 2.0 {
        return Err(format!(
            "{} on deck {} is too small",
            draft.name, draft.deck
        ));
    }
    if !rect_on_deck(draft.deck as usize, draft.x0, draft.y0, draft.x1, draft.y1) {
        return Err(format!(
            "{} on deck {} is outside the hull ({:.0},{:.0})-({:.0},{:.0})",
            draft.name, draft.deck, draft.x0, draft.y0, draft.x1, draft.y1
        ));
    }
    if let Some(other) = overlapping(drafts, &draft) {
        return Err(format!(
            "{} on deck {} overlaps {} ({:.0},{:.0})-({:.0},{:.0})",
            draft.name, draft.deck, other.name, other.x0, other.y0, other.x1, other.y1
        ));
    }
    drafts.push(draft);
    Ok(())
}

fn overlapping<'a>(drafts: &'a [Draft], draft: &Draft) -> Option<&'a Draft> {
    drafts.iter().find(|other| {
        other.deck == draft.deck
            && draft.x0 < other.x1 - 0.05
            && draft.x1 > other.x0 + 0.05
            && draft.y0 < other.y1 - 0.05
            && draft.y1 > other.y0 + 0.05
    })
}

fn doors_for(drafts: &[Draft]) -> Vec<Door> {
    let mut doors = Vec::new();
    let mut id = 0u32;
    let mut on_deck: Vec<Vec<usize>> = vec![Vec::new(); DECK_COUNT];
    for (index, draft) in drafts.iter().enumerate() {
        on_deck[draft.deck as usize].push(index);
    }
    for indices in &on_deck {
        for (a, left) in indices.iter().enumerate() {
            for right in indices.iter().skip(a + 1) {
                if shares_edge(&drafts[*left], &drafts[*right]) {
                    doors.push(Door {
                        id,
                        from_room: *left as u32,
                        to_room: *right as u32,
                    });
                    id += 1;
                }
            }
        }
    }
    let stairs: Vec<usize> = drafts
        .iter()
        .enumerate()
        .filter_map(|(index, draft)| (draft.kind == RoomKind::Stair).then_some(index))
        .collect();
    for (a, left) in stairs.iter().enumerate() {
        for right in stairs.iter().skip(a + 1) {
            let from = &drafts[*left];
            let to = &drafts[*right];
            let adjacent = (from.deck as i16 - to.deck as i16).abs() == 1;
            if adjacent && same_footprint(from, to) {
                doors.push(Door {
                    id,
                    from_room: *left as u32,
                    to_room: *right as u32,
                });
                id += 1;
            }
        }
    }
    doors
}

fn shares_edge(a: &Draft, b: &Draft) -> bool {
    let left = rect_polygon(a.x0, a.y0, a.x1, a.y1);
    let right = rect_polygon(b.x0, b.y0, b.x1, b.y1);
    shared_edge_span(&left, &right).is_some()
}

fn same_footprint(a: &Draft, b: &Draft) -> bool {
    (a.x0 - b.x0).abs() < 0.1
        && (a.y0 - b.y0).abs() < 0.1
        && (a.x1 - b.x1).abs() < 0.1
        && (a.y1 - b.y1).abs() < 0.1
}

fn stations_for(rooms: &[Room]) -> Vec<Station> {
    let mut counts: Vec<(RoomKind, usize)> = Vec::new();
    let mut stations = Vec::new();
    for room in rooms {
        if !wants_station(room.kind) {
            continue;
        }
        let n = counts
            .iter()
            .find(|(kind, _)| *kind == room.kind)
            .map(|(_, n)| *n)
            .unwrap_or(0);
        counts.retain(|(kind, _)| *kind != room.kind);
        counts.push((room.kind, n + 1));
        stations.push(Station {
            id: stations.len() as u32,
            room_id: room.id,
            recipe: default_recipe(room.kind, n),
        });
    }
    stations
}

fn wants_station(kind: RoomKind) -> bool {
    matches!(
        kind,
        RoomKind::Engineering
            | RoomKind::Electrical
            | RoomKind::Desalination
            | RoomKind::WasteTreatment
            | RoomKind::Recycling
            | RoomKind::Hydroponics
            | RoomKind::Galley
            | RoomKind::Bakery
            | RoomKind::Restaurant
            | RoomKind::Mess
            | RoomKind::Clinic
            | RoomKind::Infirmary
            | RoomKind::Carpentry
            | RoomKind::Sewing
            | RoomKind::Metalwork
            | RoomKind::Laundry
            | RoomKind::Fishing
            | RoomKind::Bridge
    )
}

fn default_recipe(kind: RoomKind, index: usize) -> Option<RecipeId> {
    match kind {
        RoomKind::Bakery => Some(RecipeId::BakeBread),
        RoomKind::Galley => Some(RecipeId::CookMeal),
        RoomKind::Desalination => Some(RecipeId::Desalinate),
        RoomKind::Sewing if index.is_multiple_of(2) => Some(RecipeId::SpinCloth),
        RoomKind::Sewing => Some(RecipeId::SewGarment),
        RoomKind::Carpentry if index.is_multiple_of(2) => Some(RecipeId::SawPlanks),
        RoomKind::Carpentry => Some(RecipeId::RepairTools),
        RoomKind::Metalwork => Some(RecipeId::ForgeFitting),
        RoomKind::Recycling => Some(RecipeId::RecycleMetal),
        RoomKind::WasteTreatment => Some(RecipeId::CompostNutrients),
        RoomKind::Laundry => Some(RecipeId::WashClothes),
        RoomKind::Engineering
        | RoomKind::Electrical
        | RoomKind::Hydroponics
        | RoomKind::Fishing
        | RoomKind::Restaurant
        | RoomKind::Mess
        | RoomKind::Clinic
        | RoomKind::Infirmary
        | RoomKind::Bridge
        | RoomKind::Corridor
        | RoomKind::Stair
        | RoomKind::Berth
        | RoomKind::Head
        | RoomKind::ColdStore
        | RoomKind::DryStore
        | RoomKind::WaterStore
        | RoomKind::TimberStore
        | RoomKind::MetalStore
        | RoomKind::FuelStore => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::polygon_centroid;
    use crate::hull::{point_on_deck, PARK_X0, PARK_X1, PARK_Y0, PARK_Y1};

    fn ship() -> BuiltShip {
        build_ship().expect("icon layout")
    }

    fn centroid(room: &Room) -> (f32, f32) {
        polygon_centroid(&room.polygon)
    }

    #[test]
    fn icon_neighborhoods_match_the_public_plans() {
        let built = ship();
        assert_eq!(built.decks.len(), 20);
        assert_eq!(built.decks[13].name, "Deck 14 Spa");
        assert_eq!(built.decks[19].name, "Deck 20 Slides");
        assert!(built.rooms.iter().all(|room| {
            room.polygon
                .iter()
                .all(|point| point_on_deck(room.deck as usize, glam::Vec2::new(point[0], point[1])))
        }));

        let named = |name: &str, deck: u8| -> Option<&Room> {
            built
                .rooms
                .iter()
                .find(|room| room.deck == deck && room.name == name)
        };
        let dining = named("Main Dining Room", 3).expect("aft dining");
        assert!(centroid(dining).0 < 80.0);
        assert!(named("Medical Center", 3).is_some());
        assert!(named("Music Hall", 3).is_some());
        assert!(named("Casino Royale", 4).is_some());
        assert!(named("Royal Theater", 4).is_some());
        assert!(named("Royal Promenade", 5).is_some());
        assert!(named("Royal Promenade", 6).is_some());
        assert!(named("Sorrento's", 5).is_some());
        assert!(named("Giovanni's", 6).is_some());
        assert!(named("Adventure Ocean", 6).is_some());
        let surfside = named("Surfside Eatery", 7).expect("surfside");
        assert!(centroid(surfside).0 < 80.0);
        assert_eq!(
            built
                .rooms
                .iter()
                .filter(|room| room.kind == RoomKind::Fishing)
                .map(|room| room.deck)
                .collect::<Vec<_>>(),
            vec![7]
        );
        let parks: Vec<_> = built
            .rooms
            .iter()
            .filter(|room| room.deck == 8 && room.kind == RoomKind::Hydroponics)
            .collect();
        assert!(parks.len() >= 4);
        for park in parks {
            let (x, y) = centroid(park);
            assert!(x > PARK_X0 && x < PARK_X1 && y > PARK_Y0 && y < PARK_Y1);
        }
        for deck in 9..=13 {
            for room in built.rooms.iter().filter(|room| room.deck == deck) {
                let (x, y) = centroid(room);
                let in_park = x > PARK_X0 && x < PARK_X1 && y > PARK_Y0 && y < PARK_Y1;
                assert!(!in_park, "{} sits in the park opening", room.name);
            }
        }
        let bridge = named("Bridge", 12).expect("bridge");
        assert!(centroid(bridge).0 > 270.0);
        assert!(named("Vitality Spa", 12).is_some());
        assert!(named("The Overlook", 13).is_some());
        let hideaway = named("The Hideaway", 14).expect("hideaway");
        assert!(centroid(hideaway).0 < 80.0);
        let dome = named("AquaDome Market", 14).expect("aquadome");
        assert!(centroid(dome).0 > 270.0);
        assert!(named("Coastal Kitchen", 15).is_some());
        assert!(named("Category 6", 15).is_some());
        assert!(named("Cloud 17", 16).is_some());
        assert!(named("The Grove", 17).is_some());
        assert!(named("Suite", 17).is_some());
        assert!(
            built
                .rooms
                .iter()
                .filter(|room| room.deck == 18 && room.kind == RoomKind::Berth)
                .count()
                == 0
        );
        assert!(named("Category 6", 19).is_some());
        assert!(named("Gas generator hall", 0).is_some());
        assert!(named("Switchgear", 0).is_some());
        assert!(named("Gas plant space", 0).is_some());
        assert!(named("Wastewater and incinerator", 1).is_some());
        for name in ["Carpentry", "Metalwork", "Sewing workshop", "Laundry"] {
            assert!(
                named(name, 1).is_some(),
                "{name} missing on the utility deck"
            );
        }
        assert_eq!(
            built
                .rooms
                .iter()
                .filter(|room| room.deck == 5 && room.kind == RoomKind::Berth)
                .count(),
            0
        );
        assert_eq!(
            built
                .rooms
                .iter()
                .filter(|room| room.deck == 6 && room.kind == RoomKind::Berth)
                .count(),
            0
        );
        let deck_nine_berths = built
            .rooms
            .iter()
            .filter(|room| room.deck == 9 && room.kind == RoomKind::Berth)
            .count();
        assert!(
            deck_nine_berths > 20,
            "cabin deck sampled {deck_nine_berths} berths"
        );

        let start = built
            .rooms
            .iter()
            .find(|room| room.kind == RoomKind::Stair)
            .unwrap()
            .id;
        let adj = adjacency(&built.doors, built.rooms.len());
        let mut seen = vec![false; built.rooms.len()];
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        seen[start as usize] = true;
        while let Some(node) = queue.pop_front() {
            for &next in &adj[node as usize] {
                if !seen[next as usize] {
                    seen[next as usize] = true;
                    queue.push_back(next);
                }
            }
        }
        let missed: Vec<_> = built
            .rooms
            .iter()
            .filter(|room| !seen[room.id as usize])
            .take(8)
            .map(|room| format!("{}#{} d{}", room.name, room.id, room.deck))
            .collect();
        assert!(missed.is_empty(), "disconnected: {}", missed.join(", "));
    }

    fn adjacency(doors: &[Door], room_count: usize) -> Vec<Vec<u32>> {
        let mut adj = vec![Vec::new(); room_count];
        for door in doors {
            adj[door.from_room as usize].push(door.to_room);
            adj[door.to_room as usize].push(door.from_room);
        }
        adj
    }
}
