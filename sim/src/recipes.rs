//! Fixed recipes. Mass in equals mass out. Seawater and losses are explicit.

use crate::kinds::{Good, PowerClass, RoomKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeId {
    BakeBread,
    CookMeal,
    Desalinate,
    SpinCloth,
    SewGarment,
    SawPlanks,
    RepairTools,
    RecycleMetal,
    ForgeFitting,
    CompostNutrients,
    WashClothes,
}

impl RecipeId {
    pub fn label(self) -> &'static str {
        match self {
            RecipeId::BakeBread => "Bake bread",
            RecipeId::CookMeal => "Cook meal",
            RecipeId::Desalinate => "Desalinate",
            RecipeId::SpinCloth => "Spin cloth",
            RecipeId::SewGarment => "Sew garment",
            RecipeId::SawPlanks => "Saw planks",
            RecipeId::RepairTools => "Repair tools",
            RecipeId::RecycleMetal => "Recycle metal",
            RecipeId::ForgeFitting => "Forge fitting",
            RecipeId::CompostNutrients => "Compost nutrients",
            RecipeId::WashClothes => "Wash clothes",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub id: RecipeId,
    pub station: RoomKind,
    pub power: PowerClass,
    /// Real seconds of staffed, powered work for one batch.
    pub duration_seconds: f64,
    /// Electrical energy for one batch, kilowatt-hours.
    pub energy_kwh: f64,
    pub inputs: &'static [(Good, f64)],
    pub outputs: &'static [(Good, f64)],
}

/// Drawn from the sea, not from a tank.
pub const ENVIRONMENTAL_SOURCE: &[Good] = &[Good::SeawaterL];
/// Discarded overboard or to the air. Not kept as ship stock.
pub const ENVIRONMENTAL_SINK: &[Good] = &[
    Good::BrineL,
    Good::WaterVaporKg,
    Good::WastewaterL,
    Good::ProcessLossKg,
];

pub fn is_source(good: Good) -> bool {
    ENVIRONMENTAL_SOURCE.contains(&good)
}

pub fn is_sink(good: Good) -> bool {
    ENVIRONMENTAL_SINK.contains(&good)
}

pub fn recipe(id: RecipeId) -> Recipe {
    match id {
        // 1.00 + 0.60 + 0.02 + 0.01 = 1.63 kg in, 1.40 bread + 0.23 vapor out.
        // About 3.5 hours and 2 kWh for a domestic-scale oven batch.
        RecipeId::BakeBread => Recipe {
            id,
            station: RoomKind::Bakery,
            power: PowerClass::Bakery,
            duration_seconds: 3.5 * 3600.0,
            energy_kwh: 2.0,
            inputs: &[
                (Good::FlourKg, 1.00),
                (Good::WaterL, 0.60),
                (Good::SaltKg, 0.02),
                (Good::YeastKg, 0.01),
            ],
            outputs: &[(Good::BreadKg, 1.40), (Good::WaterVaporKg, 0.23)],
        },
        RecipeId::CookMeal => Recipe {
            id,
            station: RoomKind::Galley,
            power: PowerClass::Galley,
            duration_seconds: 30.0 * 60.0,
            energy_kwh: 0.6,
            inputs: &[
                (Good::BreadKg, 0.25),
                (Good::VegetablesKg, 0.20),
                (Good::FishKg, 0.15),
                (Good::WaterL, 0.05),
            ],
            outputs: &[(Good::MealKg, 0.55), (Good::CompostKg, 0.10)],
        },
        // Reverse osmosis, 45% recovery, 3.5 kWh per cubic metre of fresh water.
        // 0.45 m³ fresh → 1.575 kWh. 1000 L seawater → 450 L fresh + 550 L brine.
        RecipeId::Desalinate => Recipe {
            id,
            station: RoomKind::Desalination,
            power: PowerClass::Desalination,
            duration_seconds: 30.0 * 60.0,
            energy_kwh: 1.575,
            inputs: &[(Good::SeawaterL, 1000.0)],
            outputs: &[(Good::WaterL, 450.0), (Good::BrineL, 550.0)],
        },
        RecipeId::SpinCloth => Recipe {
            id,
            station: RoomKind::Sewing,
            power: PowerClass::Essential,
            duration_seconds: 2.0 * 3600.0,
            energy_kwh: 0.4,
            inputs: &[(Good::FibreKg, 1.0)],
            outputs: &[(Good::ClothKg, 0.92), (Good::ScrapKg, 0.08)],
        },
        RecipeId::SewGarment => Recipe {
            id,
            station: RoomKind::Sewing,
            power: PowerClass::Essential,
            duration_seconds: 4.0 * 3600.0,
            energy_kwh: 0.2,
            inputs: &[(Good::ClothKg, 1.2)],
            outputs: &[(Good::GarmentKg, 1.05), (Good::ScrapKg, 0.15)],
        },
        RecipeId::SawPlanks => Recipe {
            id,
            station: RoomKind::Carpentry,
            power: PowerClass::Essential,
            duration_seconds: 3600.0,
            energy_kwh: 0.3,
            inputs: &[(Good::TimberKg, 2.0)],
            outputs: &[(Good::PlankKg, 1.7), (Good::CompostKg, 0.3)],
        },
        RecipeId::RepairTools => Recipe {
            id,
            station: RoomKind::Carpentry,
            power: PowerClass::Essential,
            duration_seconds: 2.0 * 3600.0,
            energy_kwh: 0.4,
            inputs: &[(Good::PlankKg, 0.3), (Good::MetalKg, 0.1)],
            outputs: &[(Good::ToolKg, 0.4)],
        },
        RecipeId::RecycleMetal => Recipe {
            id,
            station: RoomKind::Recycling,
            power: PowerClass::Essential,
            duration_seconds: 3.0 * 3600.0,
            energy_kwh: 1.5,
            inputs: &[(Good::ScrapKg, 1.0)],
            outputs: &[(Good::MetalKg, 0.90), (Good::ProcessLossKg, 0.10)],
        },
        RecipeId::ForgeFitting => Recipe {
            id,
            station: RoomKind::Metalwork,
            power: PowerClass::Essential,
            duration_seconds: 2.0 * 3600.0,
            energy_kwh: 0.8,
            inputs: &[(Good::MetalKg, 0.5)],
            outputs: &[(Good::ToolKg, 0.45), (Good::ScrapKg, 0.05)],
        },
        RecipeId::CompostNutrients => Recipe {
            id,
            station: RoomKind::WasteTreatment,
            power: PowerClass::Essential,
            duration_seconds: 2.0 * 3600.0,
            energy_kwh: 0.1,
            inputs: &[(Good::CompostKg, 1.0)],
            outputs: &[(Good::NutrientKg, 0.55), (Good::ProcessLossKg, 0.45)],
        },
        RecipeId::WashClothes => Recipe {
            id,
            station: RoomKind::Laundry,
            power: PowerClass::Essential,
            duration_seconds: 3600.0,
            energy_kwh: 0.3,
            inputs: &[(Good::WaterL, 8.0), (Good::GarmentKg, 1.0)],
            outputs: &[(Good::GarmentKg, 1.0), (Good::WastewaterL, 8.0)],
        },
    }
}

#[cfg(test)]
fn mass_balanced(id: RecipeId) -> bool {
    let recipe = recipe(id);
    let input: f64 = recipe.inputs.iter().map(|(_, qty)| qty).sum();
    let output: f64 = recipe.outputs.iter().map(|(_, qty)| qty).sum();
    (input - output).abs() < 1e-9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_recipe_conserves_mass() {
        for id in [
            RecipeId::BakeBread,
            RecipeId::CookMeal,
            RecipeId::Desalinate,
            RecipeId::SpinCloth,
            RecipeId::SewGarment,
            RecipeId::SawPlanks,
            RecipeId::RepairTools,
            RecipeId::RecycleMetal,
            RecipeId::ForgeFitting,
            RecipeId::CompostNutrients,
            RecipeId::WashClothes,
        ] {
            assert!(mass_balanced(id), "{id:?} does not balance");
        }
    }

    #[test]
    fn bread_and_desalination_match_published_figures() {
        let bread = recipe(RecipeId::BakeBread);
        assert!((bread.duration_seconds - 3.5 * 3600.0).abs() < 1e-9);
        let flour = bread
            .inputs
            .iter()
            .find(|(good, _)| *good == Good::FlourKg)
            .unwrap()
            .1;
        let loaf = bread
            .outputs
            .iter()
            .find(|(good, _)| *good == Good::BreadKg)
            .unwrap()
            .1;
        assert!((flour - 1.0).abs() < 1e-9);
        assert!((loaf - 1.4).abs() < 1e-9);

        let desal = recipe(RecipeId::Desalinate);
        let fresh = desal
            .outputs
            .iter()
            .find(|(good, _)| *good == Good::WaterL)
            .unwrap()
            .1;
        let fresh_m3 = fresh / 1000.0;
        let kwh_per_m3 = desal.energy_kwh / fresh_m3;
        assert!(
            (kwh_per_m3 - 3.5).abs() < 1e-9,
            "desalination was {kwh_per_m3} kWh/m³"
        );
    }
}
