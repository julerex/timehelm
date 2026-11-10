//! World state, the one-minute economic step, and catch-up.
//!
//! Need and life transitions are logged from the step. Those lines are not
//! read back into the world.

use serde::{Deserialize, Serialize};

use crate::geom::{polygon_centroid, shared_edge_span, EdgeSpan};
use crate::kinds::{Activity, Department, Good, PowerClass, RoomKind, Skill};
use crate::layout::build_ship;
use crate::proto::format_unix_utc;
use crate::recipes::{is_sink, is_source, recipe, RecipeId};

pub const STEP_SECONDS: i64 = 60;
pub const WALK_METRES_PER_SECOND: f32 = 1.3;
const WALK_PER_STEP: f32 = WALK_METRES_PER_SECOND * STEP_SECONDS as f32;
/// Nameplate average of the ship's solar plant.
pub const SOLAR_KW: f64 = 90.0;
const CLINIC_BASE_KWH: f64 = 0.03;
/// About 48 W/m² of grow lighting, expressed per minute.
const HYDRO_KWH_PER_M2: f64 = 0.0008;
const HYDRO_CYCLE_DAYS: f64 = 28.0;
const HYDRO_YIELD_KG_PER_M2: f64 = 0.12;
const HYDRO_NUTRIENT_PER_M2: f64 = 0.01;
const FISH_KG_PER_FISHER_DAY: f64 = 8.0;
const MEAL_PORTION_KG: f64 = 0.45;
const HUNGER_RELIEF: f64 = 0.4;
const DRINK_LITRES: f64 = 0.12;
const THIRST_RELIEF: f64 = 0.3;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct World {
    pub seed: u64,
    pub last_tick_unix: i64,
    pub solar_kw: f64,
    pub decks: Vec<Deck>,
    pub rooms: Vec<Room>,
    pub doors: Vec<Door>,
    pub stations: Vec<Station>,
    pub crew: Vec<CrewMember>,
    pub stocks: Vec<Stock>,
    pub jobs: Vec<Job>,
    pub crops: Vec<Crop>,
    pub next_job_id: u32,
    pub power: PowerReport,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Deck {
    pub index: u8,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Room {
    pub id: u32,
    pub deck: u8,
    pub kind: RoomKind,
    pub name: String,
    pub polygon: Vec<[f32; 2]>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Door {
    pub id: u32,
    pub from_room: u32,
    pub to_room: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Station {
    pub id: u32,
    pub room_id: u32,
    pub recipe: Option<RecipeId>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CrewMember {
    pub id: u32,
    pub name: String,
    pub department: Department,
    pub skill: Skill,
    pub room_id: u32,
    pub x: f32,
    pub y: f32,
    pub deck: u8,
    pub hunger: f64,
    pub thirst: f64,
    pub fatigue: f64,
    pub health: f64,
    pub activity: Activity,
    pub berth_id: u32,
    pub station_id: Option<u32>,
    pub watch_offset_min: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stock {
    pub room_id: u32,
    pub good: Good,
    pub qty: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Job {
    pub id: u32,
    pub station_id: u32,
    pub recipe: RecipeId,
    pub elapsed_seconds: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Crop {
    pub room_id: u32,
    pub progress_days: f64,
    pub area_m2: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PowerReport {
    pub supply_kwh: f64,
    pub demand_kwh: f64,
    pub clinic_demanded: f64,
    pub clinic_served: f64,
    pub galley_demanded: f64,
    pub galley_served: f64,
    pub hydro_demanded: f64,
    pub hydro_served: f64,
    pub desal_demanded: f64,
    pub desal_served: f64,
}

impl Default for PowerReport {
    fn default() -> Self {
        Self {
            supply_kwh: 0.0,
            demand_kwh: 0.0,
            clinic_demanded: 0.0,
            clinic_served: 0.0,
            galley_demanded: 0.0,
            galley_served: 0.0,
            hydro_demanded: 0.0,
            hydro_served: 0.0,
            desal_demanded: 0.0,
            desal_served: 0.0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MemoryStore {
    world: Option<World>,
}

impl MemoryStore {
    pub fn save(&mut self, world: &World) {
        self.world = Some(world.clone());
    }

    pub fn load(&self) -> Option<World> {
        self.world.clone()
    }
}

/// Greedy allocation. Earlier [`PowerClass`] values are served first.
pub fn allocate_power(supply_kwh: f64, demands: &[(PowerClass, f64)]) -> Vec<f64> {
    let mut order: Vec<usize> = (0..demands.len()).collect();
    order.sort_by(|&left, &right| {
        demands[left]
            .0
            .cmp(&demands[right].0)
            .then(left.cmp(&right))
    });
    let mut remaining = supply_kwh.max(0.0);
    let mut served = vec![0.0; demands.len()];
    for index in order {
        let ask = demands[index].1.max(0.0);
        let give = ask.min(remaining);
        served[index] = give;
        remaining -= give;
    }
    served
}

impl World {
    pub fn generate(seed: u64, last_tick_unix: i64) -> Result<Self, String> {
        let built = build_ship()?;
        let mut world = Self {
            seed,
            last_tick_unix,
            solar_kw: SOLAR_KW,
            decks: built.decks,
            rooms: built.rooms,
            doors: built.doors,
            stations: built.stations,
            crew: Vec::new(),
            stocks: Vec::new(),
            jobs: Vec::new(),
            crops: Vec::new(),
            next_job_id: 1,
            power: PowerReport::default(),
        };
        world.crops = world
            .rooms
            .iter()
            .filter(|room| room.kind == RoomKind::Hydroponics)
            .map(|room| Crop {
                room_id: room.id,
                progress_days: 0.0,
                area_m2: f64::from(crate::geom::polygon_area(&room.polygon)),
            })
            .collect();
        world.crew = muster(&world);
        world.endow();
        Ok(world)
    }

    pub fn supply_kwh(&self) -> f64 {
        self.solar_kw * (STEP_SECONDS as f64) / 3600.0
    }

    /// Advance every missed minute up to `now_unix`. A partial minute waits.
    pub fn catch_up_to(&mut self, now_unix: i64) {
        if now_unix <= self.last_tick_unix {
            return;
        }
        let steps = (now_unix - self.last_tick_unix) / STEP_SECONDS;
        for _ in 0..steps {
            self.step();
        }
    }

    pub fn step(&mut self) {
        self.last_tick_unix += STEP_SECONDS;
        let minute = minute_of_day(self.last_tick_unix);
        self.tick_needs(minute);
        self.tick_clinic();
        let hydro_fraction = self.tick_power_and_jobs(minute);
        self.tick_hydro(minute, hydro_fraction);
        self.tick_fishing(minute);
        self.tick_wear(minute);
        self.tick_movement(minute);
    }

    pub fn total(&self, good: Good) -> f64 {
        self.stocks
            .iter()
            .filter(|stock| stock.good == good)
            .map(|stock| stock.qty)
            .sum()
    }

    pub fn first_room(&self, kind: RoomKind) -> Option<u32> {
        self.rooms
            .iter()
            .find(|room| room.kind == kind)
            .map(|room| room.id)
    }

    fn endow(&mut self) {
        self.deposit(RoomKind::DryStore, Good::FlourKg, 2000.0);
        self.deposit(RoomKind::DryStore, Good::SaltKg, 50.0);
        self.deposit(RoomKind::DryStore, Good::YeastKg, 20.0);
        self.deposit(RoomKind::DryStore, Good::FibreKg, 250.0);
        self.deposit(RoomKind::Bakery, Good::BreadKg, 120.0);
        self.deposit(RoomKind::ColdStore, Good::VegetablesKg, 2000.0);
        self.deposit(RoomKind::ColdStore, Good::FishKg, 600.0);
        self.deposit(RoomKind::Mess, Good::MealKg, 80.0);
        self.deposit(RoomKind::WaterStore, Good::WaterL, 30_000.0);
        self.deposit(RoomKind::TimberStore, Good::TimberKg, 2500.0);
        self.deposit(RoomKind::Carpentry, Good::PlankKg, 300.0);
        self.deposit(RoomKind::Carpentry, Good::ToolKg, 40.0);
        self.deposit(RoomKind::MetalStore, Good::MetalKg, 900.0);
        self.deposit(RoomKind::MetalStore, Good::ScrapKg, 250.0);
        self.deposit(RoomKind::Sewing, Good::ClothKg, 100.0);
        self.deposit(RoomKind::Sewing, Good::GarmentKg, 400.0);
        self.deposit(RoomKind::Clinic, Good::MedicalKg, 25.0);
        self.deposit(RoomKind::Hydroponics, Good::NutrientKg, 400.0);
        self.deposit(RoomKind::WasteTreatment, Good::CompostKg, 150.0);
    }

    fn deposit(&mut self, kind: RoomKind, good: Good, qty: f64) {
        let Some(room_id) = self.first_room(kind) else {
            return;
        };
        self.add_stock(room_id, good, qty);
    }

    fn add_stock(&mut self, room_id: u32, good: Good, qty: f64) {
        if qty <= 0.0 || is_sink(good) {
            return;
        }
        if let Some(stock) = self
            .stocks
            .iter_mut()
            .find(|stock| stock.room_id == room_id && stock.good == good)
        {
            stock.qty += qty;
            return;
        }
        self.stocks.push(Stock { room_id, good, qty });
    }

    fn take_good(&mut self, good: Good, qty: f64) -> bool {
        let rooms: Vec<u32> = self.rooms.iter().map(|room| room.id).collect();
        self.take_from_rooms(&rooms, good, qty)
    }

    fn take_from_rooms(&mut self, rooms: &[u32], good: Good, qty: f64) -> bool {
        if is_source(good) || qty <= 0.0 {
            return true;
        }
        let available: f64 = self
            .stocks
            .iter()
            .filter(|stock| stock.good == good && rooms.contains(&stock.room_id))
            .map(|stock| stock.qty)
            .sum();
        if available + 1e-9 < qty {
            return false;
        }
        let mut left = qty;
        for stock in &mut self.stocks {
            if stock.good != good || !rooms.contains(&stock.room_id) || left <= 0.0 {
                continue;
            }
            let delta = stock.qty.min(left);
            stock.qty -= delta;
            left -= delta;
        }
        true
    }

    fn can_afford(&self, inputs: &[(Good, f64)]) -> bool {
        inputs
            .iter()
            .all(|(good, qty)| is_source(*good) || self.total(*good) + 1e-9 >= *qty)
    }

    fn consume_inputs(&mut self, inputs: &[(Good, f64)]) {
        for (good, qty) in inputs {
            if is_source(*good) {
                continue;
            }
            let _ = self.take_good(*good, *qty);
        }
    }

    fn tick_needs(&mut self, minute: u32) {
        let crew_count = self.crew.len();
        let mut notes = Vec::new();
        for (index, crew) in self.crew.iter_mut().enumerate() {
            if !is_alive(crew) {
                continue;
            }
            let hunger_before = crew.hunger;
            let thirst_before = crew.thirst;
            let health_before = crew.health;
            let activity_before = crew.activity;
            crew.hunger = (crew.hunger + 1.0 / 1440.0).min(1.5);
            crew.thirst = (crew.thirst + 1.0 / 1440.0).min(1.5);
            if crew.hunger > 0.85 {
                crew.health = (crew.health - 0.02 / 1440.0).max(0.0);
            }
            if crew.thirst > 0.85 {
                crew.health = (crew.health - 0.03 / 1440.0).max(0.0);
            }
            if on_shift(crew.watch_offset_min, minute) {
                crew.fatigue = (crew.fatigue + 1.0 / 480.0).min(1.0);
                if crew.activity != Activity::Eating && crew.activity != Activity::Treating {
                    crew.activity = Activity::Working;
                }
            } else if crew.room_id == crew.berth_id {
                crew.fatigue = (crew.fatigue - 1.0 / 480.0).max(0.0);
                crew.activity = Activity::Sleeping;
            } else {
                crew.activity = Activity::OffWatch;
            }
            if hunger_before <= 0.85 && crew.hunger > 0.85 {
                notes.push((
                    index,
                    LifeEvent::Starving,
                    "hunger rose above 0.85; health falls while it stays there".to_string(),
                ));
            }
            if thirst_before <= 0.85 && crew.thirst > 0.85 {
                notes.push((
                    index,
                    LifeEvent::Dehydrated,
                    "thirst rose above 0.85; health falls while it stays there".to_string(),
                ));
            }
            if health_before >= 0.55 && crew.health < 0.55 {
                let (prose, token) =
                    harm_cause(crew.hunger, crew.thirst).unwrap_or(("no hunger or thirst", "none"));
                notes.push((
                    index,
                    LifeEvent::Critical,
                    format!("health fell below 0.55 from {prose}; cause=\"{token}\""),
                ));
            }
            if health_before > 0.0 && crew.health <= 0.0 {
                let detail = match harm_cause(crew.hunger, crew.thirst) {
                    Some((prose, token)) => format!(
                        "died from {prose}; health reached 0; crew member stays aboard; cause=\"{token}\""
                    ),
                    None => "health reached 0 with no hunger or thirst damage; crew member stays aboard; cause=\"none\"".to_string(),
                };
                notes.push((index, LifeEvent::Died, detail));
            }
            if activity_before != Activity::Sleeping && crew.activity == Activity::Sleeping {
                notes.push((
                    index,
                    LifeEvent::Asleep,
                    format!("fell asleep in berth; was {}", activity_before.label()),
                ));
            }
            if activity_before == Activity::Sleeping && crew.activity != Activity::Sleeping {
                notes.push((
                    index,
                    LifeEvent::Awoke,
                    format!("woke; now {}", crew.activity.label()),
                ));
            }
        }
        for (index, event, detail) in notes {
            self.log_life(index, event, &detail);
        }
        let dining = self.dining_rooms();
        let water_rooms = self.rooms_of(RoomKind::WaterStore);
        let meals_before = self.total(Good::MealKg);
        let water_before = self.total(Good::WaterL);
        let mut missed_meals = 0u32;
        for index in 0..crew_count {
            if !is_alive(&self.crew[index]) {
                continue;
            }
            let room_id = self.crew[index].room_id;
            let in_dining = dining.contains(&room_id);
            if self.crew[index].hunger >= 0.5 && in_dining {
                if self.take_from_rooms(&dining, Good::MealKg, MEAL_PORTION_KG) {
                    let hunger_before = self.crew[index].hunger;
                    self.crew[index].hunger = (hunger_before - HUNGER_RELIEF).max(0.0);
                    self.crew[index].activity = Activity::Eating;
                    let hunger_after = self.crew[index].hunger;
                    self.log_life(
                        index,
                        LifeEvent::Ate,
                        &format!(
                            "ate {MEAL_PORTION_KG:.2} kg of meals; hunger {hunger_before:.3} -> {hunger_after:.3}"
                        ),
                    );
                    if hunger_before > 0.85 && hunger_after <= 0.85 {
                        self.log_life(
                            index,
                            LifeEvent::HungerEased,
                            "hunger fell to 0.85 or below after eating",
                        );
                    }
                } else {
                    missed_meals += 1;
                }
            }
            if self.crew[index].thirst >= 0.35
                && self.take_from_rooms(&water_rooms, Good::WaterL, DRINK_LITRES)
            {
                let thirst_before = self.crew[index].thirst;
                self.crew[index].thirst = (thirst_before - THIRST_RELIEF).max(0.0);
                let thirst_after = self.crew[index].thirst;
                self.log_life(
                    index,
                    LifeEvent::Drank,
                    &format!(
                        "drank {DRINK_LITRES:.2} L of water; thirst {thirst_before:.3} -> {thirst_after:.3}"
                    ),
                );
                if thirst_before > 0.85 && thirst_after <= 0.85 {
                    self.log_life(
                        index,
                        LifeEvent::ThirstEased,
                        "thirst fell to 0.85 or below after drinking",
                    );
                }
            }
        }
        let meals_after = self.total(Good::MealKg);
        if meals_before + 1e-9 >= MEAL_PORTION_KG && meals_after + 1e-9 < MEAL_PORTION_KG {
            self.log_ship(
                LifeEvent::MealsExhausted,
                &format!(
                    "meal stock fell below one {MEAL_PORTION_KG:.2} kg portion; {missed_meals} crew missed a meal"
                ),
            );
        }
        let water_after = self.total(Good::WaterL);
        if water_before + 1e-9 >= DRINK_LITRES && water_after + 1e-9 < DRINK_LITRES {
            self.log_ship(
                LifeEvent::WaterExhausted,
                &format!("fresh water fell below one {DRINK_LITRES:.2} L drink"),
            );
        }
    }

    fn tick_clinic(&mut self) {
        let medical_before = self.total(Good::MedicalKg);
        let crew_count = self.crew.len();
        for index in 0..crew_count {
            let room_id = self.crew[index].room_id;
            let in_clinic = self
                .rooms
                .get(room_id as usize)
                .is_some_and(|room| room.kind == RoomKind::Clinic);
            let alive = is_alive(&self.crew[index]);
            let health_before = self.crew[index].health;
            if alive && in_clinic && health_before < 0.9 && self.take_good(Good::MedicalKg, 0.02) {
                self.crew[index].health = (health_before + 0.05).min(1.0);
                self.crew[index].activity = Activity::Treating;
                self.log_life(
                    index,
                    LifeEvent::Treated,
                    &format!("treatment raised health from {health_before:.3}"),
                );
            }
        }
        let medical_after = self.total(Good::MedicalKg);
        if medical_before + 1e-9 >= 0.02 && medical_after + 1e-9 < 0.02 {
            self.log_ship(
                LifeEvent::MedicalExhausted,
                "medical supplies fell below one 0.02 kg treatment",
            );
        }
    }

    /// Returns the hydroponic power fraction for this minute (1 when there is no load).
    fn tick_power_and_jobs(&mut self, minute: u32) -> f64 {
        let mut demands: Vec<(PowerClass, f64)> = Vec::new();
        let mut tags: Vec<DemandTag> = Vec::new();

        demands.push((PowerClass::Clinic, CLINIC_BASE_KWH));
        tags.push(DemandTag::ClinicBase);

        let hydro_kwh: f64 = self
            .crops
            .iter()
            .map(|crop| crop.area_m2 * HYDRO_KWH_PER_M2)
            .sum();
        if hydro_kwh > 0.0 {
            demands.push((PowerClass::Hydroponics, hydro_kwh));
            tags.push(DemandTag::Hydro);
        }

        let station_count = self.stations.len();
        for index in 0..station_count {
            let Some(recipe_id) = self.stations[index].recipe else {
                continue;
            };
            let staffed = self.station_staffed(self.stations[index].id, minute);
            let running = self
                .jobs
                .iter()
                .any(|job| job.station_id == self.stations[index].id);
            let spec = recipe(recipe_id);
            if !running && (!staffed || !self.can_afford(spec.inputs)) {
                continue;
            }
            let per_step = spec.energy_kwh * (STEP_SECONDS as f64) / spec.duration_seconds;
            demands.push((spec.power, per_step));
            tags.push(DemandTag::Station(index));
        }

        let supply = self.supply_kwh();
        let served = allocate_power(supply, &demands);
        let demand_kwh: f64 = demands.iter().map(|(_, kwh)| *kwh).sum();
        self.power = PowerReport {
            supply_kwh: supply,
            demand_kwh,
            clinic_demanded: class_sum(&demands, PowerClass::Clinic),
            clinic_served: class_served(&demands, &served, PowerClass::Clinic),
            galley_demanded: class_sum(&demands, PowerClass::Galley),
            galley_served: class_served(&demands, &served, PowerClass::Galley),
            hydro_demanded: class_sum(&demands, PowerClass::Hydroponics),
            hydro_served: class_served(&demands, &served, PowerClass::Hydroponics),
            desal_demanded: class_sum(&demands, PowerClass::Desalination),
            desal_served: class_served(&demands, &served, PowerClass::Desalination),
        };

        let mut hydro_fraction = 1.0;
        for (index, tag) in tags.iter().enumerate() {
            let fraction = if demands[index].1 <= 1e-12 {
                1.0
            } else {
                served[index] / demands[index].1
            };
            match tag {
                DemandTag::ClinicBase => {}
                DemandTag::Hydro => hydro_fraction = fraction,
                DemandTag::Station(station_index) => {
                    self.advance_station(*station_index, minute, fraction);
                }
            }
        }
        hydro_fraction
    }

    fn advance_station(&mut self, station_index: usize, minute: u32, fraction: f64) {
        if fraction <= 0.0 {
            return;
        }
        let station_id = self.stations[station_index].id;
        let Some(recipe_id) = self.stations[station_index].recipe else {
            return;
        };
        let spec = recipe(recipe_id);
        let existing = self
            .jobs
            .iter()
            .position(|job| job.station_id == station_id);
        if let Some(job_index) = existing {
            self.jobs[job_index].elapsed_seconds += (STEP_SECONDS as f64) * fraction;
            if self.jobs[job_index].elapsed_seconds + 1e-9 >= spec.duration_seconds {
                let room_id = self.stations[station_index].room_id;
                self.finish_recipe(room_id, recipe_id);
                self.jobs.remove(job_index);
            }
            return;
        }
        if !self.station_staffed(station_id, minute) || !self.can_afford(spec.inputs) {
            return;
        }
        self.consume_inputs(spec.inputs);
        let elapsed = (STEP_SECONDS as f64) * fraction;
        if elapsed + 1e-9 >= spec.duration_seconds {
            let room_id = self.stations[station_index].room_id;
            self.finish_recipe(room_id, recipe_id);
            return;
        }
        let id = self.next_job_id;
        self.next_job_id += 1;
        self.jobs.push(Job {
            id,
            station_id,
            recipe: recipe_id,
            elapsed_seconds: elapsed,
        });
    }

    fn finish_recipe(&mut self, room_id: u32, recipe_id: RecipeId) {
        for (good, qty) in recipe(recipe_id).outputs {
            self.add_stock(room_id, *good, *qty);
        }
    }

    fn tick_hydro(&mut self, minute: u32, fraction: f64) {
        let crop_count = self.crops.len();
        for index in 0..crop_count {
            let room_id = self.crops[index].room_id;
            let staff = if self.room_staffed(room_id, minute) {
                1.0
            } else {
                0.0
            };
            self.crops[index].progress_days += (1.0 / 1440.0) * fraction * staff;
            if self.crops[index].progress_days + 1e-12 < HYDRO_CYCLE_DAYS {
                continue;
            }
            let area = self.crops[index].area_m2;
            let nutrients = area * HYDRO_NUTRIENT_PER_M2;
            if !self.take_good(Good::NutrientKg, nutrients) {
                self.crops[index].progress_days = HYDRO_CYCLE_DAYS;
                continue;
            }
            self.add_stock(room_id, Good::VegetablesKg, area * HYDRO_YIELD_KG_PER_M2);
            self.add_stock(
                room_id,
                Good::CompostKg,
                area * HYDRO_YIELD_KG_PER_M2 * 0.25,
            );
            self.crops[index].progress_days = 0.0;
        }
    }

    fn tick_fishing(&mut self, minute: u32) {
        let mut present = 0.0;
        for crew in &self.crew {
            let Some(station_id) = crew.station_id else {
                continue;
            };
            let Some(station) = self.stations.get(station_id as usize) else {
                continue;
            };
            let Some(room) = self.rooms.get(station.room_id as usize) else {
                continue;
            };
            if is_alive(crew)
                && room.kind == RoomKind::Fishing
                && crew.room_id == room.id
                && on_shift(crew.watch_offset_min, minute)
            {
                present += 1.0;
            }
        }
        if present <= 0.0 {
            return;
        }
        let kg = present * FISH_KG_PER_FISHER_DAY / 1440.0;
        if let Some(room_id) = self.first_room(RoomKind::ColdStore) {
            self.add_stock(room_id, Good::FishKg, kg);
        }
    }

    fn tick_wear(&mut self, minute: u32) {
        if minute != 0 {
            return;
        }
        let worn = self.total(Good::GarmentKg) * 0.002;
        if worn <= 0.0 {
            return;
        }
        if self.take_good(Good::GarmentKg, worn) {
            if let Some(room_id) = self.first_room(RoomKind::Sewing) {
                self.add_stock(room_id, Good::ScrapKg, worn);
            }
        }
    }

    fn tick_movement(&mut self, minute: u32) {
        let adj = adjacency(&self.doors, self.rooms.len());
        let goals: Vec<u32> = self
            .crew
            .iter()
            .map(|crew| self.goal_room(crew, minute))
            .collect();
        for (index, goal) in goals.into_iter().enumerate() {
            if !is_alive(&self.crew[index]) {
                continue;
            }
            self.move_crew(index, goal, &adj);
        }
    }

    fn goal_room(&self, crew: &CrewMember, minute: u32) -> u32 {
        if crew.health < 0.55 {
            if let Some(room_id) = self.first_room(RoomKind::Clinic) {
                return room_id;
            }
        }
        if on_shift(crew.watch_offset_min, minute) {
            if let Some(station_id) = crew.station_id {
                if let Some(station) = self.stations.get(station_id as usize) {
                    return station.room_id;
                }
            }
        }
        if crew.hunger > 0.6 {
            if let Some(room_id) = self.first_room(RoomKind::Mess) {
                return room_id;
            }
        }
        crew.berth_id
    }

    fn move_crew(&mut self, index: usize, goal: u32, adj: &[Vec<u32>]) {
        let current = self.crew[index].room_id;
        if current == goal {
            return;
        }
        let Some(next) = next_hop(adj, current, goal) else {
            return;
        };
        let (target_deck, target_id, tx, ty, doorway) = {
            let Some(target) = self.rooms.get(next as usize) else {
                return;
            };
            let Some(from_room) = self.rooms.get(current as usize) else {
                return;
            };
            let (tx, ty) = polygon_centroid(&target.polygon);
            let doorway = if from_room.deck == target.deck {
                shared_edge_span(&from_room.polygon, &target.polygon).map(EdgeSpan::midpoint)
            } else {
                None
            };
            (target.deck, target.id, tx, ty, doorway)
        };
        let mut x = self.crew[index].x;
        let mut y = self.crew[index].y;
        let entered = walk_through_door(&mut x, &mut y, doorway, tx, ty);
        if entered {
            self.crew[index].room_id = target_id;
            self.crew[index].deck = target_deck;
        }
        self.crew[index].x = x;
        self.crew[index].y = y;
        if self.crew[index].activity != Activity::Eating
            && self.crew[index].activity != Activity::Treating
        {
            self.crew[index].activity = Activity::Walking;
        }
    }

    fn station_staffed(&self, station_id: u32, minute: u32) -> bool {
        let Some(station) = self.stations.get(station_id as usize) else {
            return false;
        };
        self.crew.iter().any(|crew| {
            is_alive(crew)
                && crew.station_id == Some(station_id)
                && crew.room_id == station.room_id
                && on_shift(crew.watch_offset_min, minute)
        })
    }

    fn room_staffed(&self, room_id: u32, minute: u32) -> bool {
        self.crew.iter().any(|crew| {
            is_alive(crew)
                && crew.room_id == room_id
                && crew.station_id.is_some_and(|station_id| {
                    self.stations
                        .get(station_id as usize)
                        .is_some_and(|station| station.room_id == room_id)
                })
                && on_shift(crew.watch_offset_min, minute)
        })
    }

    fn dining_rooms(&self) -> Vec<u32> {
        self.rooms
            .iter()
            .filter(|room| {
                matches!(
                    room.kind,
                    RoomKind::Mess | RoomKind::Restaurant | RoomKind::Galley
                )
            })
            .map(|room| room.id)
            .collect()
    }

    fn rooms_of(&self, kind: RoomKind) -> Vec<u32> {
        self.rooms
            .iter()
            .filter(|room| room.kind == kind)
            .map(|room| room.id)
            .collect()
    }

    fn log_life(&self, index: usize, event: LifeEvent, detail: &str) {
        let meals_kg = self.total(Good::MealKg);
        let water_l = self.total(Good::WaterL);
        let medical_kg = self.total(Good::MedicalKg);
        let sim_unix = self.last_tick_unix;
        let sim_utc = format_unix_utc(sim_unix);
        let Some((crew_id, name, room_id, deck, hunger, thirst, fatigue, health, activity)) =
            self.crew.get(index).map(|crew| {
                (
                    crew.id,
                    crew.name.clone(),
                    crew.room_id,
                    crew.deck,
                    crew.hunger,
                    crew.thirst,
                    crew.fatigue,
                    crew.health,
                    crew.activity.label(),
                )
            })
        else {
            return;
        };
        let room = match self.rooms.get(room_id as usize) {
            Some(room) => format!("{} ({})", room.name, room.kind.label()),
            None => format!("room {room_id}"),
        };
        let token = event.token();
        let line = format!(
            "{token}: {detail}; crew_id={crew_id} name=\"{name}\" sim_unix={sim_unix} sim_utc=\"{sim_utc}\" room_id={room_id} room=\"{room}\" deck={deck} hunger={hunger:.4} thirst={thirst:.4} fatigue={fatigue:.4} health={health:.4} activity=\"{activity}\" meals_kg={meals_kg:.4} water_l={water_l:.4} medical_kg={medical_kg:.4}"
        );
        emit_life(event, &line);
    }

    fn log_ship(&self, event: LifeEvent, detail: &str) {
        let token = event.token();
        let sim_unix = self.last_tick_unix;
        let sim_utc = format_unix_utc(sim_unix);
        let meals_kg = self.total(Good::MealKg);
        let water_l = self.total(Good::WaterL);
        let medical_kg = self.total(Good::MedicalKg);
        let line = format!(
            "{token}: {detail}; sim_unix={sim_unix} sim_utc=\"{sim_utc}\" meals_kg={meals_kg:.4} water_l={water_l:.4} medical_kg={medical_kg:.4}"
        );
        emit_life(event, &line);
    }
}

/// Need and life transitions from one minute of [`World::step`].
///
/// Logging does not change the step. Health reaching 0 leaves the person aboard.
#[derive(Clone, Copy)]
enum LifeEvent {
    Ate,
    Drank,
    Starving,
    Dehydrated,
    HungerEased,
    ThirstEased,
    Critical,
    Died,
    Treated,
    Asleep,
    Awoke,
    MealsExhausted,
    WaterExhausted,
    MedicalExhausted,
}

impl LifeEvent {
    fn token(self) -> &'static str {
        match self {
            LifeEvent::Ate => "ate",
            LifeEvent::Drank => "drank",
            LifeEvent::Starving => "starving",
            LifeEvent::Dehydrated => "dehydrated",
            LifeEvent::HungerEased => "hunger_eased",
            LifeEvent::ThirstEased => "thirst_eased",
            LifeEvent::Critical => "critical",
            LifeEvent::Died => "died",
            LifeEvent::Treated => "treated",
            LifeEvent::Asleep => "asleep",
            LifeEvent::Awoke => "awoke",
            LifeEvent::MealsExhausted => "meals_exhausted",
            LifeEvent::WaterExhausted => "water_exhausted",
            LifeEvent::MedicalExhausted => "medical_exhausted",
        }
    }

    fn warns(self) -> bool {
        matches!(
            self,
            LifeEvent::Starving
                | LifeEvent::Dehydrated
                | LifeEvent::Critical
                | LifeEvent::Died
                | LifeEvent::MealsExhausted
                | LifeEvent::WaterExhausted
                | LifeEvent::MedicalExhausted
        )
    }
}

fn harm_cause(hunger: f64, thirst: f64) -> Option<(&'static str, &'static str)> {
    match (hunger > 0.85, thirst > 0.85) {
        (true, false) => Some(("hunger", "hunger")),
        (false, true) => Some(("thirst", "thirst")),
        (true, true) => Some(("hunger and thirst", "hunger_and_thirst")),
        (false, false) => None,
    }
}

fn emit_life(event: LifeEvent, line: &str) {
    if event.warns() {
        tracing::warn!(target: "timehelm::life", "{line}");
    } else {
        tracing::info!(target: "timehelm::life", "{line}");
    }
}

enum DemandTag {
    ClinicBase,
    Hydro,
    Station(usize),
}

fn class_sum(demands: &[(PowerClass, f64)], class: PowerClass) -> f64 {
    demands
        .iter()
        .filter(|(kind, _)| *kind == class)
        .map(|(_, kwh)| *kwh)
        .sum()
}

fn class_served(demands: &[(PowerClass, f64)], served: &[f64], class: PowerClass) -> f64 {
    demands
        .iter()
        .enumerate()
        .filter(|(_, (kind, _))| *kind == class)
        .map(|(index, _)| served[index])
        .sum()
}

fn minute_of_day(unix: i64) -> u32 {
    u32::try_from(unix.div_euclid(60).rem_euclid(1440)).unwrap_or(0)
}

fn is_alive(crew: &CrewMember) -> bool {
    crew.health > 0.0
}

pub fn on_shift(offset_min: u32, minute_of_day: u32) -> bool {
    let local = (minute_of_day + offset_min) % 1440;
    local < 8 * 60
}

/// Walk toward the doorway, then into the next room with whatever distance is left.
/// A stair has no shared edge, so that hop still aims at the room centre.
fn walk_through_door(
    x: &mut f32,
    y: &mut f32,
    doorway: Option<[f32; 2]>,
    tx: f32,
    ty: f32,
) -> bool {
    let mut budget = WALK_PER_STEP;
    if let Some(door) = doorway {
        budget -= walk_toward(x, y, door[0], door[1], budget);
        let at_door = (*x - door[0]).hypot(*y - door[1]) <= 0.08;
        if !at_door {
            return false;
        }
        let _ = walk_toward(x, y, tx, ty, budget);
        return true;
    }
    let _ = walk_toward(x, y, tx, ty, budget);
    (*x - tx).hypot(*y - ty) <= 0.08
}

fn walk_toward(x: &mut f32, y: &mut f32, tx: f32, ty: f32, budget: f32) -> f32 {
    if budget <= 0.0 {
        return 0.0;
    }
    let dx = tx - *x;
    let dy = ty - *y;
    let distance = (dx * dx + dy * dy).sqrt();
    if distance <= budget + 0.05 {
        *x = tx;
        *y = ty;
        distance
    } else if distance > f32::EPSILON {
        let scale = budget / distance;
        *x += dx * scale;
        *y += dy * scale;
        budget
    } else {
        0.0
    }
}

fn adjacency(doors: &[Door], room_count: usize) -> Vec<Vec<u32>> {
    let mut adj = vec![Vec::new(); room_count];
    for door in doors {
        let from = door.from_room as usize;
        let to = door.to_room as usize;
        if from < room_count && to < room_count {
            adj[from].push(door.to_room);
            adj[to].push(door.from_room);
        }
    }
    for neighbors in &mut adj {
        neighbors.sort_unstable();
        neighbors.dedup();
    }
    adj
}

fn next_hop(adj: &[Vec<u32>], from: u32, to: u32) -> Option<u32> {
    if from == to || adj.is_empty() {
        return None;
    }
    let mut previous = vec![None; adj.len()];
    let mut queue = std::collections::VecDeque::new();
    let start = from as usize;
    if start >= adj.len() || (to as usize) >= adj.len() {
        return None;
    }
    previous[start] = Some(from);
    queue.push_back(from);
    while let Some(node) = queue.pop_front() {
        if node == to {
            break;
        }
        for &neighbor in &adj[node as usize] {
            if previous[neighbor as usize].is_some() {
                continue;
            }
            previous[neighbor as usize] = Some(node);
            queue.push_back(neighbor);
        }
    }
    previous[to as usize]?;
    let mut cursor = to;
    loop {
        let parent = previous[cursor as usize]?;
        if parent == from {
            return Some(cursor);
        }
        if parent == cursor {
            return None;
        }
        cursor = parent;
    }
}

fn muster(world: &World) -> Vec<CrewMember> {
    let roster = roster();
    let berths: Vec<u32> = world
        .rooms
        .iter()
        .filter(|room| room.kind == RoomKind::Berth)
        .map(|room| room.id)
        .collect();
    let fallback = berths.first().copied().unwrap_or(0);
    let mut buckets: Vec<Vec<u32>> = vec![Vec::new(); 16];
    for station in &world.stations {
        let Some(room) = world.rooms.get(station.room_id as usize) else {
            continue;
        };
        let dept = department_for(room.kind);
        buckets[dept_index(dept) as usize].push(station.id);
    }
    let mut cursor = [0usize; 16];
    roster
        .into_iter()
        .enumerate()
        .map(|(index, (name, department, skill))| {
            let bucket = dept_index(department) as usize;
            let station_id = if buckets[bucket].is_empty() {
                None
            } else {
                let pick = cursor[bucket] % buckets[bucket].len();
                cursor[bucket] += 1;
                Some(buckets[bucket][pick])
            };
            let berth_id = if berths.is_empty() {
                fallback
            } else {
                berths[index % berths.len()]
            };
            let room = world.rooms.get(berth_id as usize).expect("berth room");
            let (x, y) = polygon_centroid(&room.polygon);
            CrewMember {
                id: index as u32,
                name,
                department,
                skill,
                room_id: berth_id,
                x,
                y,
                deck: room.deck,
                hunger: 0.2,
                thirst: 0.2,
                fatigue: 0.15,
                health: 0.95,
                activity: Activity::Idle,
                berth_id,
                station_id,
                watch_offset_min: dept_index(department) * 45,
            }
        })
        .collect()
}

fn department_for(kind: RoomKind) -> Department {
    match kind {
        RoomKind::Engineering | RoomKind::Electrical => Department::Engineering,
        RoomKind::Desalination | RoomKind::WaterStore => Department::Water,
        RoomKind::WasteTreatment | RoomKind::Recycling => Department::Waste,
        RoomKind::Galley => Department::Galley,
        RoomKind::Bakery => Department::Bakery,
        RoomKind::Restaurant | RoomKind::Mess => Department::Service,
        RoomKind::Hydroponics => Department::Hydroponics,
        RoomKind::Fishing => Department::Fishing,
        RoomKind::Clinic | RoomKind::Infirmary => Department::Clinic,
        RoomKind::Carpentry => Department::Carpentry,
        RoomKind::Metalwork => Department::Metalwork,
        RoomKind::Sewing => Department::Sewing,
        RoomKind::Laundry => Department::Laundry,
        RoomKind::Bridge => Department::Bridge,
        RoomKind::Corridor
        | RoomKind::Stair
        | RoomKind::Berth
        | RoomKind::Head
        | RoomKind::ColdStore
        | RoomKind::DryStore
        | RoomKind::TimberStore
        | RoomKind::MetalStore
        | RoomKind::FuelStore => Department::Relief,
    }
}

fn dept_index(department: Department) -> u32 {
    match department {
        Department::Engineering => 0,
        Department::Water => 1,
        Department::Waste => 2,
        Department::Galley => 3,
        Department::Bakery => 4,
        Department::Service => 5,
        Department::Hydroponics => 6,
        Department::Fishing => 7,
        Department::Clinic => 8,
        Department::Carpentry => 9,
        Department::Metalwork => 10,
        Department::Sewing => 11,
        Department::Laundry => 12,
        Department::Bridge => 13,
        Department::Relief => 14,
    }
}

fn roster() -> Vec<(String, Department, Skill)> {
    const NAMES: [&str; 80] = [
        "Amina Okonkwo",
        "Jonas Berg",
        "Priya Nair",
        "Luis Ortega",
        "Hana Sato",
        "Erik Lind",
        "Mei Chen",
        "Omar Farouk",
        "Sofia Rossi",
        "Noah Keller",
        "Leila Haddad",
        "Ivan Petrov",
        "Chloe Martin",
        "Samir Rahman",
        "Elena Popov",
        "Thomas Wright",
        "Yara Hassan",
        "Felix Weber",
        "Ines Duarte",
        "Haruto Mori",
        "Greta Holm",
        "Mateo Silva",
        "Nia Mensah",
        "Paul Adler",
        "Rania Saleh",
        "Chris Novak",
        "Amara Diallo",
        "Hugo Bergstrom",
        "Fatima Zahra",
        "Owen Blake",
        "Sigrid Nilsen",
        "Kenji Arai",
        "Marta Costa",
        "David Levy",
        "Zola Mbatha",
        "Peter Novotny",
        "Aisha Bello",
        "Marko Ilic",
        "Nina Volkova",
        "James Carter",
        "Linh Tran",
        "Robert Meier",
        "Salma Idris",
        "Victor Almeida",
        "Emma Walsh",
        "Tariq Hassan",
        "Julia Berger",
        "Kwame Asante",
        "Helena Kranz",
        "Andre Dupont",
        "Noor Elamin",
        "Stefan Horvat",
        "Chioma Eze",
        "Martin Olsen",
        "Aya Nakamura",
        "Daniel Brooks",
        "Petra Horak",
        "Ibrahim Sane",
        "Laura Fischer",
        "Sven Lindholm",
        "Maya Kapoor",
        "George Hale",
        "Anika Bose",
        "Pavel Orlov",
        "Ruth Abebe",
        "Carlo Ricci",
        "Hanna Virtanen",
        "Yusuf Demir",
        "Clara Jensen",
        "Mohammed Ali",
        "Eva Horvath",
        "Benjamin Cole",
        "Nadia Farah",
        "Oscar Lindgren",
        "Irene Papas",
        "Samuel Boateng",
        "Freya Dahl",
        "Hassan Diallo",
        "Maria Santos",
        "Ida Berg",
    ];
    let plan = [
        (Department::Engineering, Skill::Engineering, 6u32),
        (Department::Water, Skill::Engineering, 4),
        (Department::Waste, Skill::Engineering, 4),
        (Department::Galley, Skill::Cooking, 6),
        (Department::Bakery, Skill::Baking, 4),
        (Department::Service, Skill::Cooking, 8),
        (Department::Hydroponics, Skill::Farming, 10),
        (Department::Fishing, Skill::Fishing, 4),
        (Department::Clinic, Skill::Medicine, 6),
        (Department::Carpentry, Skill::Carpentry, 4),
        (Department::Metalwork, Skill::Metalwork, 4),
        (Department::Sewing, Skill::Sewing, 4),
        (Department::Laundry, Skill::General, 3),
        (Department::Bridge, Skill::Seamanship, 4),
        (Department::Relief, Skill::General, 9),
    ];
    let mut people = Vec::with_capacity(80);
    let mut name_index = 0;
    for (department, skill, count) in plan {
        for _ in 0..count {
            people.push((NAMES[name_index].to_string(), department, skill));
            name_index += 1;
        }
    }
    debug_assert_eq!(people.len(), 80);
    people
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::fittings::{cabin_fittings, fitting_inside_room, FittingKind};
    use crate::format_unix_utc;
    use crate::geom::{polygon_centroid, segment_crosses_span, shared_edge_span};
    use crate::hull::{point_in_polygon, point_on_deck};
    use crate::kinds::RoomKind;
    use glam::Vec2;

    #[test]
    fn every_kind_sits_inside_its_hull() {
        let world = World::generate(1, 1_700_000_000).expect("layout");
        assert_eq!(world.crew.len(), 80);
        for kind in RoomKind::ALL {
            assert!(
                world.rooms.iter().any(|room| room.kind == kind),
                "missing {kind:?}"
            );
        }
        for room in &world.rooms {
            for point in &room.polygon {
                assert!(
                    point_on_deck(room.deck as usize, Vec2::new(point[0], point[1])),
                    "{} is outside the hull",
                    room.name
                );
            }
        }
        let stair = world
            .rooms
            .iter()
            .find(|room| room.kind == RoomKind::Stair)
            .unwrap();
        let reachable = reachable_from(&world, stair.id);
        assert_eq!(
            reachable,
            world.rooms.len(),
            "disconnected: {}",
            unreachable_names(&world, stair.id).join(", ")
        );
    }

    #[test]
    fn power_serves_clinic_before_hydro_and_desal() {
        let served = allocate_power(
            1.0,
            &[
                (PowerClass::Hydroponics, 0.5),
                (PowerClass::Clinic, 0.4),
                (PowerClass::Desalination, 0.5),
                (PowerClass::Galley, 0.4),
            ],
        );
        assert!((served[1] - 0.4).abs() < 1e-9);
        assert!((served[3] - 0.4).abs() < 1e-9);
        assert!((served[0] - 0.2).abs() < 1e-9);
        assert!(served[2] < 1e-9);
    }

    #[test]
    fn running_ship_throttles_hydroponics_before_the_clinic() {
        let mut world = World::generate(2, 1_700_000_000).expect("layout");
        world.solar_kw = 1.8;
        assert!(world.power.hydro_demanded == 0.0 || world.crops.len() >= 4);
        world.step();
        assert!(world.crops.len() >= 4);
        assert!(world.power.hydro_demanded > world.power.clinic_demanded);
        assert!((world.power.clinic_served - world.power.clinic_demanded).abs() < 1e-6);
        assert!(world.power.hydro_served + 1e-6 < world.power.hydro_demanded);
    }

    #[test]
    fn a_hungry_crew_member_eats() {
        let mut world = World::generate(3, 1_700_000_000).expect("layout");
        let mess = world.first_room(RoomKind::Mess).expect("mess");
        let room = world.rooms[mess as usize].clone();
        let (x, y) = polygon_centroid(&room.polygon);
        world.crew[0].room_id = mess;
        world.crew[0].deck = room.deck;
        world.crew[0].x = x;
        world.crew[0].y = y;
        world.crew[0].hunger = 0.95;
        world.add_stock(mess, Good::MealKg, 2.0);
        let before = world.total(Good::MealKg);
        world.step();
        assert!(
            world.crew[0].hunger < 0.7,
            "hunger {}",
            world.crew[0].hunger
        );
        assert!(world.total(Good::MealKg) < before);
    }

    #[test]
    fn hunger_and_thirst_deaths_are_logged() {
        let start = 1_700_000_000;
        let mut world = World::generate(11, start).expect("layout");
        let starved = world.crew[0].name.clone();
        let starved_id = world.crew[0].id;
        let dehydrated = world.crew[1].name.clone();
        let dehydrated_id = world.crew[1].id;
        let critical = world.crew[2].name.clone();
        let both = world.crew[3].name.clone();
        world.crew[0].hunger = 1.0;
        world.crew[0].thirst = 0.0;
        world.crew[0].health = 1e-6;
        world.crew[1].hunger = 0.0;
        world.crew[1].thirst = 1.0;
        world.crew[1].health = 1e-6;
        world.crew[2].hunger = 1.0;
        world.crew[2].thirst = 0.0;
        world.crew[2].health = 0.55;
        world.crew[3].hunger = 1.0;
        world.crew[3].thirst = 1.0;
        world.crew[3].health = 1e-6;
        let sim_unix = start + STEP_SECONDS;
        let logs = capture_life_logs(|| {
            world.step();
            world.step();
        });
        assert_eq!(world.crew.len(), 80);
        assert!(world.crew[0].health <= 0.0);
        assert!(world.crew[1].health <= 0.0);
        assert!(world.crew[2].health > 0.0 && world.crew[2].health < 0.55);
        assert!(world.crew[3].health <= 0.0);
        assert_logged(
            &logs,
            "died",
            &starved,
            &[
                "died from hunger;",
                "cause=\"hunger\"",
                "crew member stays aboard",
                &format!("crew_id={starved_id}"),
                &format!("sim_unix={sim_unix}"),
                &format!("sim_utc=\"{}\"", format_unix_utc(sim_unix)),
            ],
        );
        assert_logged(
            &logs,
            "died",
            &dehydrated,
            &[
                "died from thirst;",
                "cause=\"thirst\"",
                "crew member stays aboard",
                &format!("crew_id={dehydrated_id}"),
                &format!("sim_unix={sim_unix}"),
            ],
        );
        assert_logged(
            &logs,
            "critical",
            &critical,
            &["health fell below 0.55 from hunger", "cause=\"hunger\""],
        );
        assert_logged(
            &logs,
            "died",
            &both,
            &[
                "died from hunger and thirst;",
                "cause=\"hunger_and_thirst\"",
                &format!("sim_unix={sim_unix}"),
            ],
        );
        assert_eq!(count_life(&logs, "died from hunger;"), 1);
        assert_eq!(count_life(&logs, "died from thirst;"), 1);
        assert_eq!(count_life(&logs, "cause=\"hunger_and_thirst\""), 1);
    }

    #[test]
    fn eating_drinking_and_shortages_are_logged() {
        let start = 1_700_000_000;
        let mut world = World::generate(12, start).expect("layout");
        let mess = world.first_room(RoomKind::Mess).expect("mess");
        let water = world.first_room(RoomKind::WaterStore).expect("water");
        place(&mut world, 0, mess);
        place(&mut world, 1, mess);
        world.stocks.retain(|stock| stock.good != Good::MealKg);
        world.add_stock(mess, Good::MealKg, 0.50);
        world.stocks.retain(|stock| stock.good != Good::WaterL);
        world.add_stock(water, Good::WaterL, 0.20);
        for crew in &mut world.crew {
            crew.hunger = 0.2;
            crew.thirst = 0.2;
        }
        world.crew[0].hunger = 0.95;
        world.crew[1].hunger = 0.95;
        world.crew[2].hunger = 0.85;
        world.crew[3].thirst = 0.50;
        world.crew[4].thirst = 0.50;
        let fed = world.crew[0].name.clone();
        let missed = world.crew[1].name.clone();
        let onset = world.crew[2].name.clone();
        let drinker = world.crew[3].name.clone();
        let meals_before = world.total(Good::MealKg);
        let water_before = world.total(Good::WaterL);
        let logs = capture_life_logs(|| world.step());
        assert!(
            world.crew[0].hunger < 0.7,
            "hunger {}",
            world.crew[0].hunger
        );
        assert!(
            world.crew[1].hunger > 0.9,
            "missed {}",
            world.crew[1].hunger
        );
        assert!(world.total(Good::MealKg) < meals_before);
        assert!(world.total(Good::WaterL) < water_before);
        assert_logged(
            &logs,
            "ate",
            &fed,
            &[
                "ate 0.45 kg of meals",
                &format!("sim_unix={}", start + STEP_SECONDS),
            ],
        );
        assert_logged(&logs, "hunger_eased", &fed, &["after eating"]);
        assert!(
            !logs.lines().any(|line| {
                line.contains("ate:") && line.contains(&format!("name=\"{missed}\""))
            }),
            "the crew member who missed a meal was logged as eating\n{logs}"
        );
        assert_logged(&logs, "starving", &onset, &["hunger rose above 0.85"]);
        assert_logged(&logs, "drank", &drinker, &["drank 0.12 L of water"]);
        assert_ship_logged(
            &logs,
            "meals_exhausted",
            &["fell below one 0.45 kg portion", "1 crew missed a meal"],
        );
        assert_ship_logged(&logs, "water_exhausted", &["fell below one 0.12 L drink"]);
    }

    #[test]
    fn clinic_revival_does_not_raise_the_dead() {
        let start = 1_700_000_000;
        let mut world = World::generate(13, start).expect("layout");
        let clinic = world.first_room(RoomKind::Clinic).expect("clinic");
        place(&mut world, 0, clinic);
        world.stocks.retain(|stock| stock.good != Good::MedicalKg);
        world.add_stock(clinic, Good::MedicalKg, 0.05);
        world.crew[0].hunger = 0.0;
        world.crew[0].thirst = 0.0;
        world.crew[0].health = 0.0;
        let room_id = world.crew[0].room_id;
        let x = world.crew[0].x;
        let y = world.crew[0].y;
        let medical = world.total(Good::MedicalKg);
        let logs = capture_life_logs(|| {
            world.step();
            world.step();
        });
        assert_eq!(world.crew[0].health, 0.0);
        assert_eq!(world.crew[0].room_id, room_id);
        assert_eq!(world.crew[0].x, x);
        assert_eq!(world.crew[0].y, y);
        assert_eq!(world.total(Good::MedicalKg), medical);
        assert!(!logs.contains("revived:"));
        assert!(!logs.contains("died:"));

        place(&mut world, 1, clinic);
        world.crew[1].hunger = 0.0;
        world.crew[1].thirst = 0.0;
        world.crew[1].health = 0.5;
        let patient = world.crew[1].name.clone();
        let logs = capture_life_logs(|| world.step());
        let expected = (0.5_f64 + 0.05).min(1.0);
        assert!((world.crew[1].health - expected).abs() < 1e-12);
        assert_eq!(world.crew[0].health, 0.0);
        assert_eq!(world.crew[0].room_id, room_id);
        assert_eq!(world.crew[0].x, x);
        assert_eq!(world.crew[0].y, y);
        assert!((world.total(Good::MedicalKg) - (medical - 0.02)).abs() < 1e-9);
        assert_logged(
            &logs,
            "treated",
            &patient,
            &[
                "treatment raised health from 0.500",
                &format!("sim_unix={}", world.last_tick_unix),
            ],
        );
        assert!(!logs.contains("revived:"));
    }

    #[test]
    fn a_dead_crew_member_stays_put_and_is_not_revived() {
        let mut world = World::generate(14, 1_700_000_000).expect("layout");
        let clinic = world.first_room(RoomKind::Clinic).expect("clinic");
        place(&mut world, 0, clinic);
        world.stocks.retain(|stock| stock.good != Good::MedicalKg);
        world.add_stock(clinic, Good::MedicalKg, 1.0);
        world.crew[0].health = 0.0;
        world.crew[1].health = 0.0;
        let id = world.crew[0].id;
        let room_id = world.crew[0].room_id;
        let x = world.crew[0].x;
        let y = world.crew[0].y;
        let berth = world.crew[1].room_id;
        let berth_x = world.crew[1].x;
        let berth_y = world.crew[1].y;
        let medical = world.total(Good::MedicalKg);
        let logs = capture_life_logs(|| {
            world.step();
            world.step();
        });
        assert_eq!(world.crew.len(), 80);
        assert_eq!(world.crew[0].id, id);
        assert_eq!(world.crew[0].health, 0.0);
        assert_eq!(world.crew[0].room_id, room_id);
        assert_eq!(world.crew[0].x, x);
        assert_eq!(world.crew[0].y, y);
        assert_eq!(world.crew[1].health, 0.0);
        assert_eq!(world.crew[1].room_id, berth);
        assert_eq!(world.crew[1].x, berth_x);
        assert_eq!(world.crew[1].y, berth_y);
        assert_ne!(berth, clinic);
        assert_eq!(world.total(Good::MedicalKg), medical);
        assert_ne!(world.crew[0].activity, Activity::Eating);
        assert_ne!(world.crew[0].activity, Activity::Treating);
        assert!(!logs.contains("revived:"));
    }

    #[test]
    fn catch_up_matches_a_day_of_steps() {
        let start = 1_700_000_000;
        let mut stepped = World::generate(4, start).expect("layout");
        let mut jumped = stepped.clone();
        for _ in 0..1440 {
            stepped.step();
        }
        jumped.catch_up_to(start + 1440 * STEP_SECONDS);
        assert_eq!(stepped.last_tick_unix, jumped.last_tick_unix);
        assert_eq!(stepped, jumped);
    }

    #[test]
    fn memory_store_round_trip() {
        let mut world = World::generate(5, 1_700_000_000).expect("layout");
        world.step();
        let mut store = MemoryStore::default();
        store.save(&world);
        assert_eq!(store.load().as_ref(), Some(&world));
    }

    #[test]
    fn crew_steps_through_a_berth_door_and_not_through_a_wall() {
        let mut world = World::generate(7, 1_700_000_000).expect("layout");
        let berth_ids: Vec<u32> = world
            .rooms
            .iter()
            .filter(|room| room.kind == RoomKind::Berth)
            .map(|room| room.id)
            .collect();
        let berth = berth_ids
            .into_iter()
            .find_map(|id| {
                corridor_neighbor(&world, id)?;
                world.rooms.iter().find(|room| room.id == id).cloned()
            })
            .expect("berth on a corridor");
        let corridor = corridor_neighbor(&world, berth.id).expect("corridor");
        let span = shared_edge_span(&corridor.polygon, &berth.polygon).expect("door span");
        let door = span.midpoint();
        let (bx, by) = polygon_centroid(&berth.polygon);
        let start = beyond(door, [bx, by], 1.5);
        assert!(
            room_contains(&corridor, start[0], start[1]),
            "approach point should sit in the corridor"
        );

        let minute = minute_of_day(world.last_tick_unix + STEP_SECONDS);
        world.crew[0].watch_offset_min = (480 + 1440 - minute % 1440) % 1440;
        world.crew[0].hunger = 0.0;
        world.crew[0].health = 1.0;
        world.crew[0].berth_id = berth.id;
        world.crew[0].room_id = corridor.id;
        world.crew[0].deck = corridor.deck;
        world.crew[0].x = start[0];
        world.crew[0].y = start[1];
        world.step();

        let arrived = &world.crew[0];
        assert_eq!(arrived.room_id, berth.id);
        assert!(room_contains(&berth, arrived.x, arrived.y));
        assert!(segment_crosses_span(start, [arrived.x, arrived.y], span));

        let blocked_ids: Vec<u32> = world
            .rooms
            .iter()
            .filter(|room| {
                room.deck == corridor.deck && room.id != corridor.id && room.id != berth.id
            })
            .map(|room| room.id)
            .collect();
        let blocked = blocked_ids
            .into_iter()
            .find_map(|id| {
                if door_between(&world, corridor.id, id) {
                    None
                } else {
                    world.rooms.iter().find(|room| room.id == id).cloned()
                }
            })
            .expect("room that does not share the corridor door");
        let minute = minute_of_day(world.last_tick_unix + STEP_SECONDS);
        world.crew[0].watch_offset_min = (480 + 1440 - minute % 1440) % 1440;
        world.crew[0].hunger = 0.0;
        world.crew[0].health = 1.0;
        world.crew[0].berth_id = blocked.id;
        world.crew[0].room_id = corridor.id;
        world.crew[0].deck = corridor.deck;
        world.crew[0].x = start[0];
        world.crew[0].y = start[1];
        world.step();
        assert_ne!(world.crew[0].room_id, blocked.id);
        if world.crew[0].room_id != corridor.id {
            let arrived_id = world.crew[0].room_id;
            assert!(
                door_between(&world, corridor.id, arrived_id),
                "a step leaves only through a door"
            );
        }
    }

    #[test]
    fn every_berth_has_a_bed_and_every_head_has_a_toilet_and_shower() {
        let world = World::generate(8, 1_700_000_000).expect("layout");
        let mut berths = 0;
        let mut heads = 0;
        for index in 0..world.rooms.len() {
            let room = world.rooms[index].clone();
            let doorway = door_midpoint(&world, room.id);
            let fittings = cabin_fittings(room.kind, &room.polygon, doorway);
            match room.kind {
                RoomKind::Berth => {
                    berths += 1;
                    assert!(doorway.is_some(), "{} has no door", room.name);
                    assert_eq!(
                        fittings
                            .iter()
                            .filter(|fitting| fitting.kind == FittingKind::Bed)
                            .count(),
                        1,
                        "{}",
                        room.name
                    );
                }
                RoomKind::Head => {
                    heads += 1;
                    assert!(doorway.is_some(), "{} has no door", room.name);
                    assert!(
                        fittings
                            .iter()
                            .any(|fitting| fitting.kind == FittingKind::Toilet),
                        "{}",
                        room.name
                    );
                    assert!(
                        fittings
                            .iter()
                            .any(|fitting| fitting.kind == FittingKind::Shower),
                        "{}",
                        room.name
                    );
                    let toilet = fittings
                        .iter()
                        .find(|fitting| fitting.kind == FittingKind::Toilet)
                        .unwrap();
                    let shower = fittings
                        .iter()
                        .find(|fitting| fitting.kind == FittingKind::Shower)
                        .unwrap();
                    let dx = toilet.center[0] - shower.center[0];
                    let dy = toilet.center[1] - shower.center[1];
                    assert!(dx * dx + dy * dy > 1.0, "toilet and shower share a spot");
                }
                RoomKind::Corridor
                | RoomKind::Stair
                | RoomKind::Hydroponics
                | RoomKind::Fishing
                | RoomKind::ColdStore
                | RoomKind::DryStore
                | RoomKind::Galley
                | RoomKind::Bakery
                | RoomKind::Restaurant
                | RoomKind::Mess
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
                | RoomKind::Bridge => assert!(fittings.is_empty(), "{}", room.name),
            }
            for fitting in fittings {
                assert!(
                    fitting_inside_room(&room.polygon, fitting),
                    "{:?} in {} sits outside the room",
                    fitting.kind,
                    room.name
                );
            }
        }
        assert!(berths > 0);
        assert!(heads > 0);
    }

    fn place(world: &mut World, index: usize, room_id: u32) {
        let room = world.rooms[room_id as usize].clone();
        let (x, y) = polygon_centroid(&room.polygon);
        let crew = &mut world.crew[index];
        crew.room_id = room_id;
        crew.deck = room.deck;
        crew.x = x;
        crew.y = y;
    }

    fn capture_life_logs(run: impl FnOnce()) -> String {
        let buffer = Arc::new(Mutex::new(Vec::<u8>::new()));
        let writer = SharedWriter(Arc::clone(&buffer));
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .without_time()
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
            .with_writer(move || writer.clone())
            .finish();
        tracing::subscriber::with_default(subscriber, run);
        let bytes = buffer.lock().expect("log buffer").clone();
        String::from_utf8(bytes).expect("log text")
    }

    fn assert_logged(logs: &str, event: &str, name: &str, parts: &[&str]) {
        let found = logs.lines().any(|line| {
            line.contains("timehelm::life")
                && line.contains(&format!("{event}:"))
                && line.contains(&format!("name=\"{name}\""))
                && parts.iter().all(|part| line.contains(part))
        });
        assert!(
            found,
            "no {event} line for {name} including {parts:?}\n{logs}"
        );
    }

    fn assert_ship_logged(logs: &str, event: &str, parts: &[&str]) {
        let found = logs.lines().any(|line| {
            line.contains("timehelm::life")
                && line.contains(&format!("{event}:"))
                && parts.iter().all(|part| line.contains(part))
        });
        assert!(found, "no {event} line including {parts:?}\n{logs}");
    }

    fn count_life(logs: &str, needle: &str) -> usize {
        logs.lines().filter(|line| line.contains(needle)).count()
    }

    #[derive(Clone)]
    struct SharedWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for SharedWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().expect("log buffer").extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn corridor_neighbor(world: &World, room_id: u32) -> Option<Room> {
        neighbor_ids(world, room_id)
            .find_map(|other| {
                world
                    .rooms
                    .iter()
                    .find(|room| room.id == other && room.kind == RoomKind::Corridor)
            })
            .cloned()
    }

    fn neighbor_ids(world: &World, room_id: u32) -> impl Iterator<Item = u32> + '_ {
        world.doors.iter().filter_map(move |door| {
            if door.from_room == room_id {
                Some(door.to_room)
            } else if door.to_room == room_id {
                Some(door.from_room)
            } else {
                None
            }
        })
    }

    fn door_between(world: &World, left: u32, right: u32) -> bool {
        world.doors.iter().any(|door| {
            (door.from_room == left && door.to_room == right)
                || (door.from_room == right && door.to_room == left)
        })
    }

    fn door_midpoint(world: &World, room_id: u32) -> Option<[f32; 2]> {
        let room = world.rooms.iter().find(|room| room.id == room_id)?;
        neighbor_ids(world, room_id).find_map(|other| {
            let neighbor = world.rooms.iter().find(|candidate| candidate.id == other)?;
            shared_edge_span(&room.polygon, &neighbor.polygon).map(|span| span.midpoint())
        })
    }

    fn beyond(door: [f32; 2], inside: [f32; 2], distance: f32) -> [f32; 2] {
        let dx = door[0] - inside[0];
        let dy = door[1] - inside[1];
        let len = (dx * dx + dy * dy).sqrt().max(0.001);
        [door[0] + dx / len * distance, door[1] + dy / len * distance]
    }

    fn room_contains(room: &Room, x: f32, y: f32) -> bool {
        let hull: Vec<Vec2> = room
            .polygon
            .iter()
            .map(|point| Vec2::new(point[0], point[1]))
            .collect();
        point_in_polygon(Vec2::new(x, y), &hull)
    }

    fn unreachable_names(world: &World, start: u32) -> Vec<String> {
        let adj = adjacency(&world.doors, world.rooms.len());
        let mut seen = vec![false; world.rooms.len()];
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
        world
            .rooms
            .iter()
            .filter(|room| !seen[room.id as usize])
            .take(8)
            .map(|room| format!("{}#{} d{}", room.name, room.id, room.deck))
            .collect()
    }

    fn reachable_from(world: &World, start: u32) -> usize {
        let adj = adjacency(&world.doors, world.rooms.len());
        let mut seen = vec![false; world.rooms.len()];
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
        seen.into_iter().filter(|yes| *yes).count()
    }
}
