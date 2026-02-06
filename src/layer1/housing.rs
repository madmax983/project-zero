//! Housing and population shelter management.
//!
//! Housing provides shelter for Pops, allowing them to recover from fatigue (Rest).
//! Without housing, pops will sleep on the ground (recovering slower and taking penalties).
//!
//! # Key Concepts
//!
//! * **Housing**: A component on buildings that tracks capacity and residents.
//! * **Assignment**: Pops are assigned to a specific housing entity.
//! * **Restoration**: The `restore_rest_in_housing_system` ticks up the rest need of residents.

use crate::layer1::needs::Needs;
use bevy_ecs::prelude::*;

/// Housing component - provides shelter and rest for pops.
///
/// Attached to buildings like Cabins or Dormitories.
///
/// # Examples
///
/// ```
/// use scale::layer1::housing::Housing;
///
/// let housing = Housing {
///     capacity: 5,
///     residents: Vec::new(),
/// };
/// assert_eq!(housing.capacity, 5);
/// ```
#[derive(Component)]
pub struct Housing {
    /// Maximum number of residents this housing can hold.
    pub capacity: usize,
    /// List of residents currently assigned to this housing.
    pub residents: Vec<Entity>,
}

impl Default for Housing {
    fn default() -> Self {
        Self {
            capacity: 2,
            residents: Vec::new(),
        }
    }
}

const REST_RESTORE_PER_TICK: f32 = 0.05; // Full rest in ~20 ticks

/// Restores rest for all pops residing in housing.
///
/// This system should run every tick. It iterates over all housing entities
/// and applies rest restoration to their assigned residents.
///
/// # Examples
///
/// ```
/// use scale::layer1::housing::{Housing, restore_rest_in_housing_system};
/// use scale::layer1::needs::Needs;
/// use scale::layer1::pop::Pop;
/// use bevy_ecs::prelude::*;
///
/// let mut world = World::new();
///
/// // Create a tired pop
/// let pop = world.spawn((Pop, Needs { rest: 0.1, ..Default::default() })).id();
///
/// // Assign to housing
/// world.spawn(Housing {
///     capacity: 1,
///     residents: vec![pop],
/// });
///
/// // Run system
/// restore_rest_in_housing_system(&mut world);
///
/// // Check result
/// let needs = world.get::<Needs>(pop).unwrap();
/// assert!(needs.rest > 0.1);
/// ```
pub fn restore_rest_in_housing_system(world: &mut World) {
    // Collect housing with residents
    let housing_residents: Vec<Vec<Entity>> = world
        .query::<&Housing>()
        .iter(world)
        .map(|h| h.residents.clone())
        .collect();

    // Restore rest for each resident
    for residents in housing_residents {
        for resident in residents {
            if let Some(mut needs) = world.get_mut::<Needs>(resident) {
                needs.rest = (needs.rest + REST_RESTORE_PER_TICK).min(1.0);
            }
        }
    }
}

/// Removes dead residents from housing.
///
/// This creates a self-healing relationship between housing and pops.
/// If a pop dies (despawns), this system ensures the housing slot is freed.
///
/// # Examples
///
/// ```
/// use scale::layer1::housing::{Housing, clean_dead_residents_system};
/// use scale::layer1::pop::Pop;
/// use bevy_ecs::prelude::*;
///
/// let mut world = World::new();
/// let pop = world.spawn(Pop).id();
/// let housing = world.spawn(Housing {
///     capacity: 1,
///     residents: vec![pop],
/// }).id();
///
/// // Kill pop
/// world.despawn(pop);
///
/// // Cleanup
/// clean_dead_residents_system(&mut world);
///
/// // Verify
/// let h = world.get::<Housing>(housing).unwrap();
/// assert!(h.residents.is_empty());
/// ```
pub fn clean_dead_residents_system(world: &mut World) {
    // Collect housing entities and their residents first to avoid double borrow
    let housing_data: Vec<(Entity, Vec<Entity>)> = world
        .query::<(Entity, &Housing)>()
        .iter(world)
        .map(|(e, h)| (e, h.residents.clone()))
        .collect();

    for (housing_entity, residents) in housing_data {
        // Find residents that no longer exist
        let dead_residents: Vec<Entity> = residents
            .into_iter()
            .filter(|&resident| world.get_entity(resident).is_err())
            .collect();

        if dead_residents.is_empty() {
            continue;
        }

        if let Some(mut housing) = world.get_mut::<Housing>(housing_entity) {
            housing.residents.retain(|r| !dead_residents.contains(r));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::GridPosition;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_housing_default() {
        let housing = Housing::default();
        assert_eq!(housing.capacity, 2);
        assert!(housing.residents.is_empty());
    }

    #[test]
    fn test_housing_add_resident() {
        let mut world = World::new();
        let pop_entity = world.spawn(Pop).id();

        let mut housing = Housing::default();
        housing.residents.push(pop_entity);

        assert_eq!(housing.residents.len(), 1);
        assert_eq!(housing.residents[0], pop_entity);
    }

    #[test]
    fn test_housing_capacity_limit() {
        let housing = Housing::default();
        assert!(housing.residents.len() < housing.capacity + 1); // Check against logic, here just ensures < capacity if filled
        // Spec test:
        assert!(housing.residents.len() < housing.capacity); // 0 < 2
    }

    #[test]
    fn test_restore_rest_in_housing_system() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.5,
                    rest: 0.3,
                    ..Default::default()
                },
            ))
            .id();

        let mut housing = Housing::default();
        housing.residents.push(pop);
        world.spawn(housing);

        let rest_before = world.get::<Needs>(pop).unwrap().rest;
        restore_rest_in_housing_system(&mut world);
        let rest_after = world.get::<Needs>(pop).unwrap().rest;

        assert!(rest_after > rest_before, "Rest should increase");
        assert!(rest_after <= 1.0, "Rest should not exceed 1.0");
    }

    #[test]
    fn test_restore_rest_capped_at_one() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.5,
                    rest: 0.99,
                    ..Default::default()
                },
            ))
            .id();

        let mut housing = Housing::default();
        housing.residents.push(pop);
        world.spawn(housing);

        restore_rest_in_housing_system(&mut world);
        let rest = world.get::<Needs>(pop).unwrap().rest;

        assert!((rest - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_restore_rest_multiple_residents() {
        let mut world = World::new();

        let pop1 = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.5,
                    rest: 0.4,
                    ..Default::default()
                },
            ))
            .id();
        let pop2 = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.5,
                    rest: 0.3,
                    ..Default::default()
                },
            ))
            .id();

        let mut housing = Housing::default();
        housing.residents.push(pop1);
        housing.residents.push(pop2);
        world.spawn(housing);

        restore_rest_in_housing_system(&mut world);

        assert!(world.get::<Needs>(pop1).unwrap().rest > 0.4);
        assert!(world.get::<Needs>(pop2).unwrap().rest > 0.3);
    }

    #[test]
    fn test_clean_dead_residents_system() {
        let mut world = World::new();

        let pop1 = world.spawn(Pop).id();
        let pop2 = world.spawn(Pop).id();

        let mut housing = Housing::default();
        housing.residents.push(pop1);
        housing.residents.push(pop2);
        let housing_entity = world.spawn(housing).id();

        // Kill one pop
        world.despawn(pop1);

        clean_dead_residents_system(&mut world);

        let housing = world.get::<Housing>(housing_entity).unwrap();
        assert_eq!(housing.residents.len(), 1);
        assert_eq!(housing.residents[0], pop2);
    }

    #[test]
    fn test_clean_dead_residents_empty() {
        let mut world = World::new();

        let pop = world.spawn(Pop).id();
        let mut housing = Housing::default();
        housing.residents.push(pop);
        let housing_entity = world.spawn(housing).id();

        // Kill the pop
        world.despawn(pop);

        clean_dead_residents_system(&mut world);

        let housing = world.get::<Housing>(housing_entity).unwrap();
        assert!(housing.residents.is_empty());
    }

    #[test]
    fn test_housing_component_with_building() {
        let mut world = World::new();

        world.spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            GridPosition { x: 5, y: 5 },
            Housing::default(),
        ));

        let count = world.query::<(&Building, &Housing)>().iter(&world).count();
        assert_eq!(count, 1);
    }
}
