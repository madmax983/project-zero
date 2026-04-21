#[cfg(test)]
mod tests {
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::health::Health;
    use crate::layer1::medical::{healing_system, Hospital, MedicalPolicy};
    use crate::layer1::pop::{Job, Pop}; // Job is the new component for employment
    use bevy_ecs::prelude::*;

    #[test]
    fn test_policy_resource_default() {
        let mut world = World::new();
        world.init_resource::<MedicalPolicy>();
        assert_eq!(
            *world.resource::<MedicalPolicy>(),
            MedicalPolicy::SaveEveryone
        );
    }

    #[test]
    fn test_workers_first_policy_ignores_unemployed() {
        let mut world = World::new();
        world.insert_resource(MedicalPolicy::WorkersFirst);

        let hospital = world
            .spawn((
                Building {
                    building_type: BuildingType::Hospital,
                },
                Hospital {
                    healing_rate: 10.0,
                    max_healing_per_tick: 50.0,
                },
            ))
            .id();

        // Employed Pop (Has Job component)
        // Note: Even if AssignedTo is Patient, Job remains
        let farm_entity = world
            .spawn(Building {
                building_type: BuildingType::Farm,
            })
            .id();

        let worker = world
            .spawn((
                Health {
                    current: 50.0,
                    max: 100.0,
                    conditions: Vec::new(),
                },
                AssignedTo {
                    assignment_type: AssignmentType::Patient,
                    entity: hospital,
                },
                Job {
                    workplace: farm_entity,
                    job_type: AssignmentType::FarmWorker,
                },
                Pop,
            ))
            .id();

        let idler = world
            .spawn((
                Health {
                    current: 50.0,
                    max: 100.0,
                    conditions: Vec::new(),
                },
                AssignedTo {
                    assignment_type: AssignmentType::Patient,
                    entity: hospital,
                },
                // No Job component -> Unemployed
                Pop,
            ))
            .id();

        healing_system(&mut world);

        let worker_health = world.get::<Health>(worker).unwrap();
        let idler_health = world.get::<Health>(idler).unwrap();

        assert!(worker_health.current > 50.0, "Worker should be healed");
        assert!(
            (idler_health.current - 50.0).abs() < f32::EPSILON,
            "Idler should NOT be healed under WorkersFirst"
        );
    }

    #[test]
    fn test_triage_policy_prioritizes_lowest_health() {
        let mut world = World::new();
        world.insert_resource(MedicalPolicy::Triage);

        // Hospital with limited output
        let hospital = world
            .spawn((
                Building {
                    building_type: BuildingType::Hospital,
                },
                Hospital {
                    healing_rate: 10.0,
                    max_healing_per_tick: 15.0,
                },
            ))
            .id();

        // Critical Patient (10/100)
        let critical = world
            .spawn((
                Health {
                    current: 10.0,
                    max: 100.0,
                    conditions: Vec::new(),
                },
                AssignedTo {
                    assignment_type: AssignmentType::Patient,
                    entity: hospital,
                },
                Pop,
            ))
            .id();

        // Stable Patient (90/100)
        let stable = world
            .spawn((
                Health {
                    current: 90.0,
                    max: 100.0,
                    conditions: Vec::new(),
                },
                AssignedTo {
                    assignment_type: AssignmentType::Patient,
                    entity: hospital,
                },
                Pop,
            ))
            .id();

        // If max_healing_per_tick is 15, and rate is 10:
        // Critical needs 90, gets 10 (Cost 10). Remaining capacity 5.
        // Stable needs 10, gets 5 (Limited by capacity).

        healing_system(&mut world);

        let crit_health = world.get::<Health>(critical).unwrap();
        let stable_health = world.get::<Health>(stable).unwrap();

        assert!(
            (crit_health.current - 20.0).abs() < f32::EPSILON,
            "Critical should get full healing rate (10.0)"
        );
        assert!(
            (stable_health.current - 95.0).abs() < f32::EPSILON,
            "Stable should get remaining capacity (5.0)"
        );
    }
}
