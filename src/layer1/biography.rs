//! Experimental module for Pop biographies.
//!
//! Tracks significant life events for individual pops.

use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::building::Building;
use crate::layer1::pop::Pop;
use crate::shared::narrative::NarrativeGenerator;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

/// A single event in a Pop's life.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BiographyEvent {
    /// The tick when the event occurred.
    pub tick: u64,
    /// The text description of the event.
    pub text: String,
}

/// Component storing the history of a Pop.
#[derive(Component, Default, Debug, Clone)]
pub struct Biography {
    /// List of life events.
    pub events: Vec<BiographyEvent>,
}

impl Biography {
    /// Add a new event to the biography.
    pub fn add_event(&mut self, tick: u64, text: String) {
        self.events.push(BiographyEvent { tick, text });
    }
}

/// System to monitor and record biography events.
pub fn biography_monitor_system(
    time: Res<SimulationTime>,
    new_pops: Query<Entity, (With<Pop>, Without<Biography>)>,
    mut bio_pops: Query<(Entity, &mut Biography, &AssignedTo)>,
    buildings: Query<&Building>,
    mut commands: Commands,
    generator: Res<NarrativeGenerator>,
) {
    let current_tick = time.tick;

    // 1. Initialize new Pops
    for entity in &new_pops {
        let arrival = generator
            .get_random_fragment("POP_ARRIVAL_METHOD")
            .map_or_else(
                || "Joined the colony.".to_string(),
                |method| format!("Arrived {method}. Joined the colony."),
            );
        commands.entity(entity).insert(Biography {
            events: vec![BiographyEvent {
                tick: current_tick,
                text: arrival,
            }],
        });
    }

    // 2. Monitor Assignments
    for (_, mut bio, assigned) in &mut bio_pops {
        let target_name = buildings
            .get(assigned.entity)
            .map_or("Unknown Building", |building| {
                building.building_type.label()
            });

        let event_text = match assigned.assignment_type {
            AssignmentType::FarmWorker => format!("Started working at {target_name}."),
            AssignmentType::HousingResident => format!("Moved into {target_name}."),
            AssignmentType::TavernVisitor => format!("Visited {target_name} to socialize."),
            AssignmentType::LibraryWorker => format!("Started research at {target_name}."),
            AssignmentType::Patient => format!("Admitted to {target_name} for treatment."),
            AssignmentType::Funeral => format!("Attending funeral at {target_name}."),
            AssignmentType::ObservatoryWorker => format!("Observing the cosmos at {target_name}."),
            AssignmentType::Miner => format!("Started mining at {target_name}."),
            AssignmentType::Hauler => format!("Started hauling at {target_name}."),
            AssignmentType::Builder => format!("Started building at {target_name}."),
            AssignmentType::Crafter => format!("Started crafting at {target_name}."),
            AssignmentType::Guard => format!("Started guarding at {target_name}."),
            AssignmentType::Engineer => format!("Started engineering at {target_name}."),
            AssignmentType::Doctor => format!("Started medical practice at {target_name}."),
            AssignmentType::Merchant => format!("Started trading at {target_name}."),
            AssignmentType::Scientist => format!("Started scientific research at {target_name}."),
            AssignmentType::Artist => format!("Started creating art at {target_name}."),
            AssignmentType::Governor => format!("Assumed governorship at {target_name}."),
            AssignmentType::Administrator => format!("Started administration at {target_name}."),
            AssignmentType::Surgery => format!("Undergoing surgery at {target_name}."),
        };

        // Avoid duplicate consecutive events
        if let Some(last) = bio.events.last()
            && last.text == event_text
        {
            continue;
        }

        bio.add_event(current_tick, event_text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::BuildingType;
    use crate::shared::narrative::NarrativeGenerator;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_biography_initialization() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(NarrativeGenerator::from_embedded());

        let pop = world.spawn(Pop).id();

        world.run_system_once(biography_monitor_system).unwrap();

        let bio = world
            .get::<Biography>(pop)
            .expect("Biography should be added");
        assert_eq!(bio.events.len(), 1);
        assert!(
            bio.events[0].text.contains("colony"),
            "Should mention the colony"
        );
    }

    #[test]
    fn test_biography_records_assignment() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(NarrativeGenerator::from_embedded());

        // Create a farm
        let farm = world
            .spawn(Building {
                building_type: BuildingType::Farm,
            })
            .id();

        // Create a pop with Biography already (simulating existing pop)
        let pop = world
            .spawn((
                Pop,
                Biography::default(),
                AssignedTo {
                    entity: farm,
                    assignment_type: AssignmentType::FarmWorker,
                },
            ))
            .id();

        world.run_system_once(biography_monitor_system).unwrap();

        let bio = world.get::<Biography>(pop).unwrap();
        assert!(
            bio.events
                .iter()
                .any(|e| e.text.contains("Started working at Farm"))
        );
    }

    #[test]
    fn test_biography_prevents_duplicate_events() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(NarrativeGenerator::from_embedded());

        let farm = world
            .spawn(Building {
                building_type: BuildingType::Farm,
            })
            .id();

        let mut bio = Biography::default();
        bio.add_event(0, "Started working at Farm.".to_string());

        let pop = world
            .spawn((
                Pop,
                bio,
                AssignedTo {
                    entity: farm,
                    assignment_type: AssignmentType::FarmWorker,
                },
            ))
            .id();

        world.run_system_once(biography_monitor_system).unwrap();

        let bio_after = world.get::<Biography>(pop).unwrap();
        assert_eq!(bio_after.events.len(), 1, "Should not add duplicate event");
    }
}
