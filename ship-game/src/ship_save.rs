//! Compressed on-disk ship layout (`DeckLayouts` / [`CellBox`]) for save/load and agent inspection.

use crate::cell::{
    Cell, Entity, EntityId, EntityKind, Fixture, Material, RoomCatalog, RoomCategory, RoomId,
};
use crate::cell_box::{CellBox, CellIndex, BEAM, DECKS, LENGTH};
use crate::deck_layout::{DeckLayouts, DeckMeta, NUM_DECKS};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io;
use std::path::{Path, PathBuf};

pub const SAVE_VERSION: u32 = 2;
pub const SAVE_VERSION_V1: u32 = 1;
const MAGIC: &[u8; 6] = b"THSHP1";

/// Repository-relative directory for ship saves (`timehelm/saved_ships`).
#[must_use]
pub fn saved_ships_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../saved_ships")
}

/// Default save path agents and Ctrl+S use as `latest`.
#[must_use]
pub fn latest_save_path() -> PathBuf {
    saved_ships_dir().join("latest.ship.zst")
}

/// Empty layout used before the player picks a save (no procedural generation).
#[must_use]
pub fn empty_deck_layouts() -> DeckLayouts {
    DeckLayouts {
        cells: CellBox::new(),
        decks: (0..NUM_DECKS)
            .map(|_| DeckMeta {
                rooms: RoomCatalog::default(),
            })
            .collect(),
        entities: HashMap::new(),
    }
}

/// Filenames of `*.ship.zst` in [`saved_ships_dir`] (native only).
#[must_use]
pub fn list_saved_ship_files() -> Vec<String> {
    let dir = saved_ships_dir();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.ends_with(".ship.zst"))
        .collect();
    names.sort();
    names
}

/// Load a save by filename from [`saved_ships_dir`].
pub fn load_save_by_filename(filename: &str) -> Result<DeckLayouts, ShipSaveError> {
    let path = saved_ships_dir().join(filename);
    read_save(&path)
}

/// HTTP path for the WASM save manifest.
#[must_use]
pub fn saved_ship_manifest_url() -> &'static str {
    "/saved_ships/manifest.json"
}

/// HTTP path for a WASM save file.
#[must_use]
pub fn saved_ship_url(filename: &str) -> String {
    format!("/saved_ships/{filename}")
}

/// Manifest served to the WASM client (`client/public/saved_ships/manifest.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedShipManifest {
    pub saves: Vec<String>,
}

/// Write `manifest.json` listing every `*.ship.zst` in `dir` (build / dev helper).
pub fn write_save_manifest(dir: &Path) -> Result<(), ShipSaveError> {
    let mut saves = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if name.ends_with(".ship.zst") {
                saves.push(name.to_owned());
            }
        }
    }
    saves.sort();
    let manifest = SavedShipManifest { saves };
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| ShipSaveError::Decode(e.to_string()))?;
    std::fs::write(dir.join("manifest.json"), json)?;
    Ok(())
}

/// On-disk envelope (magic + zstd-compressed bincode payload).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedShipFile {
    pub version: u32,
    pub layouts: SavedDeckLayouts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedDeckLayouts {
    pub occupied: Vec<SavedOccupiedCell>,
    pub decks: Vec<SavedDeckMeta>,
    #[serde(default)]
    pub entities: Vec<SavedEntity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedOccupiedCell {
    pub x: u16,
    pub y: u16,
    pub z: u8,
    pub cell: SavedCell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedCell {
    pub wall1: u8,
    pub wall2: u8,
    pub wall3: u8,
    pub wall4: u8,
    pub floor: u8,
    pub room: u32,
    #[serde(default)]
    pub fixtures: Vec<u8>,
    /// v1 only; migrated into [`SavedDeckLayouts::entities`] on load. Always empty in v2 saves.
    #[serde(default)]
    pub agents: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedEntity {
    pub id: u64,
    pub kind: u8,
    pub x: u16,
    pub y: u16,
    pub z: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedDeckMeta {
    pub rooms: Vec<SavedRoom>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedRoom {
    pub id: u32,
    pub name: String,
    pub deck: u8,
    pub category: u8,
}

impl From<&DeckLayouts> for SavedShipFile {
    fn from(layouts: &DeckLayouts) -> Self {
        Self {
            version: SAVE_VERSION,
            layouts: SavedDeckLayouts::from(layouts),
        }
    }
}

impl From<&DeckLayouts> for SavedDeckLayouts {
    fn from(layouts: &DeckLayouts) -> Self {
        let occupied = layouts
            .cells
            .iter_occupied()
            .map(|(index, cell)| SavedOccupiedCell {
                x: index.x,
                y: index.y,
                z: index.z,
                cell: SavedCell::from(cell),
            })
            .collect();
        let decks = layouts
            .decks
            .iter()
            .map(|meta| SavedDeckMeta::from(&meta.rooms))
            .collect();
        let entities = layouts
            .entities
            .iter()
            .map(|(id, entity)| SavedEntity::from((*id, entity)))
            .collect();
        Self {
            occupied,
            decks,
            entities,
        }
    }
}

impl From<(EntityId, &Entity)> for SavedEntity {
    fn from((id, entity): (EntityId, &Entity)) -> Self {
        Self {
            id: id.0,
            kind: entity.kind.to_tag(),
            x: entity.location.0,
            y: entity.location.1,
            z: entity.location.2,
        }
    }
}

impl From<&Cell> for SavedCell {
    fn from(cell: &Cell) -> Self {
        Self {
            wall1: cell.wall1 as u8,
            wall2: cell.wall2 as u8,
            wall3: cell.wall3 as u8,
            wall4: cell.wall4 as u8,
            floor: cell.floor as u8,
            room: cell.room.0,
            fixtures: cell.fixtures.iter().map(|f| f.to_tag()).collect(),
            agents: Vec::new(),
        }
    }
}

impl From<&RoomCatalog> for SavedDeckMeta {
    fn from(catalog: &RoomCatalog) -> Self {
        let mut rooms: Vec<SavedRoom> = catalog
            .rooms
            .values()
            .map(|room| SavedRoom {
                id: room.id,
                name: room.name.to_string(),
                deck: room.deck,
                category: room_category_to_u8(room.category),
            })
            .collect();
        rooms.sort_by_key(|r| r.id);
        Self { rooms }
    }
}

impl TryFrom<SavedShipFile> for DeckLayouts {
    type Error = ShipSaveError;

    fn try_from(file: SavedShipFile) -> Result<Self, Self::Error> {
        match file.version {
            SAVE_VERSION => SavedDeckLayouts::try_into(file.layouts),
            SAVE_VERSION_V1 => layouts_from_v1(file.layouts),
            v => Err(ShipSaveError::UnsupportedVersion(v)),
        }
    }
}

fn layouts_from_v1(saved: SavedDeckLayouts) -> Result<DeckLayouts, ShipSaveError> {
    if saved.decks.len() != NUM_DECKS {
        return Err(ShipSaveError::DeckCount {
            expected: NUM_DECKS,
            got: saved.decks.len(),
        });
    }

    let mut cells = CellBox::new();
    let mut entities = HashMap::new();
    for entry in saved.occupied {
        let index = CellIndex::new(entry.x, entry.y, entry.z).ok_or(
            ShipSaveError::InvalidCellIndex {
                x: entry.x,
                y: entry.y,
                z: entry.z,
            },
        )?;
        let location = index.to_location();
        let agent_ids = entry.cell.agents.clone();
        let cell = Cell::try_from(entry.cell)?;
        cells.insert(index, cell);
        for id in agent_ids {
            entities.insert(
                EntityId(id),
                Entity {
                    kind: EntityKind::SimHuman,
                    location,
                },
            );
        }
    }

    let decks = saved
        .decks
        .into_iter()
        .map(|meta| -> Result<DeckMeta, ShipSaveError> {
            Ok(DeckMeta {
                rooms: RoomCatalog::try_from(meta)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(DeckLayouts {
        cells,
        decks,
        entities,
    })
}

impl TryFrom<SavedDeckLayouts> for DeckLayouts {
    type Error = ShipSaveError;

    fn try_from(saved: SavedDeckLayouts) -> Result<Self, Self::Error> {
        if saved.decks.len() != NUM_DECKS {
            return Err(ShipSaveError::DeckCount {
                expected: NUM_DECKS,
                got: saved.decks.len(),
            });
        }

        let mut cells = CellBox::new();
        for entry in saved.occupied {
            let index = CellIndex::new(entry.x, entry.y, entry.z).ok_or(
                ShipSaveError::InvalidCellIndex {
                    x: entry.x,
                    y: entry.y,
                    z: entry.z,
                },
            )?;
            let cell = Cell::try_from(entry.cell)?;
            cells.insert(index, cell);
        }

        let decks = saved
            .decks
            .into_iter()
            .map(|meta| -> Result<DeckMeta, ShipSaveError> {
                Ok(DeckMeta {
                    rooms: RoomCatalog::try_from(meta)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut entities = HashMap::new();
        for saved_entity in saved.entities {
            let (id, entity) = <(EntityId, Entity)>::try_from(saved_entity)?;
            entities.insert(id, entity);
        }

        Ok(DeckLayouts {
            cells,
            decks,
            entities,
        })
    }
}

impl TryFrom<SavedCell> for Cell {
    type Error = ShipSaveError;

    fn try_from(saved: SavedCell) -> Result<Self, Self::Error> {
        let mut fixtures = Vec::with_capacity(saved.fixtures.len());
        for tag in saved.fixtures {
            fixtures.push(
                Fixture::from_tag(tag).ok_or(ShipSaveError::InvalidFixtureTag(tag))?,
            );
        }
        Ok(Self {
            wall1: material_from_u8(saved.wall1)?,
            wall2: material_from_u8(saved.wall2)?,
            wall3: material_from_u8(saved.wall3)?,
            wall4: material_from_u8(saved.wall4)?,
            floor: material_from_u8(saved.floor)?,
            room: RoomId(saved.room),
            fixtures,
        })
    }
}

impl TryFrom<SavedEntity> for (EntityId, Entity) {
    type Error = ShipSaveError;

    fn try_from(saved: SavedEntity) -> Result<Self, Self::Error> {
        let kind = EntityKind::from_tag(saved.kind).ok_or(ShipSaveError::InvalidEntityKind(
            saved.kind,
        ))?;
        let location = (saved.x, saved.y, saved.z);
        CellIndex::from_location(location).ok_or(ShipSaveError::InvalidCellIndex {
            x: saved.x,
            y: saved.y,
            z: saved.z,
        })?;
        Ok((
            EntityId(saved.id),
            Entity { kind, location },
        ))
    }
}

impl TryFrom<SavedDeckMeta> for RoomCatalog {
    type Error = ShipSaveError;

    fn try_from(saved: SavedDeckMeta) -> Result<Self, Self::Error> {
        let entries = saved
            .rooms
            .into_iter()
            .map(|room| {
                let category = room_category_from_u8(room.category)?;
                let name: &'static str = Box::leak(room.name.into_boxed_str());
                Ok((RoomId(room.id), name, room.deck, category))
            })
            .collect::<Result<Vec<_>, ShipSaveError>>()?;
        Ok(RoomCatalog::from_persisted(entries))
    }
}

#[derive(Debug)]
pub enum ShipSaveError {
    Io(io::Error),
    Decode(String),
    UnsupportedVersion(u32),
    DeckCount { expected: usize, got: usize },
    InvalidCellIndex { x: u16, y: u16, z: u8 },
    InvalidMaterial(u8),
    InvalidRoomCategory(u8),
    InvalidFixtureTag(u8),
    InvalidEntityKind(u8),
}

impl std::fmt::Display for ShipSaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Decode(msg) => write!(f, "decode error: {msg}"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported save version {v}"),
            Self::DeckCount { expected, got } => {
                write!(f, "expected {expected} decks, got {got}")
            }
            Self::InvalidCellIndex { x, y, z } => {
                write!(f, "cell index ({x}, {y}, {z}) out of range")
            }
            Self::InvalidMaterial(m) => write!(f, "invalid material code {m}"),
            Self::InvalidRoomCategory(c) => write!(f, "invalid room category code {c}"),
            Self::InvalidFixtureTag(t) => write!(f, "invalid fixture tag {t}"),
            Self::InvalidEntityKind(k) => write!(f, "invalid entity kind {k}"),
        }
    }
}

impl std::error::Error for ShipSaveError {}

impl From<io::Error> for ShipSaveError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

/// Write a compressed save file (creates parent directories).
pub fn write_save(path: &Path, layouts: &DeckLayouts) -> Result<(), ShipSaveError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let bytes = encode_save(layouts)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

/// Read and decode a compressed save file.
pub fn read_save(path: &Path) -> Result<DeckLayouts, ShipSaveError> {
    let bytes = std::fs::read(path)?;
    decode_save(&bytes)
}

pub fn encode_save(layouts: &DeckLayouts) -> Result<Vec<u8>, ShipSaveError> {
    let file = SavedShipFile::from(layouts);
    let payload = bincode::serialize(&file).map_err(|e| ShipSaveError::Decode(e.to_string()))?;
    let mut out = Vec::with_capacity(MAGIC.len() + payload.len());
    out.extend_from_slice(MAGIC);
    let compressed =
        zstd::encode_all(&payload[..], 3).map_err(|e| ShipSaveError::Decode(e.to_string()))?;
    out.extend_from_slice(&compressed);
    Ok(out)
}

pub fn decode_save(bytes: &[u8]) -> Result<DeckLayouts, ShipSaveError> {
    if bytes.len() < MAGIC.len() {
        return Err(ShipSaveError::Decode("file too short".into()));
    }
    if &bytes[..MAGIC.len()] != MAGIC {
        return Err(ShipSaveError::Decode("bad magic".into()));
    }
    let payload = zstd::decode_all(&bytes[MAGIC.len()..])
        .map_err(|e| ShipSaveError::Decode(e.to_string()))?;
    let file: SavedShipFile =
        bincode::deserialize(&payload).map_err(|e| ShipSaveError::Decode(e.to_string()))?;
    file.try_into()
}

/// Human-readable report for agents (`make inspect-ship`).
pub fn analyze_save(path: &Path, layouts: &DeckLayouts) -> String {
    let file = SavedShipFile::from(layouts);
    let mut out = String::new();
    let _ = writeln!(out, "path: {}", path.display());
    let _ = writeln!(out, "format_version: {}", file.version);
    analyze_cell_box(&layouts.cells, &mut out);
    let _ = writeln!(out, "entities: {}", layouts.entities.len());
    let sample_entities: Vec<_> = layouts.entities.iter().take(4).collect();
    if !sample_entities.is_empty() {
        let _ = writeln!(out, "sample_entities (up to 4):");
        for (id, entity) in sample_entities {
            let (x, y, z) = entity.location;
            let _ = writeln!(
                out,
                "  id={} kind={:?} at ({x},{y},{z})",
                id.0, entity.kind
            );
        }
    }
    let _ = writeln!(out, "decks: {}", layouts.decks.len());
    for (i, deck) in layouts.decks.iter().enumerate() {
        let _ = writeln!(out, "deck[{i}] rooms: {}", deck.rooms.rooms.len());
    }
    out
}

pub fn analyze_cell_box(box_: &CellBox, out: &mut String) {
    let occupied = box_.iter_occupied().count();
    let volume = CellBox::volume();
    let _ = writeln!(
        out,
        "CellBox: occupied={occupied} volume={volume} ({:.2}% full)",
        100.0 * occupied as f64 / volume as f64
    );
    let _ = writeln!(
        out,
        "CellBox dims: LENGTH={LENGTH} BEAM={BEAM} DECKS={DECKS}"
    );
    let _ = write!(out, "per_deck_occupied:");
    for deck in 0..DECKS {
        let n = box_.deck_occupied(deck as u8);
        let _ = write!(out, " {deck}={n}");
    }
    let _ = writeln!(out);

    let sample: Vec<_> = box_.iter_occupied().take(8).collect();
    let _ = writeln!(out, "sample_cells (up to 8):");
    for (idx, cell) in sample {
        let _ = writeln!(
            out,
            "  ({},{},{}) floor={} room={} fixtures={} walls=[{},{},{},{}]",
            idx.x,
            idx.y,
            idx.z,
            cell.floor.label(),
            cell.room.0,
            cell.fixtures.len(),
            cell.wall1.label(),
            cell.wall2.label(),
            cell.wall3.label(),
            cell.wall4.label(),
        );
    }
}

fn material_from_u8(v: u8) -> Result<Material, ShipSaveError> {
    const TABLE: [Material; Material::COUNT] = [
        Material::Open,
        Material::Hull,
        Material::Window,
        Material::CabinPartition,
        Material::Corridor,
        Material::PublicShell,
        Material::DeckBase,
        Material::Theatre,
        Material::Dining,
        Material::Buffet,
        Material::Pool,
        Material::Casino,
        Material::CabinStripeA,
        Material::CabinStripeB,
        Material::CorridorWhite,
        Material::BowAccent,
        Material::MarinePanel,
        Material::Door,
    ];
    TABLE
        .get(v as usize)
        .copied()
        .ok_or(ShipSaveError::InvalidMaterial(v))
}

fn room_category_to_u8(c: RoomCategory) -> u8 {
    match c {
        RoomCategory::Exterior => 0,
        RoomCategory::Cabin => 1,
        RoomCategory::Corridor => 2,
        RoomCategory::Public => 3,
        RoomCategory::Amenity => 4,
    }
}

fn room_category_from_u8(v: u8) -> Result<RoomCategory, ShipSaveError> {
    match v {
        0 => Ok(RoomCategory::Exterior),
        1 => Ok(RoomCategory::Cabin),
        2 => Ok(RoomCategory::Corridor),
        3 => Ok(RoomCategory::Public),
        4 => Ok(RoomCategory::Amenity),
        _ => Err(ShipSaveError::InvalidRoomCategory(v)),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_layouts_latest(layouts: &DeckLayouts) -> Result<PathBuf, ShipSaveError> {
    let dir = saved_ships_dir();
    std::fs::create_dir_all(&dir)?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let stamped = dir.join(format!("ship_{stamp}.ship.zst"));
    write_save(&stamped, layouts)?;
    let latest = latest_save_path();
    write_save(&latest, layouts)?;
    Ok(stamped)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_layouts_latest() -> Result<DeckLayouts, ShipSaveError> {
    read_save(&latest_save_path())
}

#[cfg(not(target_arch = "wasm32"))]
mod native_ui {
    use super::*;
    use bevy::prelude::*;

    /// Fired after [`DeckLayouts`] was replaced from disk (rebuild visuals in app systems).
    #[derive(Message)]
    pub struct ShipLayoutsReplaced;

    /// Fired after a successful Ctrl+S save (show HUD toast in 2D).
    #[derive(Message)]
    pub struct ShipSaveSucceeded;

    #[derive(Clone, Copy, Message)]
    enum ShipSaveRequest {
        Save,
        Load,
    }

    pub struct ShipSavePlugin;

    impl Plugin for ShipSavePlugin {
        fn build(&self, app: &mut App) {
            app.add_message::<ShipSaveRequest>()
                .add_message::<ShipLayoutsReplaced>()
                .add_message::<ShipSaveSucceeded>()
                .add_systems(Update, save_load_hotkeys)
                .add_systems(Update, handle_save_load_requests.after(save_load_hotkeys));
        }
    }

    fn save_load_hotkeys(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut requests: MessageWriter<ShipSaveRequest>,
    ) {
        let ctrl =
            keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
        if !ctrl {
            return;
        }
        if keyboard.just_pressed(KeyCode::KeyS) {
            requests.write(ShipSaveRequest::Save);
        }
        if keyboard.just_pressed(KeyCode::KeyO) {
            requests.write(ShipSaveRequest::Load);
        }
    }

    fn handle_save_load_requests(
        mut requests: MessageReader<ShipSaveRequest>,
        mut layouts: ResMut<DeckLayouts>,
        mut replaced: MessageWriter<ShipLayoutsReplaced>,
        mut saved: MessageWriter<ShipSaveSucceeded>,
    ) {
        for request in requests.read() {
            match request {
                ShipSaveRequest::Save => match save_layouts_latest(&layouts) {
                    Ok(path) => {
                        eprintln!("saved ship to {}", path.display());
                        saved.write(ShipSaveSucceeded);
                    }
                    Err(e) => eprintln!("ship save failed: {e}"),
                },
                ShipSaveRequest::Load => match load_layouts_latest() {
                    Ok(loaded) => {
                        *layouts = loaded;
                        replaced.write(ShipLayoutsReplaced);
                        eprintln!("loaded ship from {}", latest_save_path().display());
                    }
                    Err(e) => eprintln!("ship load failed: {e}"),
                },
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native_ui::{ShipLayoutsReplaced, ShipSavePlugin, ShipSaveSucceeded};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck_layout::deck_cell_layouts;
    use crate::deck_layout::CELL_SIZE_M;
    use std::path::PathBuf;

    #[test]
    fn roundtrip_procedural_layout() {
        let layouts = deck_cell_layouts(CELL_SIZE_M);
        let bytes = encode_save(&layouts).expect("encode");
        let restored = decode_save(&bytes).expect("decode");
        assert_eq!(
            restored.cells.deck_occupied(4),
            layouts.cells.deck_occupied(4)
        );
        assert_eq!(
            restored.cells.iter_occupied().count(),
            layouts.cells.iter_occupied().count()
        );
        assert_eq!(restored.entities.len(), layouts.entities.len());
    }

    #[test]
    fn roundtrip_fixtures_and_entities() {
        use crate::cell::{Bed, Entity, EntityId, EntityKind, Fixture, Shower, Toilet};

        let mut layouts = deck_cell_layouts(CELL_SIZE_M);
        let Some((index, cell)) = layouts.cells.iter_occupied_mut().next() else {
            panic!("procedural layout has no cells");
        };
        cell.fixtures = vec![
            Fixture::Bed(Bed),
            Fixture::Shower(Shower),
            Fixture::Toilet(Toilet),
        ];
        let location = index.to_location();
        layouts.entities.insert(
            EntityId(42),
            Entity {
                kind: EntityKind::SimHuman,
                location,
            },
        );

        let bytes = encode_save(&layouts).expect("encode");
        let file: SavedShipFile =
            bincode::deserialize(&zstd::decode_all(&bytes[MAGIC.len()..]).unwrap()).unwrap();
        assert_eq!(file.version, SAVE_VERSION);

        let restored = decode_save(&bytes).expect("decode");
        let restored_cell = layouts
            .cells
            .get(index)
            .expect("cell still occupied");
        assert_eq!(restored.cells.get(index).unwrap().fixtures, restored_cell.fixtures);
        assert_eq!(restored.entities.len(), 1);
        assert_eq!(
            restored.entities.get(&EntityId(42)).map(|e| e.location),
            Some(location)
        );
    }

    #[test]
    fn migrate_v1_agents_to_entities() {
        use crate::cell::EntityId;

        let layouts = deck_cell_layouts(CELL_SIZE_M);
        let Some((index, _)) = layouts.cells.iter_occupied().next() else {
            panic!("procedural layout has no cells");
        };
        let location = index.to_location();

        let v1 = SavedShipFile {
            version: SAVE_VERSION_V1,
            layouts: SavedDeckLayouts {
                occupied: vec![SavedOccupiedCell {
                    x: index.x,
                    y: index.y,
                    z: index.z,
                    cell: SavedCell {
                        wall1: Material::DeckBase as u8,
                        wall2: Material::Open as u8,
                        wall3: Material::Open as u8,
                        wall4: Material::Open as u8,
                        floor: Material::DeckBase as u8,
                        room: 0,
                        fixtures: Vec::new(),
                        agents: vec![7, 9],
                    },
                }],
                decks: (0..NUM_DECKS)
                    .map(|_| SavedDeckMeta { rooms: Vec::new() })
                    .collect(),
                entities: Vec::new(),
            },
        };

        let restored: DeckLayouts = v1.try_into().expect("migrate v1");
        assert_eq!(restored.entities.len(), 2);
        assert_eq!(
            restored.entities.get(&EntityId(7)).map(|e| e.location),
            Some(location)
        );
        assert_eq!(
            restored.entities.get(&EntityId(9)).map(|e| e.location),
            Some(location)
        );
        let cell = restored.cells.get(index).expect("cell");
        assert!(cell.fixtures.is_empty());
    }

    /// `make write-ship-default` — writes procedural layout for agent fixtures.
    #[test]
    fn write_default_ship_save() {
        let path = saved_ships_dir().join("default.ship.zst");
        let layouts = deck_cell_layouts(CELL_SIZE_M);
        write_save(&path, &layouts).expect("write default save");
        eprintln!("wrote {}", path.display());
    }

    /// Regenerates `client/public/saved_ships/manifest.json` after saves are written.
    #[test]
    fn sync_public_save_manifest() {
        let public_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../client/public/saved_ships");
        std::fs::create_dir_all(&public_dir).expect("create public saved_ships");
        for entry in std::fs::read_dir(saved_ships_dir())
            .into_iter()
            .flatten()
            .flatten()
        {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".ship.zst"))
            {
                let dest = public_dir.join(entry.file_name());
                std::fs::copy(&path, &dest).expect("copy save to public");
            }
        }
        super::write_save_manifest(&public_dir).expect("write manifest");
        eprintln!("wrote {}", public_dir.join("manifest.json").display());
    }

    /// `make inspect-ship SAVE_SHIP=...` — deserialize and print CellBox analysis.
    #[test]
    #[ignore = "agent tool: make inspect-ship (needs an existing .ship.zst)"]
    fn inspect_ship_save() {
        let path = std::env::var("SAVE_SHIP")
            .map(PathBuf::from)
            .unwrap_or_else(|_| latest_save_path());
        let layouts = read_save(&path).expect("read save");
        print!("{}", analyze_save(&path, &layouts));
    }
}
