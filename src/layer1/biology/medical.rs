//! # Medical Triage and Healing
//!
//! This module manages the recovery of [`crate::layer1::health::Health`] and the treatment of specific
//! afflictions like [`CryoTrauma`] and [`RadiationSickness`].
//!
//! ## The Hospital System
//!
//! Pops injured in the simulation do not heal automatically. They must be assigned
//! as `Patient`s to a [`Hospital`] building. The `healing_system` processes these
//! assignments and restores health based on the hospital's capacity and the
//! global [`MedicalPolicy`].
//!
//! ## Triage
//!
//! Healing capacity is finite. When a hospital reaches its `max_healing_per_tick`,
//! the [`MedicalPolicy`] dictates who gets treated first (e.g., critical patients,
//! or productive workers). Specific afflictions (like [`CryoTrauma`]) consume
//! significantly more healing capacity than standard damage.

#![allow(clippy::too_many_lines, clippy::type_complexity, clippy::doc_markdown)]
use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::cryo_dreams::CryoTrauma;
use crate::layer1::health::Health;
use crate::layer1::radioactive::RadiationSickness;
use bevy_ecs::prelude::*;

/// Policy controlling how medical treatment is prioritized.
///
/// # Examples
///
/// ```
/// use scale::layer1::medical::MedicalPolicy;
///
/// // The default policy treats everyone equally in order of arrival
/// let policy = MedicalPolicy::default();
/// assert_eq!(policy, MedicalPolicy::SaveEveryone);
/// ```
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
///
/// # Examples
///
/// ```
/// use scale::layer1::medical::PatientTreated;
/// use bevy_ecs::prelude::Entity;
///
/// let event = PatientTreated {
///     patient: Entity::PLACEHOLDER,
///     hospital: Entity::PLACEHOLDER,
///     amount: 15.0,
/// };
/// ```
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
///
/// # Examples
///
/// ```
/// use scale::layer1::medical::Hospital;
///
/// // Create a custom hospital that heals slowly but can treat many patients
/// let hospital = Hospital {
///     healing_rate: 0.1,
///     max_healing_per_tick: 20.0,
/// };
/// ```
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
/// Handles Health recovery, as well as CryoTrauma and RadiationSickness treatment.
#[allow(clippy::collapsible_if)]
pub fn healing_system(world: &mut World) {
    let policy = world
        .get_resource::<MedicalPolicy>()
        .copied()
        .unwrap_or_default();

    // Group patients by hospital
    // Key: Hospital Entity, Value: List of (Patient Entity, HP%, HasJob, HasTrauma, HasSickness)
    let mut hospitals: HashMap<Entity, Vec<(Entity, f32, bool, bool, bool)>> = HashMap::new();

    {
        // Query for pops assigned as Patient
        let mut query = world.query::<(
            Entity,
            &Health,
            &AssignedTo,
            Option<&Job>,
            Option<&CryoTrauma>,
            Option<&RadiationSickness>,
        )>();

        for (entity, health, assigned, job, trauma, sickness) in query.iter(world) {
            if assigned.assignment_type != AssignmentType::Patient {
                continue;
            }

            let has_trauma = trauma.is_some();
            let has_sickness = sickness.is_some_and(|s| s.severity > 0.0);
            let needs_healing = health.current < health.max;

            if needs_healing || has_trauma || has_sickness {
                let hp_percent = if health.max > 0.0 {
                    health.current / health.max
                } else {
                    0.0
                };
                let has_job = job.is_some();

                hospitals.entry(assigned.entity).or_default().push((
                    entity,
                    hp_percent,
                    has_job,
                    has_trauma,
                    has_sickness,
                ));
            }
        }
    }

    let mut health_updates: Vec<(Entity, f32, Entity)> = Vec::new();
    let mut trauma_updates: Vec<Entity> = Vec::new();
    let mut sickness_updates: Vec<Entity> = Vec::new();

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
                // Filter out non-workers
                patients.retain(|(_, _, has_job, _, _)| *has_job);
            }
            MedicalPolicy::Triage => {
                // Sort by Health % (Ascending) - sickest first
                patients.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            }
            MedicalPolicy::SaveEveryone => {
                // No sort needed (FIFO)
            }
        }

        // Distribute Healing / Treatment
        for (patient, _, _, has_trauma, has_sickness) in patients {
            if capacity <= 0.001 {
                break;
            }

            // Treat CryoTrauma (Expensive)
            if has_trauma && capacity >= 1.0 {
                trauma_updates.push(patient);
                capacity -= 1.0;
            }

            // Treat Radiation Sickness (Moderate)
            if has_sickness && capacity >= 0.5 {
                sickness_updates.push(patient);
                capacity -= 0.5;
            }

            // Heal Health
            let amount = rate.min(capacity);
            if amount > 0.0 {
                health_updates.push((patient, amount, hospital_ent));
                capacity -= amount;
            }
        }
    }

    // Apply Health Updates
    for (entity, amount, hospital) in health_updates {
        if let Some(mut health) = world.get_mut::<Health>(entity) {
            health.current = (health.current + amount).min(health.max);

            // Emit event
            if amount > 0.0 {
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

    // Apply Trauma Treatment
    for entity in trauma_updates {
        if let Some(mut trauma) = world.get_mut::<CryoTrauma>(entity) {
            trauma.severity -= 0.1;
            if trauma.severity <= 0.0 {
                world.entity_mut(entity).remove::<CryoTrauma>();
            }
        }
    }

    // Apply Sickness Treatment
    for entity in sickness_updates {
        if let Some(mut sick) = world.get_mut::<RadiationSickness>(entity) {
            sick.severity -= 1.0; // Aggressive treatment
            if sick.severity <= 0.0 {
                world.entity_mut(entity).remove::<RadiationSickness>();
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
                    conditions: Vec::new(),
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
        let health = world
            .get::<Health>(pop_entity)
            .expect("Missing resource or component");
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
                    conditions: Vec::new(),
                },
                AssignedTo {
                    assignment_type: AssignmentType::Patient,
                    entity: hospital_entity,
                },
            ))
            .id();

        healing_system(&mut world);

        let health = world
            .get::<Health>(pop_entity)
            .expect("Missing resource or component");
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
                    conditions: Vec::new(),
                },
                AssignedTo {
                    assignment_type: AssignmentType::Patient,
                    entity: fake_hospital,
                },
            ))
            .id();

        healing_system(&mut world);

        let health = world
            .get::<Health>(pop)
            .expect("Missing resource or component");
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
                    conditions: Vec::new(),
                },
                AssignedTo {
                    assignment_type: AssignmentType::FarmWorker, // Wrong type
                    entity: hospital,
                },
            ))
            .id();

        healing_system(&mut world);

        let health = world
            .get::<Health>(pop)
            .expect("Missing resource or component");
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
                    conditions: Vec::new(),
                },
                AssignedTo {
                    assignment_type: AssignmentType::Patient,
                    entity: hospital_entity,
                },
            ))
            .id();

        // Base: 1.0. Bonus (Hospital): 0.5. Total: 1.5.
        healing_system(&mut world);
        let health = world
            .get::<Health>(pop)
            .expect("Missing resource or component");
        assert!((health.current - 51.5).abs() < f32::EPSILON);
    }
}
