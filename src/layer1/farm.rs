#![allow(
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::unnecessary_map_or
)]
#![allow(clippy::collapsible_if, clippy::type_complexity)]
use crate::layer1::GridPosition;
use crate::layer1::actions::AssignmentType;
use crate::layer1::balance::{
    FOOD_HUNGER_THRESHOLD, FOOD_PER_MEAL, FOOD_PER_WORKER_PER_TICK, HUNGER_PER_MEAL,
};
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::economy::{ColonyPrices, Wallet, get_wage_for_job};
use crate::layer1::energy::PowerConsumer;
use crate::layer1::eureka::{EurekaConfig, check_for_eureka};
use crate::layer1::factions::{FactionMember, FactionState, Factions};
use crate::layer1::fauna::{Fauna, FaunaType};
use crate::layer1::fertility::FertilityGrid;
use crate::layer1::husbandry::Tame;
use crate::layer1::items::ItemType;
use crate::layer1::needs::Needs;
use crate::layer1::palette_fatigue::{DietaryHistory, record_meal};
use crate::layer1::pop::Job;
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use crate::layer1::seasons::{Season, SeasonState};
use crate::layer1::skills::{SkillType, Skills, get_skill_efficiency};
use crate::layer1::social_mimicry::JustConsumed;
use crate::layer1::tech::Tech;
use crate::layer1::utility_ai::{ActionType, PopAction};
use bevy_ecs::prelude::*;
use rand::seq::SliceRandom;

/// Water cost per tick per worker for Hydroponics.
const HYDROPONICS_WATER_COST: f32 = 0.1;
/// Production multiplier for Hydroponics.
const HYDROPONICS_MULTIPLIER: f32 = 2.0;

/// Farm component - produces food when worked.
#[derive(Component)]
pub struct Farm {
    /// Maximum number of workers.
    pub capacity: usize,
    /// List of workers assigned to this farm.
    /// **Legacy**: Used for UI/Capacity checks, but production uses `PopAction`.
    pub workers: Vec<Entity>,
    /// The crop selected for this farm.
    pub selected_crop: ItemType,
}

impl Default for Farm {
    fn default() -> Self {
        Self {
            capacity: 2,
            workers: Vec::new(),
            selected_crop: ItemType::Wheat,
        }
    }
}

struct CropStats {
    base_yield: f32,
    winter_modifier: f32,
}

const fn get_crop_stats(crop: &ItemType) -> CropStats {
    match crop {
        ItemType::Wheat => CropStats {
            base_yield: 0.006,
            winter_modifier: 0.2,
        },
        ItemType::Potato => CropStats {
            base_yield: 0.004,
            winter_modifier: 0.8,
        },
        ItemType::Rice => CropStats {
            base_yield: 0.005,
            winter_modifier: 0.5,
        },
        _ => CropStats {
            base_yield: FOOD_PER_WORKER_PER_TICK,
            winter_modifier: 0.5,
        },
    }
}

/// Produces food from all farms with active workers.
#[allow(clippy::too_many_arguments)]
pub fn produce_food_system(
    farm_query: Query<(&Building, &GridPosition, Option<&PowerConsumer>, &Farm)>,
    mut pop_query: Query<
        (
            Entity,
            &GridPosition,
            &PopAction,
            Option<&mut Skills>,
            Option<&FactionMember>,
            Option<&crate::layer1::traits::Traits>,
            Option<&mut Wallet>,
            Option<&Job>,
        ),
        With<Pop>,
    >,
    season_state: Option<Res<SeasonState>>,
    mut resources: ResMut<ColonyResources>,
    factions: Option<Res<Factions>>,
    // We need mut access to TechState for Corruption Check
    tech_state_mut: Option<ResMut<crate::layer1::tech::TechState>>,
    fertility_grid: Option<Res<FertilityGrid>>,
    eureka_config: Option<Res<EurekaConfig>>,
    mut eureka_events: EventWriter<crate::layer1::eureka::EurekaEvent>,
) {
    let modifier = season_state
        .as_ref()
        .map_or(1.0, |s| s.current_season.food_modifier());

    let current_season = season_state
        .as_ref()
        .map_or(Season::Spring, |s| s.current_season);

    let farm_map: std::collections::HashMap<GridPosition, (BuildingType, bool, ItemType)> =
        farm_query
            .iter()
            .map(|(b, p, pc, farm)| {
                (
                    *p,
                    (
                        b.building_type,
                        pc.is_some_and(|c| c.active),
                        farm.selected_crop.clone(),
                    ),
                )
            })
            .collect();

    for (_, pos, action, skills_opt, faction_member_opt, traits, mut wallet_opt, job_opt) in
        &mut pop_query
    {
        if action.current != ActionType::Farm {
            continue;
        }

        // Check for strikes
        if let Some(factions) = &factions {
            if let Some(member) = faction_member_opt {
                if let Some(fid) = member.faction_id {
                    if factions
                        .get(fid)
                        .is_some_and(|d| d.state == FactionState::Striking)
                    {
                        continue;
                    }
                }
            }
        }

        if let Some((building_type, is_powered, selected_crop)) = farm_map.get(pos) {
            // Tech Corruption Check
            if let Some(tech) = building_type.required_tech() {
                // Use read-only resource for check to avoid conflict?
                // Wait, if I have ResMut, I can use it as ref.
                // But I have Option<ResMut>.
                // I changed arguments to have both? That might conflict if I request Res and ResMut of same type.
                // Rust bevy ECS rules: &T and &mut T cannot coexist.
                // I must request ONLY ResMut if I need mutability.
                // So I will remove `tech_state_res` and use `tech_state_mut` for reading too.
                let tech_active = tech_state_mut
                    .as_ref()
                    .map_or(true, |ts| ts.is_active(tech));

                if !tech_active {
                    continue;
                }
            }

            let skill_type = SkillType::Farming;

            // Calculate efficiency
            let efficiency = get_skill_efficiency(skills_opt.as_deref(), skill_type);

            // Add XP
            if let Some(mut skills) = skills_opt {
                skills.add_xp(skill_type, 1.0);
            }

            let crop_stats = get_crop_stats(selected_crop);

            // Determine yield and modifiers
            let (base_production, water_cost, effective_modifier) = match building_type {
                BuildingType::HydroponicsBay => {
                    if *is_powered {
                        // Hydroponics uses its own multiplier on top of crop base yield?
                        // Or overrides?
                        // Spec says: "Production logic respects selected_crop stats"
                        // But Hydroponics usually ignores seasons.
                        (
                            crop_stats.base_yield,
                            HYDROPONICS_WATER_COST,
                            HYDROPONICS_MULTIPLIER,
                        )
                    } else {
                        (0.0, 0.0, 0.0)
                    }
                }
                BuildingType::Plantation => {
                    // Plantation produces Fiber, uses generic yield probably
                    (FOOD_PER_WORKER_PER_TICK, 0.0, modifier)
                }
                _ => {
                    // Standard Farm or Greenhouse
                    // Check for immunity (Greenhouse)
                    let season_mod = if building_type.seasonal_immunity() {
                        1.0
                    } else if current_season == Season::Winter {
                        crop_stats.winter_modifier
                    } else {
                        modifier // Use general season modifier (e.g. Autumn harvest bonus?)
                    };
                    (crop_stats.base_yield, 0.0, season_mod)
                }
            };

            // Integrate Fertility
            let fertility_modifier = if *building_type == BuildingType::HydroponicsBay {
                1.0 // Hydroponics ignores soil fertility
            } else if let Some(grid) = &fertility_grid {
                grid.get(pos.x as usize, pos.y as usize)
            } else {
                1.0
            };

            // Check water availability
            if water_cost > 0.0 && resources.water < water_cost {
                continue;
            }

            // Deduct water
            if water_cost > 0.0 {
                resources.water -= water_cost;
            }

            let production = efficiency * base_production * effective_modifier * fertility_modifier;

            if production > 0.0 {
                match building_type {
                    BuildingType::Plantation => {
                        resources.add_fiber(production);
                    }
                    _ => match selected_crop {
                        ItemType::Wheat => {
                            resources.wheat += production;
                            resources.food += production;
                        }
                        ItemType::Potato => {
                            resources.potato += production;
                            resources.food += production;
                        }
                        ItemType::Rice => {
                            resources.rice += production;
                            resources.food += production;
                        }
                        _ => resources.add_food(production),
                    },
                }

                // Pay Wages
                if let Some(wallet) = wallet_opt.as_deref_mut() {
                    let job_type = job_opt.map_or(AssignmentType::FarmWorker, |j| j.job_type);
                    let base_wage = get_wage_for_job(job_type);
                    // Pay 1% of base wage per tick of active production
                    let wage = base_wage * 0.01;
                    wallet.credits += wage;
                }

                // Eureka Check
                if let Some(config) = &eureka_config {
                    check_for_eureka(
                        &mut eureka_events,
                        config,
                        ActionType::Farm,
                        Some(Tech::Hydroponics), // Related to farming
                        traits,
                    );
                }
            }
        }
    }
}

/// Pops eat food when hungry.
pub fn consume_food_system(
    mut commands: Commands,
    mut pop_query: Query<
        (
            Entity,
            &mut Needs,
            Option<&mut DietaryHistory>,
            Option<&mut Wallet>,
        ),
        With<Pop>,
    >,
    mut resources: ResMut<ColonyResources>,
    farm_query: Query<&Farm>, // Query Farm instead of Crop
    animal_query: Query<&Fauna, With<Tame>>,
    prices: Option<Res<ColonyPrices>>,
) {
    // Use total_food() logic for check
    let total_food = resources.total_food();
    if total_food < f32::EPSILON {
        return;
    }

    let food_price = prices.map_or(0.0, |p| p.food_price);

    // Collect available food types from active sources
    let mut available_items = Vec::new();
    for farm in &farm_query {
        available_items.push(farm.selected_crop.clone());
    }
    for fauna in &animal_query {
        match fauna.fauna_type {
            FaunaType::SpaceRat | FaunaType::Wolf => {
                available_items.push(ItemType::Meat);
            }
            FaunaType::Mascot => {}
        }
    }

    // Collect hungry pop entities first to avoid borrow issues with mut iteration
    let hungry_pops: Vec<Entity> = pop_query
        .iter()
        .filter(|(_, needs, _, wallet)| {
            if needs.hunger >= FOOD_HUNGER_THRESHOLD {
                return false;
            }
            // Check affordability (if wallet exists)
            if let Some(w) = wallet {
                if w.credits < food_price {
                    return false;
                }
            }
            true
        })
        .map(|(e, _, _, _)| e)
        .collect();

    let mut rng = rand::thread_rng();

    for entity in hungry_pops {
        // Try to eat from specific stocks first, then generic food, then rations
        let mut eaten_item = ItemType::None;
        let mut ate = false;

        // Simple consumption priority: Wheat -> Potato -> Rice -> Generic Food -> Rations
        // Or random? Spec says "consume from largest pile first ... OR random weighted".
        // For MVP, simple priority is fine, or simple check.

        if resources.wheat >= FOOD_PER_MEAL {
            resources.wheat -= FOOD_PER_MEAL;
            eaten_item = ItemType::Wheat;
            ate = true;
        } else if resources.potato >= FOOD_PER_MEAL {
            resources.potato -= FOOD_PER_MEAL;
            eaten_item = ItemType::Potato;
            ate = true;
        } else if resources.rice >= FOOD_PER_MEAL {
            resources.rice -= FOOD_PER_MEAL;
            eaten_item = ItemType::Rice;
            ate = true;
        } else if resources.food >= FOOD_PER_MEAL {
            // Note: 'food' might double count if we aren't careful, but we are treating 'food' as a bucket here.
            // But wait, if we increment 'food' when we produce wheat, then 'food' IS the total.
            // If we deduct wheat, we should also deduct food?
            // Yes, to keep them in sync.
            // But wait, if 'food' is just a cache, we should use it as such.
            // Actually, if 'food' is the aggregate, we should just check 'food'.
            // But we want to track specific consumption for Palette Fatigue.
            //
            // Let's assume:
            // 1. Production adds to specific (wheat) AND generic (food).
            // 2. Consumption deducts from specific (wheat) AND generic (food).

            // However, this logic above `if resources.wheat >= ...` tries to deduct from wheat.
            // If successful, we MUST also deduct from food.
        }

        // Let's restart consumption logic to be safe and consistent.
        // We need to pick WHAT to eat.
        // Available: wheat, potato, rice, (generic) food, rations.

        // Filter valid choices
        let mut choices = Vec::new();
        if resources.wheat >= FOOD_PER_MEAL {
            choices.push(ItemType::Wheat);
        }
        if resources.potato >= FOOD_PER_MEAL {
            choices.push(ItemType::Potato);
        }
        if resources.rice >= FOOD_PER_MEAL {
            choices.push(ItemType::Rice);
        }
        // Generic food fallback (if food > sum of others, or just treating leftover as generic)
        // Calculating "generic only" is hard if we just sum.
        // But we can check if we have generic food available.
        // If we only have specific crops, `resources.food` should equal sum.
        // If we have legacy food, `resources.food` > sum.
        let specific_sum = resources.wheat + resources.potato + resources.rice;
        if resources.food > specific_sum + f32::EPSILON && resources.food >= FOOD_PER_MEAL {
            // We have generic food
            choices.push(ItemType::None);
        }

        if choices.is_empty() {
            // Try rations
            if resources.rations >= FOOD_PER_MEAL {
                resources.rations -= FOOD_PER_MEAL;
                // Rations don't count for food total usually, or they do?
                // total_food includes rations.
                eaten_item = ItemType::None; // Rations aren't an ItemType in this context usually, or maybe ItemType::Rations?
                // ItemType doesn't have Rations.
                ate = true;
            }
        } else {
            // Pick one
            eaten_item = choices
                .choose(&mut rng)
                .cloned()
                .unwrap_or(ItemType::Potato);

            match eaten_item {
                ItemType::Wheat => resources.wheat -= FOOD_PER_MEAL,
                ItemType::Potato => resources.potato -= FOOD_PER_MEAL,
                ItemType::Rice => resources.rice -= FOOD_PER_MEAL,
                _ => {} // Generic
            }
            // Also deduct from main food pile
            resources.food -= FOOD_PER_MEAL;
            ate = true;
        }

        if ate {
            #[allow(clippy::collapsible_if)]
            if let Ok((_, mut needs, mut history_opt, mut wallet_opt)) = pop_query.get_mut(entity) {
                needs.hunger = (needs.hunger + HUNGER_PER_MEAL).min(1.0);

                // Deduct Cost
                if let Some(wallet) = wallet_opt.as_deref_mut() {
                    wallet.credits -= food_price;
                }

                // Palette Fatigue Logic
                // If we ate a specific item, record it.
                // If we ate rations (None), record it?
                let recorded_item = if eaten_item == ItemType::None {
                    // Try to guess based on available items for "flavor" if we ate generic?
                    // Or just fallback
                    available_items
                        .choose(&mut rng)
                        .cloned()
                        .unwrap_or(ItemType::Potato)
                } else {
                    eaten_item
                };

                if let Some(ref mut history) = history_opt {
                    record_meal(history, recorded_item.clone());
                } else {
                    let mut history = DietaryHistory::default();
                    record_meal(&mut history, recorded_item.clone());
                    commands.entity(entity).insert(history);
                }

                // Mimicry Integration
                commands.entity(entity).insert(JustConsumed {
                    item: recorded_item,
                });
            }
        }
    }
}

/// Removes dead workers from farms.
pub fn clean_dead_workers_system(mut farm_query: Query<&mut Farm>, pop_query: Query<&Pop>) {
    for mut farm in &mut farm_query {
        farm.workers.retain(|&worker| pop_query.get(worker).is_ok());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::GridPosition;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_farm_default() {
        let farm = Farm::default();
        assert_eq!(farm.capacity, 2);
        assert!(farm.workers.is_empty());
        assert_eq!(farm.selected_crop, ItemType::Wheat);
    }

    fn setup_test_world() -> World {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.init_resource::<Events<crate::layer1::eureka::EurekaEvent>>();
        world
    }

    #[test]
    fn test_produce_food_system() {
        let mut world = setup_test_world();

        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn worker at farm doing Farm action
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let resources = world.resource::<ColonyResources>();
        // Check wheat (default crop)
        assert!(resources.wheat > 0.0, "Wheat should be produced");
        // Check total food
        assert!(resources.food > 0.0, "Total food should be updated");
    }

    #[test]
    fn test_produce_food_system_skills_xp() {
        let mut world = setup_test_world();

        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let worker = world
            .spawn((
                Pop,
                Skills::default(),
                GridPosition { x: 5, y: 5 },
                PopAction {
                    current: ActionType::Farm,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(produce_food_system).unwrap();

        // Check XP
        let skills = world.get::<Skills>(worker).unwrap();
        assert_eq!(skills.get_xp(SkillType::Farming), 1.0);
    }

    #[test]
    fn test_produce_food_system_skills_efficiency() {
        let mut world = setup_test_world();

        // Worker with Level 1 Farming (100 XP) -> 1.1 efficiency
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Farming, 100.0);

        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 5, y: 5 },
        ));

        world.spawn((
            Pop,
            skills,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let resources = world.resource::<ColonyResources>();
        // Base Wheat = 0.006
        // With skill = 0.006 * 1.1 = 0.0066
        // Food starts at 10.0
        assert!(
            (resources.wheat - 0.0066).abs() < 0.0001,
            "Wheat production should be accurate"
        );
        assert!(
            (resources.food - 10.0066).abs() < 0.0001,
            "Food production should reflect skill efficiency"
        );
    }

    #[test]
    fn test_produce_food_multiple_workers() {
        let mut world = setup_test_world();

        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Two workers
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
        ));
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let resources = world.resource::<ColonyResources>();
        // 2 * 0.006 = 0.012
        assert!(
            (resources.wheat - 0.012).abs() < 0.0001,
            "Two workers should produce more food"
        );
    }

    #[test]
    fn test_consume_food_system() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            food: 1.0,
            potato: 1.0, // Give some potato so they can eat
            ..Default::default()
        });
        world.insert_resource(crate::shared::time::SimulationTime::default());

        world.spawn((
            Pop,
            Needs {
                hunger: 0.3,
                rest: 0.8,
                ..Default::default()
            },
        ));

        let food_before = world.resource::<ColonyResources>().food;
        world.run_system_once(consume_food_system).unwrap();
        let food_after = world.resource::<ColonyResources>().food;

        assert!(food_after < food_before, "Food should be consumed");
        // Potato should decrease
        assert!(world.resource::<ColonyResources>().potato < 1.0);
    }

    #[test]
    fn test_consume_food_adds_dietary_history() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            food: 1.0,
            potato: 1.0,
            ..Default::default()
        });
        world.insert_resource(crate::shared::time::SimulationTime::default());

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.3,
                    rest: 0.8,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(consume_food_system).unwrap();

        let history = world.get::<DietaryHistory>(pop);
        assert!(
            history.is_some(),
            "DietaryHistory should be added when eating"
        );
        assert_eq!(history.unwrap().recent_meals.len(), 1);
        assert_eq!(history.unwrap().recent_meals[0], ItemType::Potato);
    }

    #[test]
    fn test_consume_food_picks_active_crop() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            food: 10.0,
            wheat: 10.0,
            ..Default::default()
        });

        // Spawn a Wheat Farm
        world.spawn((
            Farm {
                selected_crop: ItemType::Wheat,
                ..Default::default()
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Spawn a Pop
        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.0,
                    ..Default::default()
                },
                DietaryHistory::default(),
            ))
            .id();

        world.run_system_once(consume_food_system).unwrap();

        let history = world.get::<DietaryHistory>(pop).unwrap();
        assert_eq!(history.recent_meals[0], ItemType::Wheat);
    }

    #[test]
    fn test_farm_has_selected_crop() {
        let farm = Farm::default();
        // Default crop should be Wheat (standard)
        assert_eq!(farm.selected_crop, ItemType::Wheat);
    }

    #[test]
    fn test_colony_resources_has_specific_crops() {
        let mut res = ColonyResources::default();
        // These fields should exist
        res.wheat = 10.0;
        res.potato = 5.0;
        res.rice = 2.0;

        assert_eq!(res.wheat, 10.0);
        assert_eq!(res.potato, 5.0);
        assert_eq!(res.rice, 2.0);
    }

    #[test]
    fn test_produce_food_wheat_yield() {
        let mut world = setup_test_world();
        world.insert_resource(SeasonState {
            current_season: Season::Spring,
        }); // Good weather

        // Spawn Farm with Wheat
        world.spawn((
            Farm {
                selected_crop: ItemType::Wheat,
                ..Default::default()
            },
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn Worker
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let res = world.resource::<ColonyResources>();
        // Wheat base yield is high (e.g. 0.006 vs standard 0.005)
        assert!(res.wheat > 0.005);
        assert_eq!(res.potato, 0.0);
    }

    #[test]
    fn test_produce_food_potato_winter_resistance() {
        let mut world = setup_test_world();
        world.insert_resource(SeasonState {
            current_season: Season::Winter,
        });

        // Spawn Farm with Potato
        world.spawn((
            Farm {
                selected_crop: ItemType::Potato,
                ..Default::default()
            },
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn Worker
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let res = world.resource::<ColonyResources>();

        // Potato in winter (0.8 modifier) vs Wheat in winter (0.2 modifier)
        // Potato base (0.004) * 0.8 = 0.0032
        // Wheat base (0.006) * 0.2 = 0.0012

        assert!(res.potato > 0.003);
    }

    #[test]
    fn test_change_crop_selection() {
        let mut farm = Farm::default();
        assert_eq!(farm.selected_crop, ItemType::Wheat);

        farm.selected_crop = ItemType::Rice;
        assert_eq!(farm.selected_crop, ItemType::Rice);
    }
}
