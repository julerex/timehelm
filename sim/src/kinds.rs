//! Room kinds, goods, and crew categories.
//!
//! New variants have to be handled at every match: layout, stations, and colours.

use serde::{Deserialize, Serialize};

/// What a room is for. Circulation kinds connect the deck; the rest are work or stores.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomKind {
    Corridor,
    Stair,
    Berth,
    Head,
    Hydroponics,
    Fishing,
    ColdStore,
    DryStore,
    Galley,
    Bakery,
    Restaurant,
    Mess,
    Desalination,
    WaterStore,
    WasteTreatment,
    Recycling,
    Carpentry,
    Sewing,
    Metalwork,
    Laundry,
    TimberStore,
    MetalStore,
    Clinic,
    Infirmary,
    Engineering,
    Electrical,
    FuelStore,
    Bridge,
}

impl RoomKind {
    pub const ALL: [RoomKind; 28] = [
        RoomKind::Corridor,
        RoomKind::Stair,
        RoomKind::Berth,
        RoomKind::Head,
        RoomKind::Hydroponics,
        RoomKind::Fishing,
        RoomKind::ColdStore,
        RoomKind::DryStore,
        RoomKind::Galley,
        RoomKind::Bakery,
        RoomKind::Restaurant,
        RoomKind::Mess,
        RoomKind::Desalination,
        RoomKind::WaterStore,
        RoomKind::WasteTreatment,
        RoomKind::Recycling,
        RoomKind::Carpentry,
        RoomKind::Sewing,
        RoomKind::Metalwork,
        RoomKind::Laundry,
        RoomKind::TimberStore,
        RoomKind::MetalStore,
        RoomKind::Clinic,
        RoomKind::Infirmary,
        RoomKind::Engineering,
        RoomKind::Electrical,
        RoomKind::FuelStore,
        RoomKind::Bridge,
    ];

    pub fn label(self) -> &'static str {
        match self {
            RoomKind::Corridor => "Corridor",
            RoomKind::Stair => "Stair",
            RoomKind::Berth => "Berth",
            RoomKind::Head => "Head",
            RoomKind::Hydroponics => "Hydroponics",
            RoomKind::Fishing => "Fishing station",
            RoomKind::ColdStore => "Cold store",
            RoomKind::DryStore => "Dry store",
            RoomKind::Galley => "Galley",
            RoomKind::Bakery => "Bakery",
            RoomKind::Restaurant => "Restaurant",
            RoomKind::Mess => "Mess",
            RoomKind::Desalination => "Desalination",
            RoomKind::WaterStore => "Water store",
            RoomKind::WasteTreatment => "Waste treatment",
            RoomKind::Recycling => "Recycling",
            RoomKind::Carpentry => "Carpentry",
            RoomKind::Sewing => "Sewing workshop",
            RoomKind::Metalwork => "Metalwork",
            RoomKind::Laundry => "Laundry",
            RoomKind::TimberStore => "Timber store",
            RoomKind::MetalStore => "Metal store",
            RoomKind::Clinic => "Clinic",
            RoomKind::Infirmary => "Infirmary",
            RoomKind::Engineering => "Engineering",
            RoomKind::Electrical => "Electrical",
            RoomKind::FuelStore => "Fuel store",
            RoomKind::Bridge => "Bridge",
        }
    }

    /// sRGB colour used by the viewer. Kept here so docs and client share one map.
    pub fn color_rgba(self) -> [f32; 4] {
        match self {
            RoomKind::Corridor => [0.45, 0.48, 0.52, 1.0],
            RoomKind::Stair => [0.35, 0.38, 0.42, 1.0],
            RoomKind::Berth => [0.55, 0.62, 0.78, 1.0],
            RoomKind::Head => [0.62, 0.78, 0.82, 1.0],
            RoomKind::Hydroponics => [0.35, 0.62, 0.38, 1.0],
            RoomKind::Fishing => [0.25, 0.48, 0.62, 1.0],
            RoomKind::ColdStore => [0.70, 0.86, 0.92, 1.0],
            RoomKind::DryStore => [0.72, 0.64, 0.48, 1.0],
            RoomKind::Galley => [0.85, 0.55, 0.32, 1.0],
            RoomKind::Bakery => [0.86, 0.72, 0.42, 1.0],
            RoomKind::Restaurant => [0.72, 0.40, 0.38, 1.0],
            RoomKind::Mess => [0.78, 0.48, 0.42, 1.0],
            RoomKind::Desalination => [0.30, 0.55, 0.72, 1.0],
            RoomKind::WaterStore => [0.40, 0.62, 0.82, 1.0],
            RoomKind::WasteTreatment => [0.42, 0.48, 0.32, 1.0],
            RoomKind::Recycling => [0.48, 0.58, 0.36, 1.0],
            RoomKind::Carpentry => [0.62, 0.46, 0.28, 1.0],
            RoomKind::Sewing => [0.62, 0.42, 0.58, 1.0],
            RoomKind::Metalwork => [0.50, 0.52, 0.56, 1.0],
            RoomKind::Laundry => [0.68, 0.70, 0.82, 1.0],
            RoomKind::TimberStore => [0.55, 0.40, 0.24, 1.0],
            RoomKind::MetalStore => [0.42, 0.44, 0.48, 1.0],
            RoomKind::Clinic => [0.82, 0.88, 0.90, 1.0],
            RoomKind::Infirmary => [0.75, 0.82, 0.84, 1.0],
            RoomKind::Engineering => [0.58, 0.32, 0.28, 1.0],
            RoomKind::Electrical => [0.78, 0.72, 0.32, 1.0],
            RoomKind::FuelStore => [0.32, 0.30, 0.28, 1.0],
            RoomKind::Bridge => [0.28, 0.40, 0.58, 1.0],
        }
    }
}

/// Tracked stock. Litres are treated as kilograms for mass balance (water density).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Good {
    FlourKg,
    WaterL,
    SaltKg,
    YeastKg,
    BreadKg,
    VegetablesKg,
    FishKg,
    MealKg,
    FibreKg,
    ClothKg,
    GarmentKg,
    TimberKg,
    PlankKg,
    MetalKg,
    ScrapKg,
    ToolKg,
    MedicalKg,
    NutrientKg,
    CompostKg,
    SeawaterL,
    BrineL,
    WaterVaporKg,
    WastewaterL,
    ProcessLossKg,
}

impl Good {
    pub fn label(self) -> &'static str {
        match self {
            Good::FlourKg => "flour (kg)",
            Good::WaterL => "water (L)",
            Good::SaltKg => "salt (kg)",
            Good::YeastKg => "yeast (kg)",
            Good::BreadKg => "bread (kg)",
            Good::VegetablesKg => "vegetables (kg)",
            Good::FishKg => "fish (kg)",
            Good::MealKg => "meals (kg)",
            Good::FibreKg => "fibre (kg)",
            Good::ClothKg => "cloth (kg)",
            Good::GarmentKg => "garments (kg)",
            Good::TimberKg => "timber (kg)",
            Good::PlankKg => "planks (kg)",
            Good::MetalKg => "metal (kg)",
            Good::ScrapKg => "scrap (kg)",
            Good::ToolKg => "tools (kg)",
            Good::MedicalKg => "medical supplies (kg)",
            Good::NutrientKg => "nutrients (kg)",
            Good::CompostKg => "compost (kg)",
            Good::SeawaterL => "seawater (L)",
            Good::BrineL => "brine (L)",
            Good::WaterVaporKg => "water vapor (kg)",
            Good::WastewaterL => "wastewater (L)",
            Good::ProcessLossKg => "process loss (kg)",
        }
    }
}

/// Who is throttled first when solar output is short.
/// Lower values are served first. Hydroponics and desalination are last.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerClass {
    Clinic,
    Galley,
    Bakery,
    Essential,
    Hydroponics,
    Desalination,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Department {
    Engineering,
    Water,
    Waste,
    Galley,
    Bakery,
    Service,
    Hydroponics,
    Fishing,
    Clinic,
    Carpentry,
    Metalwork,
    Sewing,
    Laundry,
    Bridge,
    Relief,
}

impl Department {
    pub fn label(self) -> &'static str {
        match self {
            Department::Engineering => "Engineering",
            Department::Water => "Water",
            Department::Waste => "Waste",
            Department::Galley => "Galley",
            Department::Bakery => "Bakery",
            Department::Service => "Service",
            Department::Hydroponics => "Hydroponics",
            Department::Fishing => "Fishing",
            Department::Clinic => "Clinic",
            Department::Carpentry => "Carpentry",
            Department::Metalwork => "Metalwork",
            Department::Sewing => "Sewing",
            Department::Laundry => "Laundry",
            Department::Bridge => "Bridge",
            Department::Relief => "Relief",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Skill {
    Engineering,
    Cooking,
    Baking,
    Farming,
    Fishing,
    Medicine,
    Carpentry,
    Metalwork,
    Sewing,
    Seamanship,
    General,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Activity {
    Idle,
    Walking,
    Working,
    Cooking,
    Eating,
    Sleeping,
    Treating,
    OffWatch,
}

impl Activity {
    pub fn label(self) -> &'static str {
        match self {
            Activity::Idle => "idle",
            Activity::Walking => "walking",
            Activity::Working => "working",
            Activity::Cooking => "cooking",
            Activity::Eating => "eating",
            Activity::Sleeping => "sleeping",
            Activity::Treating => "treating",
            Activity::OffWatch => "off watch",
        }
    }
}
