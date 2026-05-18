//! Compressed on-disk ship layout (`DeckLayouts` / [`CellBox`]) for save/load and agent inspection.

use crate::cell::{Cell, Entity, EntityId, EntityKind, Fixture, FloorMaterial, SideMaterial};
use crate::cell_box::{CellBox, CellIndex, BEAM, DECKS, LENGTH};
use crate::deck_layout::{DeckLayouts, DeckMeta, NUM_DECKS};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io;
use std::path::{Path, PathBuf};

pub const SAVE_VERSION: u32 = 5;
const SAVE_VERSION_V4: u32 = 4;
const SAVE_VERSION_V3: u32 = 3;

/// v3 grid **y** was port→starboard; v4 is starboard→port.
fn migrate_grid_y_v3_to_v4(y: u16) -> u16 {
    (BEAM - 1) as u16 - y
}
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
        decks: (0..NUM_DECKS).map(|_| DeckMeta {}).collect(),
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
    pub side1: u8,
    pub side2: u8,
    pub side3: u8,
    pub side4: u8,
    pub floor: u8,
    #[serde(default)]
    pub fixtures: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedEntity {
    pub id: u64,
    pub kind: u8,
    pub x: u16,
    pub y: u16,
    pub z: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SavedDeckMeta {}

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
            .map(|_| SavedDeckMeta::default())
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
            side1: cell.side1 as u8,
            side2: cell.side2 as u8,
            side3: cell.side3 as u8,
            side4: cell.side4 as u8,
            floor: cell.floor as u8,
            fixtures: cell.fixtures.iter().map(|f| f.to_tag()).collect(),
        }
    }
}

impl TryFrom<SavedShipFile> for DeckLayouts {
    type Error = ShipSaveError;

    fn try_from(file: SavedShipFile) -> Result<Self, Self::Error> {
        match file.version {
            SAVE_VERSION | SAVE_VERSION_V4 => file.layouts.try_into(),
            SAVE_VERSION_V3 => {
                let mut layouts = file.layouts;
                for entry in &mut layouts.occupied {
                    entry.y = migrate_grid_y_v3_to_v4(entry.y);
                }
                for entity in &mut layouts.entities {
                    entity.y = migrate_grid_y_v3_to_v4(entity.y);
                }
                layouts.try_into()
            }
            v => Err(ShipSaveError::UnsupportedVersion(v)),
        }
    }
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

        let decks = saved.decks.into_iter().map(|_| DeckMeta {}).collect();

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
            fixtures.push(Fixture::from_tag(tag).ok_or(ShipSaveError::InvalidFixtureTag(tag))?);
        }
        Ok(Self {
            side1: side_from_u8(saved.side1)?,
            side2: side_from_u8(saved.side2)?,
            side3: side_from_u8(saved.side3)?,
            side4: side_from_u8(saved.side4)?,
            floor: floor_from_u8(saved.floor)?,
            fixtures,
        })
    }
}

impl TryFrom<SavedEntity> for (EntityId, Entity) {
    type Error = ShipSaveError;

    fn try_from(saved: SavedEntity) -> Result<Self, Self::Error> {
        let kind =
            EntityKind::from_tag(saved.kind).ok_or(ShipSaveError::InvalidEntityKind(saved.kind))?;
        let location = (saved.x, saved.y, saved.z);
        CellIndex::from_location(location).ok_or(ShipSaveError::InvalidCellIndex {
            x: saved.x,
            y: saved.y,
            z: saved.z,
        })?;
        Ok((EntityId(saved.id), Entity { kind, location }))
    }
}

#[derive(Debug)]
pub enum ShipSaveError {
    Io(io::Error),
    Decode(String),
    UnsupportedVersion(u32),
    DeckCount { expected: usize, got: usize },
    InvalidCellIndex { x: u16, y: u16, z: u8 },
    InvalidFloorMaterial(u8),
    InvalidSideMaterial(u8),
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
            Self::InvalidFloorMaterial(m) => write!(f, "invalid floor material code {m}"),
            Self::InvalidSideMaterial(m) => write!(f, "invalid side material code {m}"),
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
            let _ = writeln!(out, "  id={} kind={:?} at ({x},{y},{z})", id.0, entity.kind);
        }
    }
    let _ = writeln!(out, "decks: {}", layouts.decks.len());
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
            "  ({},{},{}) floor={} fixtures={} sides=[{},{},{},{}]",
            idx.x,
            idx.y,
            idx.z,
            cell.floor.label(),
            cell.fixtures.len(),
            cell.side1.label(),
            cell.side2.label(),
            cell.side3.label(),
            cell.side4.label(),
        );
    }
}

fn floor_from_u8(v: u8) -> Result<FloorMaterial, ShipSaveError> {
    match v {
        0 => Ok(FloorMaterial::Carpet),
        1 => Ok(FloorMaterial::Wood),
        _ => Err(ShipSaveError::InvalidFloorMaterial(v)),
    }
}

fn side_from_u8(v: u8) -> Result<SideMaterial, ShipSaveError> {
    match v {
        0 => Ok(SideMaterial::Open),
        1 => Ok(SideMaterial::MarinePanel),
        2 => Ok(SideMaterial::Door),
        3 => Ok(SideMaterial::Window),
        _ => Err(ShipSaveError::InvalidSideMaterial(v)),
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
        let restored_cell = layouts.cells.get(index).expect("cell still occupied");
        assert_eq!(
            restored.cells.get(index).unwrap().fixtures,
            restored_cell.fixtures
        );
        assert_eq!(restored.entities.len(), 1);
        assert_eq!(
            restored.entities.get(&EntityId(42)).map(|e| e.location),
            Some(location)
        );
    }

    #[test]
    fn rejects_unsupported_save_version() {
        let layouts = deck_cell_layouts(CELL_SIZE_M);
        let mut bytes = encode_save(&layouts).expect("encode");
        let payload_start = MAGIC.len();
        let mut file: SavedShipFile =
            bincode::deserialize(&zstd::decode_all(&bytes[payload_start..]).unwrap()).unwrap();
        file.version = 2;
        let payload = bincode::serialize(&file).unwrap();
        bytes.truncate(MAGIC.len());
        bytes.extend(zstd::encode_all(&payload[..], 3).unwrap());
        let err = match decode_save(&bytes) {
            Err(e) => e,
            Ok(_) => panic!("expected unsupported version error"),
        };
        assert!(matches!(err, ShipSaveError::UnsupportedVersion(2)));
    }

    #[test]
    fn loads_v3_save_with_y_axis_migration() {
        let layouts = deck_cell_layouts(CELL_SIZE_M);
        let mut bytes = encode_save(&layouts).expect("encode v4");
        let payload_start = MAGIC.len();
        let mut file: SavedShipFile =
            bincode::deserialize(&zstd::decode_all(&bytes[payload_start..]).unwrap()).unwrap();
        file.version = SAVE_VERSION_V3;
        for entry in &mut file.layouts.occupied {
            entry.y = migrate_grid_y_v3_to_v4(entry.y);
        }
        for entity in &mut file.layouts.entities {
            entity.y = migrate_grid_y_v3_to_v4(entity.y);
        }
        let payload = bincode::serialize(&file).unwrap();
        bytes.truncate(MAGIC.len());
        bytes.extend(zstd::encode_all(&payload[..], 3).unwrap());
        let restored = decode_save(&bytes).expect("v3 load");
        assert_eq!(
            restored.cells.deck_occupied(4),
            layouts.cells.deck_occupied(4)
        );
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

    /// `make refresh-ship-sides SAVE_SHIP=...` — reassign cabin doors/windows and rewrite save.
    #[test]
    #[ignore = "agent tool: make refresh-ship-sides"]
    fn refresh_ship_sides_save() {
        use crate::deck_layout::refresh_all_cabin_sides;

        let path = std::env::var("SAVE_SHIP")
            .map(PathBuf::from)
            .unwrap_or_else(|_| latest_save_path());
        let mut layouts = read_save(&path).expect("read save");
        refresh_all_cabin_sides(&mut layouts);
        write_save(&path, &layouts).expect("write save");
        let public_dir =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../client/public/saved_ships");
        std::fs::create_dir_all(&public_dir).expect("create public dir");
        if let Some(name) = path.file_name() {
            let dest = public_dir.join(name);
            std::fs::copy(&path, &dest).expect("copy to public");
        }
        super::write_save_manifest(&public_dir).expect("manifest");
        eprintln!("refreshed cabin sides: {}", path.display());
    }

    #[test]
    fn roundtrip_door_and_window_sides() {
        use crate::cell::SideMaterial;

        let layouts = deck_cell_layouts(CELL_SIZE_M);
        let deck = layouts.deck(4);
        let mut set_door = false;
        let mut set_window = false;
        for (_, cell) in deck.iter_cells() {
            if cell.side1 == SideMaterial::Door
                || cell.side2 == SideMaterial::Door
                || cell.side3 == SideMaterial::Door
                || cell.side4 == SideMaterial::Door
            {
                set_door = true;
            }
            if cell.side1 == SideMaterial::Window
                || cell.side2 == SideMaterial::Window
                || cell.side3 == SideMaterial::Window
                || cell.side4 == SideMaterial::Window
            {
                set_window = true;
            }
        }
        assert!(set_door, "procedural layout should include doors");
        assert!(set_window, "procedural layout should include windows");

        let bytes = encode_save(&layouts).expect("encode");
        let file: SavedShipFile =
            bincode::deserialize(&zstd::decode_all(&bytes[MAGIC.len()..]).unwrap()).unwrap();
        assert_eq!(file.version, SAVE_VERSION);
        let restored = decode_save(&bytes).expect("decode");
        let restored_deck = restored.deck(4);
        assert!(
            restored_deck.iter_cells().any(|(_, c)| {
                c.side1 == SideMaterial::Door
                    || c.side2 == SideMaterial::Door
                    || c.side3 == SideMaterial::Door
                    || c.side4 == SideMaterial::Door
            })
        );
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
