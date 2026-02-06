//! Experimental module for Pop biographies.
//!
//! Tracks significant life events for individual pops.

use crate::layer1::building::Building;
use crate::layer1::execution::{AssignedTo, AssignmentType};
use crate::layer1::pop::Pop;
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
#[derive(Component, Default, Debug)]
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
pub fn biography_monitor_system(world: &mut World) {
    let current_tick = world.resource::<SimulationTime>().tick;

    // 1. Initialize new Pops
    let new_pops: Vec<Entity> = world
        .query_filtered::<Entity, (With<Pop>, Without<Biography>)>()
        .iter(world)
        .collect();

    for entity in new_pops {
        world.entity_mut(entity).insert(Biography {
            events: vec![BiographyEvent {
                tick: current_tick,
                text: "Joined the colony.".to_string(),
            }],
        });
    }

    // 2. Monitor Assignments
    let mut events_to_add: Vec<(Entity, String)> = Vec::new();

    let mut query = world.query::<(Entity, &Biography, &AssignedTo)>();
    for (entity, bio, assigned) in query.iter(world) {
        let target_name = world
            .get::<Building>(assigned.entity)
            .map_or("Unknown Building", |building| building.building_type.label());

        let event_text = match assigned.assignment_type {
            AssignmentType::FarmWorker => format!("Started working at {target_name}."),
            AssignmentType::HousingResident => format!("Moved into {target_name}."),
        };

        // Avoid duplicate consecutive events
        if let Some(last) = bio.events.last()
            && last.text == event_text {
                continue;
            }

        events_to_add.push((entity, event_text));
    }

    for (entity, text) in events_to_add {
        if let Some(mut bio) = world.get_mut::<Biography>(entity) {
            bio.add_event(current_tick, text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::BuildingType;

    #[test]
    fn test_biography_initialization() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        let pop = world.spawn(Pop).id();

        biography_monitor_system(&mut world);

        let bio = world
            .get::<Biography>(pop)
            .expect("Biography should be added");
        assert_eq!(bio.events.len(), 1);
        assert_eq!(bio.events[0].text, "Joined the colony.");
    }

    #[test]
    fn test_biography_records_assignment() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

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

        biography_monitor_system(&mut world);

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

        biography_monitor_system(&mut world);

        let bio_after = world.get::<Biography>(pop).unwrap();
        assert_eq!(bio_after.events.len(), 1, "Should not add duplicate event");
    }
}
