use bevy_ecs::prelude::*;
use bevy::utils::HashMap;
use crate::layer1::entities::pop::Pop;
use crate::layer1::core::chronicle::AddChronicleEvent;
use crate::shared::time::SimulationTime;

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
    pub weight: f32, // How commonly it is used
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
        if event.text.contains("Fire") {
            dialect_manager.slang_dictionary.insert(
                "fire".to_string(),
                SlangEntry { usage: SlangUsage::Curse, weight: 1.0 },
            );
        } else if event.text.contains("Rust") {
            dialect_manager.slang_dictionary.insert(
                "rust".to_string(),
                SlangEntry { usage: SlangUsage::Curse, weight: 1.0 },
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
        let mut vocab = Vec::new();
        for (word, entry) in dialect_manager.slang_dictionary.iter() {
            if entry.weight > 0.0 {
                vocab.push(word.clone());
            }
        }
        commands.entity(entity).insert(PersonalDialect { vocabulary: vocab });
    }
}

pub fn decay_slang_weight(
    mut dialect_manager: ResMut<DialectManager>,
    time: Res<SimulationTime>,
) {
    // Simplified decay logic based on time ticks
    if time.tick.is_multiple_of(1000) && time.tick > 0 {
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
    use bevy::MinimalPlugins;
    use bevy_app::{App, Update};
    use crate::layer1::core::chronicle::EventImportance;

    #[test]
    fn test_event_creates_dialect_slang() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<DialectManager>();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(Update, process_chronicle_events_for_dialect);

        // Trigger a major historical event
        app.world_mut().send_event(AddChronicleEvent {
            text: "The Great Fire".to_string(),
            importance: EventImportance::Major,
        });

        app.update();

        let dialect = app.world().resource::<DialectManager>();
        // High severity fire should create slang related to 'Fire'
        assert!(dialect.slang_dictionary.contains_key("fire"));
        assert_eq!(dialect.slang_dictionary.get("fire").unwrap().usage, SlangUsage::Curse);
    }

    #[test]
    fn test_new_pop_adopts_colony_dialect() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<DialectManager>();
        app.add_systems(Update, initialize_pop_dialect);

        let mut dialect = app.world_mut().resource_mut::<DialectManager>();
        dialect.slang_dictionary.insert(
            "rust".to_string(),
            SlangEntry { usage: SlangUsage::Praise, weight: 1.0 }
        );

        // Spawn a new pop
        let pop_id = app.world_mut().spawn(Pop).id();

        app.update();

        // The pop should have a personal dialect that incorporates the colony's prevailing slang
        let pop_dialect = app.world().get::<PersonalDialect>(pop_id).unwrap();
        assert!(pop_dialect.vocabulary.contains(&"rust".to_string()));
    }

    #[test]
    fn test_slang_decays_over_time_without_reinforcement() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<DialectManager>();
        app.insert_resource(SimulationTime { tick: 0, ..Default::default() });
        app.add_systems(Update, decay_slang_weight);

        let mut dialect = app.world_mut().resource_mut::<DialectManager>();
        dialect.slang_dictionary.insert(
            "old_slang".to_string(),
            SlangEntry { usage: SlangUsage::Curse, weight: 0.5 }
        );

        // Advance time significantly
        app.world_mut().resource_mut::<SimulationTime>().tick = 1000;
        app.update();

        let dialect = app.world().resource::<DialectManager>();
        // Weight should decrease
        assert!(dialect.slang_dictionary.get("old_slang").is_none_or(|entry| entry.weight < 0.5));
    }
}
