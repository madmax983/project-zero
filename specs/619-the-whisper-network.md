# 619: The Whisper Network

## 1. Overview
Spies aren't trained agents; they are just bored bartenders listening to loose lips. The Whisper Network is a cross-layer mechanic where "Listening Posts" on Layer 2 and Taverns on Layer 1 generate "Intel" on neighboring factions based on visitor traffic. Pops with high Social stats transmit messages freely, but these messages can mutate (game of telephone) or be fabricated. Tension arises between secure, free communication and the high risk of catastrophic misinformation.

## 2. Dependencies
- Layer 1 Tavern/Social buildings.
- Layer 2 Fleet/Trade routing.
- Cross-layer messaging or Intel resource system.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_tavern_generates_intel_based_on_social_visitors() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, tavern_intel_generation_system);
        app.insert_resource(GlobalIntel { points: 0 });

        let tavern = app.world_mut().spawn(Tavern { visitors: vec![] }).id();

        let high_social_pop = app.world_mut().spawn(Pop { social_stat: 8 }).id();
        let low_social_pop = app.world_mut().spawn(Pop { social_stat: 2 }).id();

        // Act - Add pops to tavern
        app.world_mut().get_mut::<Tavern>(tavern).unwrap().visitors.push(high_social_pop);
        app.world_mut().get_mut::<Tavern>(tavern).unwrap().visitors.push(low_social_pop);

        app.update();

        // Assert
        let intel = app.world().resource::<GlobalIntel>();
        // Only high social pop (>5) should generate intel (e.g., +1 per high pop)
        assert_eq!(intel.points, 1, "Tavern should generate 1 intel point for the high social pop visitor");
    }

    #[test]
    fn test_whisper_message_mutation() {
        // Arrange
        let mut app = App::new();
        app.add_event::<WhisperMessageEvent>();
        app.add_event::<ReceivedWhisperEvent>();
        app.add_systems(Update, whisper_transmission_system);

        // Act - Send a message with high mutation chance
        app.world_mut().send_event(WhisperMessageEvent {
            original_content: "Trade Deal Accepted".to_string(),
            mutation_chance: 1.0, // 100% chance to mutate
            target_colony: Entity::PLACEHOLDER,
        });

        app.update();

        // Assert
        let received_events = app.world().resource::<Events<ReceivedWhisperEvent>>();
        let mut reader = received_events.get_reader();
        let event = reader.read(received_events).next().unwrap();

        assert_ne!(event.content, "Trade Deal Accepted", "Message should have mutated");
        assert!(event.is_corrupted, "Message should be flagged as corrupted/mutated");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Tavern {
    pub visitors: Vec<Entity>,
}

#[derive(Component)]
pub struct Pop {
    pub social_stat: u32,
}

#[derive(Resource)]
pub struct GlobalIntel {
    pub points: u32,
}

#[derive(Event)]
pub struct WhisperMessageEvent {
    pub original_content: String,
    pub mutation_chance: f32,
    pub target_colony: Entity,
}

#[derive(Event)]
pub struct ReceivedWhisperEvent {
    pub content: String,
    pub is_corrupted: bool,
    pub target_colony: Entity,
}

pub fn tavern_intel_generation_system(
    mut intel: ResMut<GlobalIntel>,
    mut query: Query<&mut Tavern>,
    pop_query: Query<&Pop>,
) {
    for mut tavern in query.iter_mut() {
        for &visitor_id in &tavern.visitors {
            if let Ok(pop) = pop_query.get(visitor_id) {
                if pop.social_stat > 5 {
                    intel.points += 1;
                }
            }
        }
        // Clear visitors after processing to simulate them leaving
        tavern.visitors.clear();
    }
}

pub fn whisper_transmission_system(
    mut requests: EventReader<WhisperMessageEvent>,
    mut deliveries: EventWriter<ReceivedWhisperEvent>,
) {
    for req in requests.read() {
        let is_corrupted = req.mutation_chance >= 1.0; // Minimal implementation: fixed trigger

        let final_content = if is_corrupted {
            format!("{} [CORRUPTED: Misunderstood]", req.original_content)
        } else {
            req.original_content.clone()
        };

        deliveries.send(ReceivedWhisperEvent {
            content: final_content,
            is_corrupted,
            target_colony: req.target_colony,
        });
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Mutation Logic:** Replace the simple boolean `mutation_chance >= 1.0` with a proper RNG check using `rand` or `fastrand`.
- **Content Mutation:** Instead of just appending `[CORRUPTED]`, swap out keywords using a lexicon to fundamentally change the message's meaning (e.g., "Trade" -> "War").
- **Tavern Flow:** The visitor processing clears visitors instantly. Integrate this with the existing job/needs system so Pops stay in the tavern over time and generate Intel continuously.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Taverns generate Intel points only from Pops with high Social stats.
- [ ] Transmitted messages have a chance to mutate before arriving.

## 7. Technical Guidance
- **Cross-Layer Coupling:** The `WhisperMessageEvent` acts as the bridge. Layer 1 UI can trigger it, and Layer 2/3 diplomacy systems can listen for `ReceivedWhisperEvent`.
- **RNG Determinism:** For multiplayer or replays, ensure the RNG used for `mutation_chance` is deterministic or seeded appropriately.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
