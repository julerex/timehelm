//! Room layout inside the hull. Rectangles share edges; those edges become doors.

use glam::Vec2;

use crate::geom::{rect_inside, rect_polygon};
use crate::hull::{hull_for_deck, UPPER_DECK_START};
use crate::kinds::RoomKind;
use crate::recipes::RecipeId;
use crate::world::{Deck, Door, Room, Station};

pub const DECK_COUNT: usize = 20;
pub const DECK_HEIGHT_M: f32 = 3.0;

const LOWER_CORRIDOR_Y0: f32 = 28.0;
const LOWER_CORRIDOR_Y1: f32 = 34.0;
const UPPER_STBD_Y0: f32 = 13.0;
const UPPER_STBD_Y1: f32 = 17.0;
const UPPER_PORT_Y0: f32 = 43.0;
const UPPER_PORT_Y1: f32 = 47.0;

pub const DECK_NAMES: [&str; DECK_COUNT] = [
    "Engine Deck",
    "Tank Deck",
    "Hold",
    "Garden Deck",
    "Upper Garden",
    "Galley Deck",
    "Mess Deck",
    "Hospital Deck",
    "Carpentry Deck",
    "Metal Deck",
    "Textile Deck",
    "Fishing Deck",
    "Bridge Deck",
    "Berth Deck A",
    "Berth Deck B",
    "Berth Deck C",
    "Berth Deck D",
    "Berth Deck E",
    "Berth Deck F",
    "Berth Deck G",
];

#[derive(Clone, Copy)]
struct Draft {
    deck: u8,
    kind: RoomKind,
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
    let lower = hull_for_deck(0);
    let upper = hull_for_deck(UPPER_DECK_START);
    let stair = choose_stair(&lower, &upper)?;
    let mut drafts = Vec::new();

    for deck in 0..DECK_COUNT {
        let hull = if deck >= UPPER_DECK_START {
            &upper
        } else {
            &lower
        };
        place_deck(&mut drafts, hull, deck as u8, stair)?;
    }

    let rooms = drafts
        .iter()
        .enumerate()
        .map(|(id, draft)| Room {
            id: id as u32,
            deck: draft.deck,
            kind: draft.kind,
            name: format!("{} {}-{}", draft.kind.label(), draft.deck, id),
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

fn missing_kinds(rooms: &[Room]) -> Vec<RoomKind> {
    RoomKind::ALL
        .into_iter()
        .filter(|kind| !rooms.iter().any(|room| room.kind == *kind))
        .collect()
}

fn choose_stair(lower: &[Vec2], upper: &[Vec2]) -> Result<Draft, String> {
    let mut x = 96.0;
    while x <= 200.0 {
        let stair = Draft {
            deck: 0,
            kind: RoomKind::Stair,
            x0: x,
            y0: 7.0,
            x1: x + 8.0,
            y1: 13.0,
        };
        let connector = Draft {
            deck: 0,
            kind: RoomKind::Corridor,
            x0: stair.x0,
            y0: stair.y1,
            x1: stair.x1,
            y1: LOWER_CORRIDOR_Y0,
        };
        if rect_inside(lower, stair.x0, stair.y0, stair.x1, stair.y1)
            && rect_inside(upper, stair.x0, stair.y0, stair.x1, stair.y1)
            && rect_inside(
                lower,
                connector.x0,
                connector.y0,
                connector.x1,
                connector.y1,
            )
        {
            return Ok(stair);
        }
        x += 4.0;
    }
    Err("no stair footprint fits both the lower hull and the upper side band".into())
}

fn place_deck(
    drafts: &mut Vec<Draft>,
    hull: &[Vec2],
    deck: u8,
    stair: Draft,
) -> Result<(), String> {
    let upper = deck as usize >= UPPER_DECK_START;
    place_corridors(drafts, hull, deck, upper);
    drafts.push(Draft { deck, ..stair });
    if !upper {
        drafts.push(Draft {
            deck,
            kind: RoomKind::Corridor,
            x0: stair.x0,
            y0: stair.y1,
            x1: stair.x1,
            y1: LOWER_CORRIDOR_Y0,
        });
    }
    place_program(drafts, hull, deck, upper)?;
    Ok(())
}

fn place_corridors(drafts: &mut Vec<Draft>, hull: &[Vec2], deck: u8, upper: bool) {
    if upper {
        place_corridor_run(
            drafts,
            hull,
            deck,
            88.0,
            250.0,
            UPPER_STBD_Y0,
            UPPER_STBD_Y1,
        );
        place_corridor_run(
            drafts,
            hull,
            deck,
            88.0,
            250.0,
            UPPER_PORT_Y0,
            UPPER_PORT_Y1,
        );
        place_upper_link(drafts, hull, deck);
    } else {
        place_corridor_run(
            drafts,
            hull,
            deck,
            36.0,
            286.0,
            LOWER_CORRIDOR_Y0,
            LOWER_CORRIDOR_Y1,
        );
    }
}

fn place_corridor_run(
    drafts: &mut Vec<Draft>,
    hull: &[Vec2],
    deck: u8,
    x_start: f32,
    x_end: f32,
    y0: f32,
    y1: f32,
) {
    let mut x = x_start;
    while x + 12.0 <= x_end {
        if rect_inside(hull, x, y0, x + 16.0, y1)
            && !overlaps_any(drafts, deck, x, y0, x + 16.0, y1)
        {
            drafts.push(Draft {
                deck,
                kind: RoomKind::Corridor,
                x0: x,
                y0,
                x1: x + 16.0,
                y1,
            });
            x += 16.0;
        } else {
            x += 4.0;
        }
    }
}

fn place_program(
    drafts: &mut Vec<Draft>,
    hull: &[Vec2],
    deck: u8,
    upper: bool,
) -> Result<(), String> {
    for kind in required(deck) {
        if !place_kind(drafts, hull, deck, upper, *kind) {
            return Err(format!("could not place {} on deck {deck}", kind.label()));
        }
    }
    let filler = filler_kind(deck);
    let mut extras = 0;
    let cap = if (13..20).contains(&deck) { 10 } else { 6 };
    while extras < cap && place_kind(drafts, hull, deck, upper, filler) {
        extras += 1;
    }
    Ok(())
}

fn required(deck: u8) -> &'static [RoomKind] {
    match deck {
        0 => &[
            RoomKind::Engineering,
            RoomKind::Electrical,
            RoomKind::FuelStore,
        ],
        1 => &[
            RoomKind::Desalination,
            RoomKind::WaterStore,
            RoomKind::WasteTreatment,
            RoomKind::Recycling,
        ],
        2 => &[
            RoomKind::ColdStore,
            RoomKind::DryStore,
            RoomKind::TimberStore,
            RoomKind::MetalStore,
        ],
        3 | 4 => &[
            RoomKind::Hydroponics,
            RoomKind::Hydroponics,
            RoomKind::Hydroponics,
            RoomKind::Hydroponics,
        ],
        5 => &[RoomKind::Galley, RoomKind::Bakery, RoomKind::Bakery],
        6 => &[RoomKind::Restaurant, RoomKind::Mess, RoomKind::Mess],
        7 => &[RoomKind::Clinic, RoomKind::Infirmary],
        8 => &[RoomKind::Carpentry, RoomKind::Carpentry],
        9 => &[RoomKind::Metalwork, RoomKind::Metalwork],
        10 => &[RoomKind::Sewing, RoomKind::Sewing, RoomKind::Laundry],
        11 => &[RoomKind::Fishing, RoomKind::Fishing],
        12 => &[RoomKind::Bridge],
        13..=19 => &[
            RoomKind::Berth,
            RoomKind::Berth,
            RoomKind::Berth,
            RoomKind::Berth,
            RoomKind::Head,
            RoomKind::Head,
        ],
        _ => &[],
    }
}

fn filler_kind(deck: u8) -> RoomKind {
    match deck {
        0 => RoomKind::Engineering,
        1 => RoomKind::WaterStore,
        2 => RoomKind::DryStore,
        3 | 4 => RoomKind::Hydroponics,
        5 => RoomKind::Galley,
        6 => RoomKind::Mess,
        7 => RoomKind::Clinic,
        8 => RoomKind::Carpentry,
        9 => RoomKind::Metalwork,
        10 => RoomKind::Sewing,
        11 => RoomKind::Fishing,
        12 => RoomKind::Bridge,
        _ => RoomKind::Berth,
    }
}

fn place_kind(
    drafts: &mut Vec<Draft>,
    hull: &[Vec2],
    deck: u8,
    upper: bool,
    kind: RoomKind,
) -> bool {
    let (width, depth) = room_size(kind, upper);
    let mut x = if upper { 90.0 } else { 40.0 };
    let x_end = if upper { 210.0 } else { 280.0 };
    while x + width <= x_end {
        if let Some(draft) = candidate(
            hull,
            drafts,
            Placement {
                deck,
                upper,
                kind,
                x,
                width,
                depth,
            },
        ) {
            drafts.push(draft);
            return true;
        }
        x += 4.0;
    }
    false
}

struct Placement {
    deck: u8,
    upper: bool,
    kind: RoomKind,
    x: f32,
    width: f32,
    depth: f32,
}

fn candidate(hull: &[Vec2], drafts: &[Draft], place: Placement) -> Option<Draft> {
    let Placement {
        deck,
        upper,
        kind,
        x,
        width,
        depth,
    } = place;
    let bands: Vec<(f32, f32)> = if upper {
        vec![
            (UPPER_PORT_Y1, UPPER_PORT_Y1 + depth),
            (UPPER_STBD_Y0 - depth, UPPER_STBD_Y0),
        ]
    } else {
        vec![
            (LOWER_CORRIDOR_Y1, LOWER_CORRIDOR_Y1 + depth),
            (LOWER_CORRIDOR_Y0 - depth, LOWER_CORRIDOR_Y0),
        ]
    };
    for (y0, y1) in bands {
        if y1 - y0 < 4.0 || y0 < 1.0 {
            continue;
        }
        if rect_inside(hull, x, y0, x + width, y1)
            && !overlaps_any(drafts, deck, x, y0, x + width, y1)
        {
            return Some(Draft {
                deck,
                kind,
                x0: x,
                y0,
                x1: x + width,
                y1,
            });
        }
    }
    None
}

fn room_size(kind: RoomKind, upper: bool) -> (f32, f32) {
    let (width, depth): (f32, f32) = match kind {
        RoomKind::Berth | RoomKind::Head => (8.0, 8.0),
        RoomKind::Hydroponics => (18.0, 14.0),
        RoomKind::Bridge | RoomKind::Clinic | RoomKind::Galley | RoomKind::Bakery => (16.0, 12.0),
        _ => (14.0, 12.0),
    };
    if upper {
        (width.min(16.0), depth.min(8.0))
    } else {
        (width, depth)
    }
}

fn overlaps_any(drafts: &[Draft], deck: u8, x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
    drafts.iter().any(|other| {
        other.deck == deck
            && x0 < other.x1 - 0.05
            && x1 > other.x0 + 0.05
            && y0 < other.y1 - 0.05
            && y1 > other.y0 + 0.05
    })
}

fn doors_for(drafts: &[Draft]) -> Vec<Door> {
    let mut doors = Vec::new();
    let mut id = 0u32;
    for i in 0..drafts.len() {
        for j in (i + 1)..drafts.len() {
            let connect = if drafts[i].deck == drafts[j].deck {
                shares_edge(&drafts[i], &drafts[j])
            } else {
                drafts[i].kind == RoomKind::Stair
                    && drafts[j].kind == RoomKind::Stair
                    && (drafts[i].deck as i16 - drafts[j].deck as i16).abs() == 1
                    && same_footprint(&drafts[i], &drafts[j])
            };
            if connect {
                doors.push(Door {
                    id,
                    from_room: i as u32,
                    to_room: j as u32,
                });
                id += 1;
            }
        }
    }
    doors
}

fn shares_edge(a: &Draft, b: &Draft) -> bool {
    let y_overlap = a.y1.min(b.y1) - a.y0.max(b.y0);
    let x_touch = (a.x1 - b.x0).abs() < 0.8 || (b.x1 - a.x0).abs() < 0.8;
    if x_touch && y_overlap > 1.0 {
        return true;
    }
    let x_overlap = a.x1.min(b.x1) - a.x0.max(b.x0);
    let y_touch = (a.y1 - b.y0).abs() < 0.8 || (b.y1 - a.y0).abs() < 0.8;
    y_touch && x_overlap > 1.0
}

fn same_footprint(a: &Draft, b: &Draft) -> bool {
    (a.x0 - b.x0).abs() < 0.1
        && (a.y0 - b.y0).abs() < 0.1
        && (a.x1 - b.x1).abs() < 0.1
        && (a.y1 - b.y1).abs() < 0.1
}

fn place_upper_link(drafts: &mut Vec<Draft>, hull: &[Vec2], deck: u8) {
    let starboard: Vec<(f32, f32, f32)> = drafts
        .iter()
        .filter(|draft| draft.deck == deck && draft.kind == RoomKind::Corridor && draft.y1 <= 18.0)
        .map(|draft| (draft.x0, draft.x1, draft.y1))
        .collect();
    let port: Vec<(f32, f32, f32)> = drafts
        .iter()
        .filter(|draft| draft.deck == deck && draft.kind == RoomKind::Corridor && draft.y0 >= 40.0)
        .map(|draft| (draft.x0, draft.x1, draft.y0))
        .collect();
    for (sx0, sx1, sy1) in &starboard {
        for (px0, px1, py0) in &port {
            let x0 = sx0.max(*px0);
            let x1 = sx1.min(*px1);
            if x1 - x0 < 4.0 {
                continue;
            }
            if rect_inside(hull, x0, *sy1, x1, *py0)
                && !overlaps_any(drafts, deck, x0, *sy1, x1, *py0)
            {
                drafts.push(Draft {
                    deck,
                    kind: RoomKind::Corridor,
                    x0,
                    y0: *sy1,
                    x1,
                    y1: *py0,
                });
                return;
            }
        }
    }
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
