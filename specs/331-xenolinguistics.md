# Specification: Xenolinguistics

## 1. Overview
**Layer:** 3 (Diplomacy/Galaxy)
**Fantasy:** First Contact is confusing, terrifying, and hilarious.
**Mechanic:** You don't start with a universal translator. Alien messages appear as cipher text. Interaction reveals "Concepts". Guessing/Translating wrong leads to diplomatic incidents.
**Emergence:** You accept a gift of "Bio-Sludge" thinking it's fuel. It's actually a grave insult. War starts.
**Tension:** Guess the meaning now (risky) or wait for more data (missed opportunity)?

This feature introduces a `LanguageBarrier` and `CipherMessage` system where incoming messages from undiscovered factions are masked. Players accumulate `TranslationData` to slowly map alien concepts to known concepts.

## 2. Dependencies
- `FactionId` (Base faction tracking)
- `ResourceItem` or Cargo mechanics (for gifts/trades)
- `NotificationEvent` (for incoming messages)
- `DiplomaticState` (from layer 3 diplomacy)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_cipher_message_generation() {
        let mut world = World::new();
        let message = String::from("GIFT BIO_SLUDGE");
        let barrier = LanguageBarrier { understanding_level: 0 };

        let ciphered = generate_cipher_message(&message, &barrier);

        assert_ne!(message, ciphered);
        assert!(!ciphered.contains("GIFT"));
    }

    #[test]
    fn test_translation_progress_unlocks_words() {
        let mut world = World::new();
        let message = String::from("GIFT BIO_SLUDGE");

        // 50% understanding should reveal some structure or words
        let barrier = LanguageBarrier { understanding_level: 50 };
        let ciphered = generate_cipher_message(&message, &barrier);

        // It might not be identical, but it should reveal "GIFT" or "BIO_SLUDGE"
        // depending on deterministic translation logic.
        assert!(ciphered.contains("GIFT") || ciphered.contains("BIO_SLUDGE") || ciphered.contains("****"));
    }

    #[test]
    fn test_misunderstanding_triggers_diplomatic_incident() {
        let mut world = World::new();
        let faction_id = FactionId(1);

        world.spawn((
            Faction { id: faction_id },
            DiplomaticState { relations: 50 },
            LanguageBarrier { understanding_level: 10 }
        ));

        let mut events = Events::<MessageResponseEvent>::default();
        // Player chose "ACCEPT_FUEL" when the true intent was "INSULT_BIO_SLUDGE"
        events.send(MessageResponseEvent {
            faction_id,
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
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;

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

    // Minimal green: just return asterisks if understanding is 0
    if barrier.understanding_level == 0 {
        return "*** ********".to_string();
    }

    // Partial: Reveal words deterministically based on length/hash or just random for MVP
    let words: Vec<&str> = true_message.split_whitespace().collect();
    let mut result = Vec::new();

    for word in words {
        // Simplified: 50% chance to obscure if understanding is 50
        if barrier.understanding_level < 50 {
            result.push("****");
        } else {
            result.push(word);
        }
    }

    result.join(" ")
}

pub fn process_alien_responses_system(
    mut events: EventReader<MessageResponseEvent>,
    mut query: Query<(&mut DiplomaticState, &LanguageBarrier)>,
) {
    for event in events.read() {
        // Find matching faction (simplification: assume single for green phase)
        for (mut dip_state, _barrier) in query.iter_mut() {
            if event.chosen_response == ResponseType::AcceptFuel && event.true_intent == IntentType::Insult {
                dip_state.relations -= 20; // Incident!
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Word Hashing:** Instead of random chance, use a deterministic hash of each word mapped to the faction's dictionary, so `GIFT` always translates to `ZORG` until unlocked.
- **Data Structure:** Create a `FactionDictionary` component storing mapped concepts/words.
- **UI Integration:** The ciphered text should be rendered in the comms UI. Hovering could show "Translation Confidence: X%".

## 6. Acceptance Criteria
- [ ] `generate_cipher_message` correctly obscures text based on `understanding_level`.
- [ ] Responding incorrectly based on misunderstood intent lowers diplomatic relations.
- [ ] `process_alien_responses_system` correctly evaluates responses against true intent.
- [ ] All RED phase tests pass.
- [ ] Coverage >= 85%.

## 7. Technical Guidance
- **Dictionaries:** Keep dictionaries small and conceptual (e.g., `[GIFT, THREAT, TRADE, FUEL, BIO_SLUDGE]`) rather than mapping full English dictionaries.
- **Discovery:** `understanding_level` should slowly increase via a background `research_xenolinguistics_system` or by successful safe interactions.

## 8. Questions
*Builder: Add any questions here.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
