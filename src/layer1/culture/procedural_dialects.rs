use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::entities::pop::Pop;
use crate::shared::time::SimulationTime;
use bevy::prelude::*;
use bevy_utils::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlangUsage {
    Curse,
    Praise,
    Greeting,
    Warning,
}

#[derive(Debug, Clone)]
pub struct SlangEntry {
    pub usage: SlangUsage,
    pub weight: f32,
}

#[derive(Resource, Default)]
pub struct DialectManager {
    pub slang_dictionary: HashMap<String, SlangEntry>,
}

#[derive(Component, Default)]
pub struct PersonalDialect {
    pub vocabulary: Vec<String>,
}

pub fn process_chronicle_events_for_dialect(
    mut events: EventReader<AddChronicleEvent>,
    mut dialect_manager: ResMut<DialectManager>,
) {
    for event in events.read() {
        if (event.importance == EventImportance::Major
            || event.importance == EventImportance::Legendary)
            && event.text.to_lowercase().contains("fire")
        {
            dialect_manager.slang_dictionary.insert(
                "fire".to_string(),
                SlangEntry {
                    usage: SlangUsage::Curse,
                    weight: 1.0,
                },
            );
        }
    }
}

pub fn initialize_pop_dialect(
    mut commands: Commands,
    query: Query<Entity, Added<Pop>>,
    dialect_manager: Res<DialectManager>,
) {
    for entity in query.iter() {
        let mut vocab: Vec<String> = Vec::new();
        for (word, entry) in dialect_manager.slang_dictionary.iter() {
            if entry.weight > 0.0 {
                vocab.push(word.clone());
            }
        }
        commands
            .entity(entity)
            .insert(PersonalDialect { vocabulary: vocab });
    }
}

pub fn decay_slang_weight(mut dialect_manager: ResMut<DialectManager>, time: Res<SimulationTime>) {
    if time.tick.is_multiple_of(1000) {
        for entry in dialect_manager.slang_dictionary.values_mut() {
            entry.weight -= 0.01;
            if entry.weight < 0.0 {
                entry.weight = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creates_dialect_slang() {
        let mut world = World::new();
        world.init_resource::<DialectManager>();
        world.init_resource::<Events<AddChronicleEvent>>();

        world
            .resource_mut::<Events<AddChronicleEvent>>()
            .send(AddChronicleEvent {
                text: "The Great Fire".to_string(),
                importance: EventImportance::Legendary,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_chronicle_events_for_dialect);
        schedule.run(&mut world);

        let dialect = world.resource::<DialectManager>();
        assert!(dialect.slang_dictionary.contains_key("fire"));
        assert_eq!(
            dialect.slang_dictionary.get("fire").unwrap().usage,
            SlangUsage::Curse
        );
    }

    #[test]
    fn test_new_pop_adopts_colony_dialect() {
        let mut world = World::new();
        world.init_resource::<DialectManager>();

        let mut dialect = world.resource_mut::<DialectManager>();
        dialect.slang_dictionary.insert(
            "rust".to_string(),
            SlangEntry {
                usage: SlangUsage::Praise,
                weight: 1.0,
            },
        );

        let pop_id = world.spawn(Pop).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(initialize_pop_dialect);
        schedule.run(&mut world);

        let pop_dialect = world.get::<PersonalDialect>(pop_id).unwrap();
        assert!(pop_dialect.vocabulary.contains(&"rust".to_string()));
    }

    #[test]
    fn test_slang_decays_over_time_without_reinforcement() {
        let mut world = World::new();
        world.init_resource::<DialectManager>();
        world.insert_resource(SimulationTime {
            tick: 100_000,
            speed: crate::shared::time::SimSpeed::Normal,
        });

        let mut dialect = world.resource_mut::<DialectManager>();
        dialect.slang_dictionary.insert(
            "old_slang".to_string(),
            SlangEntry {
                usage: SlangUsage::Curse,
                weight: 0.5,
            },
        );

        let mut schedule = Schedule::default();
        schedule.add_systems(decay_slang_weight);
        schedule.run(&mut world);

        let dialect = world.resource::<DialectManager>();
        assert!(dialect
            .slang_dictionary
            .get("old_slang")
            .is_none_or(|entry| entry.weight < 0.5));
    }
}
