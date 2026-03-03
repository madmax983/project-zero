//! Obsessive Optimization mechanics.
//!
//! Highly skilled engineers occasionally feel the need to tinker with buildings
//! to make them run better (Spec 199).
//!
//! - **Success**: Adds `Optimized` component (+10% efficiency).
//! - **Critical Success**: Adds `Optimized` component (+25% efficiency).
//! - **Failure**: Damages the building's structure.
//! - **Critical Failure**: Instantly breaks the building (HP to 0).

use crate::layer1::structure::Structure;
use bevy_ecs::prelude::*;

/// Component indicating a building has been optimized by an obsessive engineer.
#[derive(Component, Debug, Clone, Default)]
pub struct Optimized {
    /// The efficiency bonus granted (e.g., 0.10 for +10%).
    pub efficiency_bonus: f32,
}

/// The result of an optimization attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationResult {
    /// Standard success.
    Success,
    /// Exceptional success.
    CriticalSuccess,
    /// Optimization failed, damaging the building.
    Failure,
    /// Optimization failed catastrophically, breaking the building.
    CriticalFailure,
}

/// Applies the result of an optimization attempt to a building.
pub fn perform_optimization(world: &mut World, target: Entity, result: OptimizationResult) {
    match result {
        OptimizationResult::Success => {
            if world.get::<Optimized>(target).is_none() {
                world.entity_mut(target).insert(Optimized {
                    efficiency_bonus: 0.10,
                });
            }
        }
        OptimizationResult::CriticalSuccess => {
            if world.get::<Optimized>(target).is_none() {
                world.entity_mut(target).insert(Optimized {
                    efficiency_bonus: 0.25,
                });
            }
        }
        OptimizationResult::Failure => {
            if let Some(mut structure) = world.get_mut::<Structure>(target) {
                structure.current_hp -= structure.max_hp * 0.25;
                if structure.current_hp < 0.0 {
                    structure.current_hp = 0.0;
                }
            }
        }
        OptimizationResult::CriticalFailure => {
            if let Some(mut structure) = world.get_mut::<Structure>(target) {
                structure.current_hp = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::optimization::{Optimized, OptimizationResult, perform_optimization};
    use crate::layer1::structure::Structure;

    #[test]
    fn test_optimization_success_applies_component() {
        let mut world = World::new();
        let building = world.spawn((
            Building { building_type: BuildingType::Generator },
            Structure { max_hp: 100.0, current_hp: 100.0 },
        )).id();

        let result = OptimizationResult::Success;
        perform_optimization(&mut world, building, result);

        let optimized = world.get::<Optimized>(building);
        assert!(optimized.is_some());
        assert_eq!(optimized.unwrap().efficiency_bonus, 0.10);
    }

    #[test]
    fn test_optimization_failure_damages_building() {
        let mut world = World::new();
        let building = world.spawn((
            Building { building_type: BuildingType::Generator },
            Structure { max_hp: 100.0, current_hp: 100.0 },
        )).id();

        let result = OptimizationResult::Failure;
        perform_optimization(&mut world, building, result);

        let structure = world.get::<Structure>(building).unwrap();
        assert!(structure.current_hp < 100.0);
        assert!(world.get::<Optimized>(building).is_none());
    }

    #[test]
    fn test_critical_failure_breaks_building() {
        let mut world = World::new();
        let building = world.spawn((
            Building { building_type: BuildingType::Generator },
            Structure { max_hp: 100.0, current_hp: 100.0 },
        )).id();

        let result = OptimizationResult::CriticalFailure;
        perform_optimization(&mut world, building, result);

        let structure = world.get::<Structure>(building).unwrap();
        // Should be broken (0 HP or specific Broken component)
        assert!(structure.current_hp <= 0.0);
    }

    #[test]
    fn test_already_optimized_cannot_be_optimized_again() {
        let mut world = World::new();
        let building = world.spawn((
            Building { building_type: BuildingType::Generator },
            Optimized { efficiency_bonus: 0.10 },
        )).id();

        perform_optimization(&mut world, building, OptimizationResult::Success);

        let optimized = world.get::<Optimized>(building).unwrap();
        assert_eq!(optimized.efficiency_bonus, 0.10); // Should not stack to 0.20
    }
}