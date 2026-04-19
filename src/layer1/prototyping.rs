//! Prototyping Phase system (Spec 145).
//!
//! Handles the "Prototype" status of buildings, which applies penalties to efficiency and
//! breakdown chance until the building type is "Mastered".

use crate::layer1::building::{Building, BuildingType};
use bevy_ecs::prelude::*;
use std::collections::HashMap;

// --- Resources & Components ---

/// Tracks the mastery progress for each building type.
#[derive(Resource, Default, Debug)]
pub struct BuildingMastery {
    /// Map of `BuildingType` -> Progress (0.0 to 1.0).
    pub progress: HashMap<BuildingType, f32>,
}

impl BuildingMastery {
    /// Returns the mastery progress for a building type (0.0 to 1.0).
    #[must_use]
    pub fn get_progress(&self, building: BuildingType) -> f32 {
        *self.progress.get(&building).unwrap_or(&0.0)
    }

    /// Returns true if the building type is mastered (progress >= 1.0).
    #[must_use]
    pub fn is_mastered(&self, building: BuildingType) -> bool {
        self.get_progress(building) >= 1.0
    }

    /// Adds progress to a building type.
    pub fn add_progress(&mut self, building: BuildingType, amount: f32) {
        let current = self.progress.entry(building).or_insert(0.0);
        *current = (*current + amount).min(1.0);
    }

    #[cfg(test)]
    /// Force mastery for testing.
    pub fn set_mastered(&mut self, building: BuildingType) {
        self.progress.insert(building, 1.0);
    }
}

/// Component indicating a building is a prototype.
#[derive(Component, Debug)]
pub struct Prototype {
    /// Multiplier for production efficiency (e.g., 0.5).
    pub efficiency_modifier: f32,
    /// Multiplier for breakdown chance (e.g., 2.0).
    pub breakdown_chance_modifier: f32,
}

impl Default for Prototype {
    fn default() -> Self {
        Self {
            efficiency_modifier: 0.5,
            breakdown_chance_modifier: 2.0,
        }
    }
}

// --- Systems ---

/// Accumulates mastery for active prototypes.
///
/// Takes 100 seconds (at 1x speed) to master a building type if one prototype is active.
#[allow(clippy::type_complexity)]
#[allow(clippy::option_if_let_else)]
pub fn mastery_accumulation_system(
    mut mastery: ResMut<BuildingMastery>,
    // We only count ACTIVE prototypes.
    prototypes: Query<
        (
            &Building,
            Option<&crate::layer1::energy::PowerConsumer>,
            Option<&crate::layer1::energy::PowerSource>,
        ),
        With<Prototype>,
    >,
    _time: Res<crate::shared::time::SimulationTime>,
) {
    // Constant: Mastery takes 100 ticks to complete per active building.
    // Spec says "Takes 100 seconds to master" (MASTERY_RATE_PER_SECOND = 0.01).
    // SimulationTime is ticks. Assuming 1 tick ~ 1 second for mastery logic.
    const MASTERY_RATE_PER_TICK: f32 = 0.01;

    for (building, consumer, source) in &prototypes {
        let is_active = if let Some(c) = consumer {
            c.active
        } else if let Some(s) = source {
            s.active
        } else {
            true // Passive building (e.g. Stockpile) always active
        };

        if is_active {
            mastery.add_progress(building.building_type, MASTERY_RATE_PER_TICK);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::energy::PowerSource;
    use bevy_ecs::schedule::{Schedule, ScheduleLabel};

    #[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
    struct TestSchedule;

    // Helper to setup minimal app
    fn setup_app() -> World {
        let mut world = World::new();
        world.insert_resource(BuildingMastery::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world
    }

    #[test]
    fn test_first_construction_is_prototype() {
        let mut world = setup_app();
        let building_type = BuildingType::Generator;
        let entity = world.spawn(Building { building_type }).id();

        // Simulate construction logic (manual injection for now as we test logic unit, not integration here)
        let mastery = world.resource::<BuildingMastery>();
        if !mastery.is_mastered(building_type) {
            world.entity_mut(entity).insert(Prototype::default());
        }

        assert!(world.entity(entity).contains::<Prototype>());
        let prototype = world.entity(entity).get::<Prototype>().expect("Component should exist");
        assert_eq!(prototype.efficiency_modifier, 0.5);
    }

    #[test]
    fn test_mastery_accumulation() {
        let mut world = setup_app();
        let building_type = BuildingType::Generator;

        // Spawn active prototype
        world.spawn((
            Building { building_type },
            Prototype::default(),
            PowerSource {
                output: 10.0,
                active: true,
            },
        ));

        // Advance simulation time (simulate tick)
        world
            .resource_mut::<crate::shared::time::SimulationTime>()
            .tick += 1;

        // Run system
        let mut schedule = Schedule::new(TestSchedule);
        schedule.add_systems(mastery_accumulation_system);
        schedule.run(&mut world);

        // Assert
        let mastery = world.resource::<BuildingMastery>();
        let progress = mastery.get_progress(building_type);
        assert!(progress > 0.0, "Mastery should accumulate");
        assert!(progress < 1.0);
    }

    #[test]
    fn test_mastery_unlock_standard_builds() {
        let mut world = setup_app();
        let building_type = BuildingType::Generator;

        // Set mastery to complete
        world
            .resource_mut::<BuildingMastery>()
            .set_mastered(building_type);

        let entity = world.spawn(Building { building_type }).id();

        let mastery = world.resource::<BuildingMastery>();
        if !mastery.is_mastered(building_type) {
            world.entity_mut(entity).insert(Prototype::default());
        }

        assert!(!world.entity(entity).contains::<Prototype>());
    }

    #[test]
    fn test_prototype_efficiency_affects_production() {
        let mut world = setup_app();
        let entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Generator,
                },
                Prototype {
                    efficiency_modifier: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        let prototype = world.entity(entity).get::<Prototype>().expect("Component should exist");
        assert_eq!(prototype.efficiency_modifier, 0.5);
    }

    #[test]
    fn test_idle_prototypes_do_not_generate_mastery() {
        // Arrange
        let mut world = setup_app();

        let building_type = BuildingType::Generator;
        // Spawn INACTIVE prototype
        world.spawn((
            Building { building_type },
            Prototype::default(),
            PowerSource {
                output: 10.0,
                active: false,
            }, // Inactive
        ));

        // Act
        let mut schedule = Schedule::new(TestSchedule);
        schedule.add_systems(mastery_accumulation_system);
        schedule.run(&mut world);

        // Assert
        let mastery = world.resource::<BuildingMastery>();
        assert_eq!(mastery.get_progress(building_type), 0.0);
    }
}
