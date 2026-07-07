use bevy::prelude::*;
use crate::layer3::economy::{Faction, FactionId};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Component)]
pub struct LanguageBarrier {
    pub understanding_level: u8, // 0 to 100
}

#[derive(Clone, PartialEq, Eq)]
pub enum ResponseType {
    AcceptFuel,
    Reject,
}

#[derive(Clone, PartialEq, Eq)]
pub enum IntentType {
    GiftFuel,
    Insult,
}

#[derive(Event)]
pub struct MessageResponseEvent {
    pub faction_id: u32,
    pub chosen_response: ResponseType,
    pub true_intent: IntentType,
}

#[derive(Component)]
pub struct DiplomaticState {
    pub relations: i32,
}

pub fn generate_cipher_message(true_message: &str, barrier: &LanguageBarrier) -> String {
    if barrier.understanding_level >= 100 {
        return true_message.to_string();
    }


    let words: Vec<&str> = true_message.split_whitespace().collect();
    let mut result = Vec::new();

    for word in words {
        let mut hasher = DefaultHasher::new();
        word.hash(&mut hasher);
        let hash_val = hasher.finish();

        // Deterministic threshold check using the hash
        if (hash_val % 100) >= (barrier.understanding_level as u64) {
            result.push("****".to_string());
        } else {
            result.push(word.to_string());
        }
    }

    result.join(" ")
}

pub fn process_alien_responses_system(
    mut events: EventReader<MessageResponseEvent>,
    mut query: Query<(&mut DiplomaticState, &LanguageBarrier, &Faction)>,
) {
    for event in events.read() {
        for (mut dip_state, _barrier, faction) in query.iter_mut() {
            if faction.id.0 == event.faction_id {
                if event.chosen_response == ResponseType::AcceptFuel && event.true_intent == IntentType::Insult {
                    dip_state.relations -= 20;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cipher_message_generation() {
        let message = String::from("GIFT BIO_SLUDGE");
        let barrier = LanguageBarrier { understanding_level: 0 };

        let ciphered = generate_cipher_message(&message, &barrier);

        assert_ne!(message, ciphered);
        assert!(!ciphered.contains("GIFT"));
    }

    #[test]
    fn test_translation_progress_unlocks_words() {
        let message = String::from("GIFT BIO_SLUDGE");

        let barrier = LanguageBarrier { understanding_level: 50 };
        let ciphered = generate_cipher_message(&message, &barrier);

        assert!(ciphered.contains("GIFT") || ciphered.contains("BIO_SLUDGE") || ciphered.contains("****"));
    }

    #[test]
    fn test_misunderstanding_triggers_diplomatic_incident() {
        let mut world = World::new();
        let faction_id = crate::layer3::economy::FactionId(1);

        world.spawn((
            crate::layer3::economy::Faction { id: faction_id },
            DiplomaticState { relations: 50 },
            LanguageBarrier { understanding_level: 10 }
        ));

        let mut events = Events::<MessageResponseEvent>::default();
        events.send(MessageResponseEvent {
            faction_id: 1,
            chosen_response: ResponseType::AcceptFuel,
            true_intent: IntentType::Insult,
        });
        world.insert_resource(events);

        let mut schedule = Schedule::default();
        schedule.add_systems(process_alien_responses_system);
        schedule.run(&mut world);

        let dip_state = world.query::<&DiplomaticState>().single(&world);
        assert!(dip_state.relations < 50, "Relations should drop due to misunderstanding");
    }
}
