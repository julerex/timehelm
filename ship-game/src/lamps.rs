//! At most eight lamps in cabins, galleys, and dining rooms.
//!
//! The clip shader reads this array. It does not read Bevy point lights.

use bevy::prelude::*;
use sim::{polygon_centroid, RoomKind, RoomView, DECK_HEIGHT_M};

pub(crate) const LAMP_LIMIT: usize = 8;
pub(crate) const LAMP_RADIUS_M: f32 = 6.0;
/// Metres above the deck datum. The floor slab is 0.22 m and partitions rise 2.4 m
/// from the floor, so this sits just under the cabin ceiling.
const LAMP_ABOVE_DECK_M: f32 = 2.2;

pub(crate) fn place_lamps(rooms: &[RoomView]) -> Vec<Vec4> {
    rooms
        .iter()
        .filter(|room| lit_room(room.kind))
        .take(LAMP_LIMIT)
        .map(lamp_at)
        .collect()
}

pub(crate) fn lamp_uniform(rooms: &[RoomView]) -> [Vec4; LAMP_LIMIT] {
    let mut uniform = [Vec4::ZERO; LAMP_LIMIT];
    for (slot, lamp) in uniform.iter_mut().zip(place_lamps(rooms)) {
        *slot = lamp;
    }
    uniform
}

/// Cubic falloff. 1 at the lamp, about one half at `radius`, and below 0.05 at 20 m
/// when the radius is [`LAMP_RADIUS_M`]. The same expression is in `ship_clip_forward.wgsl`.
#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "the clip shader evaluates this falloff; tests lock the numbers"
    )
)]
pub(crate) fn lamp_gain(world_pos: Vec3, lamps: &[Vec4]) -> f32 {
    lamps.iter().map(|lamp| one_lamp(world_pos, *lamp)).sum()
}

fn lit_room(kind: RoomKind) -> bool {
    match kind {
        RoomKind::Berth | RoomKind::Galley | RoomKind::Restaurant | RoomKind::Mess => true,
        RoomKind::Corridor
        | RoomKind::Stair
        | RoomKind::Head
        | RoomKind::Hydroponics
        | RoomKind::Fishing
        | RoomKind::ColdStore
        | RoomKind::DryStore
        | RoomKind::Bakery
        | RoomKind::Desalination
        | RoomKind::WaterStore
        | RoomKind::WasteTreatment
        | RoomKind::Recycling
        | RoomKind::Carpentry
        | RoomKind::Sewing
        | RoomKind::Metalwork
        | RoomKind::Laundry
        | RoomKind::TimberStore
        | RoomKind::MetalStore
        | RoomKind::Clinic
        | RoomKind::Infirmary
        | RoomKind::Engineering
        | RoomKind::Electrical
        | RoomKind::FuelStore
        | RoomKind::Bridge => false,
    }
}

fn lamp_at(room: &RoomView) -> Vec4 {
    let (x, y) = polygon_centroid(&room.polygon);
    Vec4::new(x, y, lamp_z(room.deck), LAMP_RADIUS_M)
}

fn lamp_z(deck: u8) -> f32 {
    deck as f32 * DECK_HEIGHT_M + LAMP_ABOVE_DECK_M
}

fn one_lamp(world_pos: Vec3, lamp: Vec4) -> f32 {
    let radius = lamp.w;
    if radius <= 0.0 {
        return 0.0;
    }
    let q = world_pos.distance(lamp.truncate()) / radius;
    1.0 / (1.0 + q * q * q)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim::DECK_HEIGHT_M;

    fn room(id: u32, kind: RoomKind, deck: u8, x: f32, y: f32) -> RoomView {
        RoomView {
            id,
            deck,
            kind,
            name: kind.label().to_string(),
            polygon: vec![[x, y], [x + 4.0, y], [x + 4.0, y + 3.0], [x, y + 3.0]],
        }
    }

    #[test]
    fn lamp_gain_falls_off_by_20_m() {
        let lamp = Vec4::new(4.0, 8.0, 2.2, LAMP_RADIUS_M);
        let at_lamp = lamp_gain(lamp.truncate(), &[lamp]);
        let away = lamp_gain(lamp.truncate() + Vec3::X * 20.0, &[lamp]);
        assert!(at_lamp > 0.5, "gain at the lamp was {at_lamp}");
        assert!(away < 0.05, "gain at 20 m was {away}");
        assert_eq!(lamp_gain(Vec3::ZERO, &[Vec4::ZERO]), 0.0);
    }

    fn expected_lamp(room: &RoomView) -> Vec4 {
        let (x, y) = polygon_centroid(&room.polygon);
        Vec4::new(
            x,
            y,
            room.deck as f32 * DECK_HEIGHT_M + super::LAMP_ABOVE_DECK_M,
            LAMP_RADIUS_M,
        )
    }

    #[test]
    fn place_lamps_caps_at_eight() {
        let cabins: Vec<RoomView> = (0..10)
            .map(|id| room(id, RoomKind::Berth, 9, id as f32 * 12.0, 20.0))
            .collect();
        let mut rooms = vec![room(100, RoomKind::Corridor, 9, -30.0, -30.0)];
        rooms.extend(cabins.iter().cloned());
        let lamps = place_lamps(&rooms);
        let expected: Vec<Vec4> = cabins.iter().take(LAMP_LIMIT).map(expected_lamp).collect();
        assert_eq!(lamps, expected);

        let galley = room(200, RoomKind::Galley, 3, 0.0, 40.0);
        let bakery = room(201, RoomKind::Bakery, 3, 20.0, 40.0);
        let dining = room(202, RoomKind::Restaurant, 3, 40.0, 40.0);
        let mess = room(203, RoomKind::Mess, 4, 60.0, 40.0);
        let service = place_lamps(&[galley.clone(), bakery, dining.clone(), mess.clone()]);
        let expected_service = vec![
            expected_lamp(&galley),
            expected_lamp(&dining),
            expected_lamp(&mess),
        ];
        assert_eq!(service, expected_service);
    }
}
