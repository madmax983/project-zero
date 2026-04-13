use bevy_ecs::prelude::*;
use std::collections::HashMap;

use crate::layer1::memory::{Memories, MemoryType};
use crate::layer1::social::morale::Morale;
use crate::layer1::traits::{Trait, Traits};

#[derive(Resource, Default)]
pub struct DailyChronicle {
    pub entries: Vec<String>,
    pub redacted_truths: HashMap<usize, String>,
}

#[derive(Event)]
pub struct RedactEvent {
    pub entry_index: usize,
}

pub fn process_redactions_system(
    mut events: EventReader<RedactEvent>,
    mut chronicle: ResMut<DailyChronicle>,
) {
    for event in events.read() {
        if let Some(entry) = chronicle.entries.get_mut(event.entry_index) {
            let truth = entry.clone();
            *entry = "[REDACTED]".to_string();
            chronicle.redacted_truths.insert(event.entry_index, truth);
        }
    }
}

pub fn apply_propaganda_effects_system(
    chronicle: Res<DailyChronicle>,
    mut query: Query<(&mut Traits, &Memories, &mut Morale)>,
) {
    for (mut traits, memories, mut morale) in query.iter_mut() {
        let mut is_dissident = false;

        // Check if they witnessed a redacted truth
        for (_, truth) in chronicle.redacted_truths.iter() {
            // Memory system matches enum types. We'll map the bad events to specific types.
            // For example, starvation string mapped to StarvationTrauma.
            let has_memory = if truth.contains("Starvation") {
                memories
                    .items
                    .iter()
                    .any(|m| m.memory_type == MemoryType::StarvationTrauma)
            } else {
                false
            };

            if has_memory {
                traits.add(Trait::Dissident);
                is_dissident = true;
                morale.value = (morale.value - 0.1).max(0.0);
            }
        }

        // If not a dissident, the sanitized paper boosts morale
        if !is_dissident && !chronicle.entries.is_empty() {
            morale.value = (morale.value + 0.05).min(1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redacting_chronicle_hides_event() {
        let mut world = World::new();
        world.init_resource::<DailyChronicle>();
        world.init_resource::<Events<RedactEvent>>();

        // Setup an initial chronicle with a bad event
        let mut chronicle = world.resource_mut::<DailyChronicle>();
        chronicle
            .entries
            .push("Starvation occurred in Sector 4".to_string());

        // Send a redact command
        world
            .resource_mut::<Events<RedactEvent>>()
            .send(RedactEvent { entry_index: 0 });

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_redactions_system);
        schedule.run(&mut world);

        // Verify the entry was redacted
        let updated_chronicle = world.resource::<DailyChronicle>();
        assert!(updated_chronicle.entries[0].contains("[REDACTED]"));
        assert_eq!(
            updated_chronicle.redacted_truths.get(&0).unwrap(),
            "Starvation occurred in Sector 4"
        );
    }

    #[test]
    fn test_witnesses_become_dissidents_when_reading_redacted_truth() {
        let mut world = World::new();

        let mut chronicle = DailyChronicle::default();
        chronicle.entries.push("[REDACTED]".to_string());
        chronicle
            .redacted_truths
            .insert(0, "Starvation occurred in Sector 4".to_string());
        world.insert_resource(chronicle);

        // Populate pop with memory of starvation
        let mut memories = Memories::default();
        memories.add(MemoryType::StarvationTrauma, 100);

        // Spawn a pop who witnessed the event and is reading the paper
        let pop_entity = world
            .spawn((
                Traits::default(),
                memories,
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                }, // They will get angry, not happy
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_propaganda_effects_system);
        schedule.run(&mut world);

        // Verify pop gained Dissident trait and lost morale
        let traits = world.get::<Traits>(pop_entity).unwrap();
        assert!(traits.has(Trait::Dissident));

        let morale = world.get::<Morale>(pop_entity).unwrap();
        assert!((morale.value - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ignorant_pops_gain_morale_from_propaganda() {
        let mut world = World::new();

        let mut chronicle = DailyChronicle::default();
        chronicle.entries.push("[REDACTED]".to_string());
        chronicle
            .redacted_truths
            .insert(0, "Starvation occurred in Sector 4".to_string());
        world.insert_resource(chronicle);

        let memories = Memories::default();

        let pop_entity = world
            .spawn((
                Traits::default(),
                memories,
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_propaganda_effects_system);
        schedule.run(&mut world);

        // Verify pop did NOT gain trait and gained morale
        let traits = world.get::<Traits>(pop_entity).unwrap();
        assert!(!traits.has(Trait::Dissident));

        let morale = world.get::<Morale>(pop_entity).unwrap();
        assert!((morale.value - 0.55).abs() < f32::EPSILON);
    }
}
