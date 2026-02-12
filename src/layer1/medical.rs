use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::health::Health;
use bevy_ecs::prelude::*;

/// Component indicating a building is a hospital that can heal patients.
#[derive(Component)]
pub struct Hospital {
    /// The amount of health restored per tick to patients.
    pub healing_rate: f32,
}

impl Default for Hospital {
    fn default() -> Self {
        Self { healing_rate: 0.5 } // 0.5 HP per tick
    }
}

/// System to heal pops assigned to a hospital.
pub fn healing_system(world: &mut World) {
    let mut updates: Vec<(Entity, f32)> = Vec::new();

    // Collect updates
    {
        // Query for pops assigned as Patient
        let mut query = world.query::<(Entity, &Health, &AssignedTo)>();

        let mut patients = Vec::new();
        for (entity, health, assigned) in query.iter(world) {
            if assigned.assignment_type == AssignmentType::Patient && health.current < health.max {
                patients.push((entity, assigned.entity));
            }
        }

        for (pop_entity, hospital_entity) in patients {
            if let Some(hospital) = world.get::<Hospital>(hospital_entity) {
                let bonus = crate::layer1::zone::get_zone_bonus(world, hospital_entity);
                updates.push((pop_entity, hospital.healing_rate * (1.0 + bonus)));
            }
        }
    }

    // Apply updates
    for (entity, amount) in updates {
        if let Some(mut health) = world.get_mut::<Health>(entity) {
            health.current = (health.current + amount).min(health.max);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_hospital_component_defaults() {
        let hospital = Hospital::default();
        assert!(hospital.healing_rate > 0.0);
    }

    #[test]
    fn test_healing_system_restores_health() {
        let mut world = World::new();

        // Spawn Hospital
        let hospital_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Hospital,
                },
                Hospital { healing_rate: 1.0 },
            ))
            .id();

        // Spawn Injured Pop assigned to Hospital (as patient)
        let pop_entity = world
            .spawn((
                Health {
                    current: 50.0,
                    max: 100.0,
                },
                AssignedTo {
                    assignment_type: AssignmentType::Patient,
                    entity: hospital_entity,
                },
            ))
            .id();

        // Run system
        healing_system(&mut world);

        // Check health
        let health = world.get::<Health>(pop_entity).unwrap();
        assert!((health.current - 51.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_healing_stops_at_max() {
        let mut world = World::new();

        let hospital_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Hospital,
                },
                Hospital { healing_rate: 10.0 },
            ))
            .id();

        let pop_entity = world
            .spawn((
                Health {
                    current: 95.0,
                    max: 100.0,
                },
                AssignedTo {
                    assignment_type: AssignmentType::Patient,
                    entity: hospital_entity,
                },
            ))
            .id();

        healing_system(&mut world);

        let health = world.get::<Health>(pop_entity).unwrap();
        assert!((health.current - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_assignment_type_patient_exists() {
        let assignment = AssignmentType::Patient;
        assert!(matches!(assignment, AssignmentType::Patient));
    }

    #[test]
    fn test_healing_system_missing_hospital_component() {
        let mut world = World::new();

        // Spawn a building that is NOT a hospital (just an entity with ID)
        let fake_hospital = world.spawn(GridPosition { x: 0, y: 0 }).id();

        // Pop assigned to it
        let pop = world
            .spawn((
                Health {
                    current: 50.0,
                    max: 100.0,
                },
                AssignedTo {
                    assignment_type: AssignmentType::Patient,
                    entity: fake_hospital,
                },
            ))
            .id();

        healing_system(&mut world);

        let health = world.get::<Health>(pop).unwrap();
        assert!(
            (health.current - 50.0).abs() < f32::EPSILON,
            "Health should not change if assigned entity is not a hospital"
        );
    }

    #[test]
    fn test_healing_system_wrong_assignment_type() {
        let mut world = World::new();

        let hospital = world
            .spawn((Hospital { healing_rate: 10.0 }, GridPosition { x: 0, y: 0 }))
            .id();

        // Pop assigned as Worker (not Patient)
        let pop = world
            .spawn((
                Health {
                    current: 50.0,
                    max: 100.0,
                },
                AssignedTo {
                    assignment_type: AssignmentType::FarmWorker, // Wrong type
                    entity: hospital,
                },
            ))
            .id();

        healing_system(&mut world);

        let health = world.get::<Health>(pop).unwrap();
        assert!(
            (health.current - 50.0).abs() < f32::EPSILON,
            "Should not heal if not a Patient"
        );
    }

    #[test]
    fn test_healing_with_zone_bonus() {
        let mut world = World::new();
        let mut zone_grid = crate::layer1::zone::ZoneGrid::new(10, 10);
        zone_grid.set(0, 0, crate::layer1::zone::ZoneType::Hospital);
        world.insert_resource(zone_grid);

        let hospital_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Hospital,
                },
                GridPosition { x: 0, y: 0 },
                Hospital { healing_rate: 1.0 },
            ))
            .id();

        let pop = world
            .spawn((
                Health {
                    current: 50.0,
                    max: 100.0,
                },
                AssignedTo {
                    assignment_type: AssignmentType::Patient,
                    entity: hospital_entity,
                },
            ))
            .id();

        // Base: 1.0. Bonus (Hospital): 0.5. Total: 1.5.
        healing_system(&mut world);
        let health = world.get::<Health>(pop).unwrap();
        assert!((health.current - 51.5).abs() < f32::EPSILON);
    }
}
