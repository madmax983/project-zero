use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::health::Health;
use bevy_ecs::prelude::*;

/// Policy controlling how medical treatment is prioritized.
#[derive(Resource, Default, Debug, PartialEq, Eq, Copy, Clone)]
pub enum MedicalPolicy {
    /// Treat everyone equally (FIFO or random).
    #[default]
    SaveEveryone,
    /// Prioritize productive workers (those with a Job).
    WorkersFirst,
    /// Prioritize those with the lowest health percentage.
    Triage,
}

/// Event emitted when a patient is treated.
#[derive(Event, Debug, Clone)]
pub struct PatientTreated {
    /// The patient being healed.
    pub patient: Entity,
    /// The hospital where treatment occurred.
    pub hospital: Entity,
    /// The amount of health restored.
    pub amount: f32,
}

/// Component indicating a building is a hospital that can heal patients.
#[derive(Component)]
pub struct Hospital {
    /// The amount of health restored per tick to patients.
    pub healing_rate: f32,
    /// The maximum total health that can be dispensed per tick (capacity).
    /// Simulates limited beds/medicine.
    pub max_healing_per_tick: f32,
}

impl Default for Hospital {
    fn default() -> Self {
        Self {
            healing_rate: 0.5,         // 0.5 HP per tick per patient
            max_healing_per_tick: 5.0, // Default cap (e.g. 10 patients)
        }
    }
}

use crate::layer1::pop::Job;
use std::collections::HashMap;

/// System to heal pops assigned to a hospital.
#[allow(clippy::collapsible_if)]
pub fn healing_system(world: &mut World) {
    let policy = world
        .get_resource::<MedicalPolicy>()
        .copied()
        .unwrap_or_default();

    // Group patients by hospital
    // Key: Hospital Entity, Value: List of (Patient Entity, HP%, HasJob)
    let mut hospitals: HashMap<Entity, Vec<(Entity, f32, bool)>> = HashMap::new();

    {
        // Query for pops assigned as Patient
        let mut query = world.query::<(Entity, &Health, &AssignedTo, Option<&Job>)>();

        for (entity, health, assigned, job) in query.iter(world) {
            if assigned.assignment_type == AssignmentType::Patient && health.current < health.max {
                let hp_percent = if health.max > 0.0 {
                    health.current / health.max
                } else {
                    0.0
                };
                let has_job = job.is_some();

                hospitals
                    .entry(assigned.entity)
                    .or_default()
                    .push((entity, hp_percent, has_job));
            }
        }
    }

    let mut updates: Vec<(Entity, f32, Entity)> = Vec::new();

    // Process each hospital
    for (hospital_ent, mut patients) in hospitals {
        let Some(hospital) = world.get::<Hospital>(hospital_ent) else {
            continue;
        };

        // Check Power
        if let Some(power) = world.get::<crate::layer1::energy::PowerConsumer>(hospital_ent) {
            if !power.active {
                continue;
            }
        }

        let zone_bonus = crate::layer1::zone::get_zone_bonus(world, hospital_ent);
        let rate = hospital.healing_rate * (1.0 + zone_bonus);
        let mut capacity = hospital.max_healing_per_tick;

        // Apply Policy
        match policy {
            MedicalPolicy::WorkersFirst => {
                // Filter out non-workers if we have workers waiting?
                // Or strict priority? "Ignores unemployed" implies we don't treat them if policy is active.
                // Spec says: "WorkersFirst policy ignores unemployed".
                patients.retain(|(_, _, has_job)| *has_job);
            }
            MedicalPolicy::Triage => {
                // Sort by Health % (Ascending) - sickest first
                patients.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            }
            MedicalPolicy::SaveEveryone => {
                // No sort needed (FIFO)
            }
        }

        // Distribute Healing
        for (patient, _, _) in patients {
            if capacity <= 0.001 {
                break;
            } // Epsilon check

            // Each patient consumes 'rate' amount of capacity?
            // Or capacity is total HP dispensed? Spec says "max_healing_per_tick".
            // If rate is 0.5, we give 0.5.

            let amount = rate.min(capacity);
            updates.push((patient, amount, hospital_ent));
            capacity -= amount;
        }
    }

    // Apply updates
    for (entity, amount, hospital) in updates {
        if let Some(mut health) = world.get_mut::<Health>(entity) {
            health.current = (health.current + amount).min(health.max);

            // Emit event
            if amount > 0.0 {
                // We use resource_mut because we are in an exclusive system.
                // We must ensure the resource exists to avoid panic, though it should exist in simulation.
                if let Some(mut events) = world.get_resource_mut::<Events<PatientTreated>>() {
                    events.send(PatientTreated {
                        patient: entity,
                        hospital,
                        amount,
                    });
                }
            }
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
                Hospital {
                    healing_rate: 1.0,
                    ..Default::default()
                },
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
                Hospital {
                    healing_rate: 10.0,
                    ..Default::default()
                },
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
            .spawn((
                Hospital {
                    healing_rate: 10.0,
                    ..Default::default()
                },
                GridPosition { x: 0, y: 0 },
            ))
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
                Hospital {
                    healing_rate: 1.0,
                    ..Default::default()
                },
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
