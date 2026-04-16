//! Tech Envy System (Spec 162).
//!
//! Handles the "Obsolescence" mood penalty for pops working in outdated buildings.

use crate::layer1::actions::AssignedTo;
use crate::layer1::building::Building;
use crate::layer1::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Configuration for the Tech Envy system.
#[derive(Resource)]
pub struct TechEnvyConfig {
    /// Penalty multiplier per tier gap. Default: -0.1.
    pub penalty_multiplier: f32,
    /// Duration of the mood modifier in ticks. Default: 10.
    pub duration: u32,
}

impl Default for TechEnvyConfig {
    fn default() -> Self {
        Self {
            penalty_multiplier: -0.1,
            duration: 10,
        }
    }
}

/// System that applies "Obsolescence" mood modifier to pops working in lower-tier buildings.
///
/// It compares the tier of the building a pop is assigned to against the maximum tier
/// available in the colony for that category.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::collapsible_if
)]
pub fn tech_envy_system(
    mut pops: Query<(&AssignedTo, &mut Morale)>,
    buildings: Query<&Building>,
    config: Res<TechEnvyConfig>,
) {
    // 1. Calculate Max Tier per Category
    let mut max_tiers = HashMap::new();
    for building in buildings.iter() {
        if let Some((category, tier)) = building.building_type.tier_info() {
            let current_max = max_tiers.entry(category).or_insert(tier);
            if tier > *current_max {
                *current_max = tier;
            }
        }
    }

    // 2. Check Pops
    for (assignment, mut morale) in &mut pops {
        if let Ok(building) = buildings.get(assignment.entity) {
            if let Some((category, tier)) = building.building_type.tier_info() {
                if let Some(&max_tier) = max_tiers.get(&category) {
                    if tier < max_tier {
                        // Apply Envy
                        // Calculate penalty: multiplier * (gap)
                        let gap = (max_tier as i32) - (tier as i32);
                        let penalty = config.penalty_multiplier * (gap as f32);

                        // Check if modifier already exists to avoid stacking duplicates
                        // We want to refresh it or keep it.
                        morale.modifiers.retain(|m| m.label != "Obsolescence");

                        morale.add_modifier(MoodModifier {
                            label: "Obsolescence".to_string(),
                            value: penalty,
                            duration: config.duration,
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::layer1::building::{Building, BuildingType, Category, Tier};
    use crate::layer1::morale::Morale;

    #[test]
    fn test_building_tiers_and_categories() {
        // Verify metadata exists
        assert_eq!(
            BuildingType::Farm.tier_info(),
            Some((Category::FoodProduction, Tier::Basic))
        );
        assert_eq!(
            BuildingType::HydroponicsBay.tier_info(),
            Some((Category::FoodProduction, Tier::HighTech))
        );
        assert_eq!(BuildingType::Housing.tier_info(), None); // Housing doesn't trigger envy (handled by Room Quality)
    }

    #[test]
    fn test_tech_envy_trigger() {
        let mut world = World::new();
        // Setup systems and resources
        world.insert_resource(TechEnvyConfig::default());

        // 1. Spawn a Tier 1 Building (Farm)
        let farm = world
            .spawn((Building {
                building_type: BuildingType::Farm,
            },))
            .id();

        // 2. Spawn a Pop assigned to the Farm
        let pop = world
            .spawn((
                AssignedTo {
                    entity: farm,
                    assignment_type: AssignmentType::FarmWorker,
                },
                Morale::default(),
            ))
            .id();

        // 3. Spawn a Tier 2 Building (HydroponicsBay) of the same category
        world.spawn((Building {
            building_type: BuildingType::HydroponicsBay,
        },));

        // 4. Run the system
        let mut schedule = Schedule::default();
        schedule.add_systems(tech_envy_system);
        schedule.run(&mut world);

        // 5. Assert Pop has "Obsolescence" modifier
        let morale = world.get::<Morale>(pop).expect("Pop missing Morale");

        let obs_mod = morale.modifiers.iter().find(|m| m.label == "Obsolescence");
        assert!(obs_mod.is_some(), "Pop should have Obsolescence modifier");

        let modifier = obs_mod.expect("Missing Obsolescence modifier");
        // Gap is HighTech(3) - Basic(1) = 2. Penalty = -0.1 * 2 = -0.2
        assert!((modifier.value - -0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_no_envy_same_tier() {
        let mut world = World::new();
        world.insert_resource(TechEnvyConfig::default());

        // Spawn Tier 1 Farm and Pop
        let farm = world
            .spawn(Building {
                building_type: BuildingType::Farm,
            })
            .id();
        let pop = world
            .spawn((
                AssignedTo {
                    entity: farm,
                    assignment_type: AssignmentType::FarmWorker,
                },
                Morale::default(),
            ))
            .id();

        // Spawn another Tier 1 Farm
        world.spawn(Building {
            building_type: BuildingType::Farm,
        });

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(tech_envy_system);
        schedule.run(&mut world);

        // Assert NO modifier
        let morale = world.get::<Morale>(pop).expect("Pop missing Morale");
        assert!(!morale.modifiers.iter().any(|m| m.label == "Obsolescence"));
    }

    #[test]
    fn test_no_envy_different_category() {
        let mut world = World::new();
        world.insert_resource(TechEnvyConfig::default());

        // Spawn Tier 1 Farm and Pop
        let farm = world
            .spawn(Building {
                building_type: BuildingType::Farm,
            })
            .id();
        let pop = world
            .spawn((
                AssignedTo {
                    entity: farm,
                    assignment_type: AssignmentType::FarmWorker,
                },
                Morale::default(),
            ))
            .id();

        // Spawn Tier 3 Power Building (AncientReactor)
        world.spawn(Building {
            building_type: BuildingType::AncientReactor,
        });

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(tech_envy_system);
        schedule.run(&mut world);

        // Assert NO modifier
        let morale = world.get::<Morale>(pop).expect("Pop missing Morale");
        assert!(!morale.modifiers.iter().any(|m| m.label == "Obsolescence"));
    }

    #[test]
    fn test_envy_refresh() {
        let mut world = World::new();
        world.insert_resource(TechEnvyConfig::default());

        // Spawn Tier 1 Farm and Pop
        let farm = world
            .spawn(Building {
                building_type: BuildingType::Farm,
            })
            .id();
        let pop = world
            .spawn((
                AssignedTo {
                    entity: farm,
                    assignment_type: AssignmentType::FarmWorker,
                },
                Morale::default(),
            ))
            .id();

        // Spawn Tier 2 Greenhouse
        world.spawn(Building {
            building_type: BuildingType::Greenhouse,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(tech_envy_system);

        // Run once
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).expect("Pop missing Morale");
        let count_1 = morale
            .modifiers
            .iter()
            .filter(|m| m.label == "Obsolescence")
            .count();
        assert_eq!(count_1, 1);

        // Run again
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).expect("Pop missing Morale");
        let count_2 = morale
            .modifiers
            .iter()
            .filter(|m| m.label == "Obsolescence")
            .count();
        assert_eq!(count_2, 1, "Should not stack duplicate modifiers");

        let modifier = morale
            .modifiers
            .iter()
            .find(|m| m.label == "Obsolescence")
            .expect("Missing Obsolescence modifier");
        assert_eq!(modifier.duration, 10, "Duration should be refreshed");
    }
}
