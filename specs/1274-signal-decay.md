# 1274: Signal Decay

## 1. Overview
**Layer:** Cross-layer (Layer 2/3)
**Fantasy:** Space is noisy. You have to shout to be heard, and sometimes you mishear the reply.
**Mechanic:** Comms messages (Trade Offers, Threats, Quests) have a "Corruption" % based on distance and interference (Nebulae/Storms). Key words are scrambled (e.g., "Demand 500 [CORRUPTED]"). Players must guess the context or boost signal power to clarify.
**Emergence:** You receive a message: "We are [CORRUPTED]! Send help!" You think it's an ally and send a rescue fleet. It was a pirate trap saying "We are boarding!"
**Tension:** Risk a guess (Speed) vs. Wait for clarity (Safety/Cost).

## 2. Dependencies
- Core `Message` or `CommsEvent` system.
- Distance calculation between nodes in Layer 2/3.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_message_corruption_based_on_distance() {
        let mut app = App::new();
        app.add_systems(Update, process_signal_decay_system);
        app.add_event::<CommsMessageEvent>();

        // High distance message
        app.world_mut().send_event(CommsMessageEvent {
            sender_id: Entity::PLACEHOLDER,
            receiver_id: Entity::PLACEHOLDER,
            distance: 1000.0,
            original_text: "We demand 500 steel immediately.".to_string(),
            scrambled_text: String::new(),
            corruption_chance: 0.0,
        });

        app.update();

        let events = app.world().resource::<Events<CommsMessageEvent>>();
        let mut reader = events.get_reader();
        let processed_msg = reader.read(events).next().unwrap();

        assert!(processed_msg.corruption_chance > 0.0);
        assert!(processed_msg.scrambled_text.contains("[CORRUPTED]"));
        assert_ne!(processed_msg.original_text, processed_msg.scrambled_text);
    }

    #[test]
    fn test_message_perfect_signal_no_corruption() {
        let mut app = App::new();
        app.add_systems(Update, process_signal_decay_system);
        app.add_event::<CommsMessageEvent>();

        app.world_mut().send_event(CommsMessageEvent {
            sender_id: Entity::PLACEHOLDER,
            receiver_id: Entity::PLACEHOLDER,
            distance: 0.0,
            original_text: "Clear signal".to_string(),
            scrambled_text: String::new(),
            corruption_chance: 0.0,
        });

        app.update();

        let events = app.world().resource::<Events<CommsMessageEvent>>();
        let mut reader = events.get_reader();
        let processed_msg = reader.read(events).next().unwrap();

        assert_eq!(processed_msg.corruption_chance, 0.0);
        assert_eq!(processed_msg.original_text, processed_msg.scrambled_text);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use rand::Rng;

#[derive(Event, Clone)]
pub struct CommsMessageEvent {
    pub sender_id: Entity,
    pub receiver_id: Entity,
    pub distance: f32,
    pub original_text: String,
    pub scrambled_text: String,
    pub corruption_chance: f32,
}

pub fn process_signal_decay_system(
    mut events: ResMut<Events<CommsMessageEvent>>,
) {
    let mut modified_events = Vec::new();

    // We drain the events to modify them, then put them back
    // (In a real implementation, you might use a pipeline of events, e.g. RawMessage -> ProcessedMessage)
    for mut msg in events.drain() {
        // Calculate corruption chance based on distance (0.1% per distance unit)
        msg.corruption_chance = (msg.distance * 0.001).min(0.9);

        let mut rng = rand::thread_rng();
        let words: Vec<&str> = msg.original_text.split_whitespace().collect();
        let mut scrambled_words = Vec::new();

        for word in words {
            if rng.gen::<f32>() < msg.corruption_chance {
                scrambled_words.push("[CORRUPTED]");
            } else {
                scrambled_words.push(word);
            }
        }

        msg.scrambled_text = scrambled_words.join(" ");
        modified_events.push(msg);
    }

    for msg in modified_events {
        events.send(msg);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Pipeline:** Modifying events in place by draining them is an anti-pattern. Better to have `RawCommsEvent` as input and `ProcessedCommsEvent` as output.
- **Interference Sources:** Factor in environmental hazards (Nebulae, Solar Flares) into the `corruption_chance` calculation, not just distance.
- **Selective Scrambling:** Instead of pure random word replacement, prioritize scrambling important game terms (numbers, resource names) to increase player tension.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Messages over long distances have words replaced with `[CORRUPTED]`.

## 7. Technical Guidance
- Ensure `rand` is available in `Cargo.toml`.
- Consider adding a `BoostSignal` action that costs energy to re-read the message with lower corruption.

## 8. Questions
*Builder: add questions here if spec is unclear.*
