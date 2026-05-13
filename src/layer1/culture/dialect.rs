use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::entities::pop::Pop;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
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
        if event.importance == EventImportance::Major {
            let lower_text = event.text.to_lowercase();
            // A more procedural approach: check for different disaster keywords
            if lower_text.contains("fire") {
                dialect_manager.slang_dictionary.insert(
                    "fire".to_string(),
                    SlangEntry { usage: SlangUsage::Curse, weight: 1.0 },
                );
            } else if lower_text.contains("starve") || lower_text.contains("starvation") {
                dialect_manager.slang_dictionary.insert(
                    "dust".to_string(),
                    SlangEntry { usage: SlangUsage::Curse, weight: 1.0 },
                );
            } else if lower_text.contains("bountiful") || lower_text.contains("harvest") {
                dialect_manager.slang_dictionary.insert(
                    "green".to_string(),
                    SlangEntry { usage: SlangUsage::Praise, weight: 1.0 },
                );
            }
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
        let mut app = bevy_app::App::new();
        app.add_event::<AddChronicleEvent>();
        app.init_resource::<DialectManager>();
        app.add_systems(bevy_app::Update, process_chronicle_events_for_dialect);

        app.world_mut().send_event(AddChronicleEvent {
            importance: EventImportance::Major,
            text: "The Great Fire has started.".to_string(),
        });

        app.update();

        let dialect = app.world().resource::<DialectManager>();
        assert!(dialect.slang_dictionary.contains_key("fire"));
        assert_eq!(dialect.slang_dictionary.get("fire").unwrap().usage, SlangUsage::Curse);
    }

    #[test]
    fn test_procedural_extraction_starvation() {
        let mut app = bevy_app::App::new();
        app.add_event::<AddChronicleEvent>();
        app.init_resource::<DialectManager>();
        app.add_systems(bevy_app::Update, process_chronicle_events_for_dialect);

        app.world_mut().send_event(AddChronicleEvent {
            importance: EventImportance::Major,
            text: "Mass starvation sweeps the colony.".to_string(),
        });

        app.update();

        let dialect = app.world().resource::<DialectManager>();
        assert!(dialect.slang_dictionary.contains_key("dust"));
        assert_eq!(dialect.slang_dictionary.get("dust").unwrap().usage, SlangUsage::Curse);
    }

    #[test]
    fn test_new_pop_adopts_colony_dialect() {
        let mut app = bevy_app::App::new();
        app.init_resource::<DialectManager>();
        app.add_systems(bevy_app::Update, initialize_pop_dialect);

        let mut dialect = app.world_mut().resource_mut::<DialectManager>();
        dialect.slang_dictionary.insert(
            "rust".to_string(),
            SlangEntry { usage: SlangUsage::Praise, weight: 1.0 }
        );

        let pop_id = app.world_mut().spawn(Pop).id();

        app.update();

        let pop_dialect = app.world().get::<PersonalDialect>(pop_id).unwrap();
        assert!(pop_dialect.vocabulary.contains(&"rust".to_string()));
    }

    #[test]
    fn test_slang_decays_over_time_without_reinforcement() {
        let mut app = bevy_app::App::new();
        app.init_resource::<DialectManager>();
        app.insert_resource(SimulationTime { tick: 0, speed: crate::shared::time::SimSpeed::Normal });
        app.add_systems(bevy_app::Update, decay_slang_weight);

        let mut dialect = app.world_mut().resource_mut::<DialectManager>();
        dialect.slang_dictionary.insert(
            "old_slang".to_string(),
            SlangEntry { usage: SlangUsage::Curse, weight: 0.5 }
        );

        app.world_mut().resource_mut::<SimulationTime>().tick = 100_000;
        app.update();

        let dialect = app.world().resource::<DialectManager>();
        assert!(dialect.slang_dictionary.get("old_slang").map_or(true, |entry| entry.weight < 0.5));
    }
}
