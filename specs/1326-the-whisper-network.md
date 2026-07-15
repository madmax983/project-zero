# 1326: The Whisper Network

## 1. Overview
A shadow communication channel that bypasses official propaganda and logic, where rumors become reality. When a Pop experiences a traumatic event, they share a distorted version with social contacts. If enough Pops share the same distorted memory, it materializes as a colony-wide "Whisper," generating localized panics or mass migrations regardless of ground truth.

## 2. Dependencies
- `010` Chronicle & Memories (for the Pop memory system)
- `147` Secret Societies (for social networking basics, optional but related)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::social::whisper::{WhisperNetwork, spread_rumor_system, WhisperEvent, Rumor};
    use crate::layer1::pop::Pop;
    use crate::layer1::memory::{Memory, MemoryType};

    #[test]
    fn test_rumor_spreads_to_nearby_pops() {
        let mut world = World::new();

        let pop1 = world.spawn((Pop::new(), Transform::from_xyz(0.0, 0.0, 0.0))).id();
        let pop2 = world.spawn((Pop::new(), Transform::from_xyz(1.0, 0.0, 0.0))).id();

        // Pop 1 experiences a traumatic event and generates a Rumor
        world.entity_mut(pop1).insert(Rumor { topic: "Atmosphere Failure".to_string(), intensity: 5.0 });

        world.insert_resource(WhisperNetwork::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(spread_rumor_system);
        schedule.run(&mut world);

        // Pop 2 should now have the rumor
        assert!(world.get::<Rumor>(pop2).is_some(), "Rumor should spread to nearby pop");

        // Intensity should be tracked globally
        let network = world.resource::<WhisperNetwork>();
        assert!(*network.active_rumors.get("Atmosphere Failure").unwrap_or(&0.0) > 0.0, "Global rumor intensity should increase");
    }

    #[test]
    fn test_high_intensity_rumor_creates_whisper_event() {
        let mut world = World::new();

        let mut network = WhisperNetwork::default();
        network.active_rumors.insert("Atmosphere Failure".to_string(), 150.0); // High intensity
        world.insert_resource(network);

        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_whispers_system);
        schedule.run(&mut world);

        // An event should be spawned
        let mut query = world.query::<&WhisperEvent>();
        assert_eq!(query.iter(&world).count(), 1, "A WhisperEvent should be generated when intensity exceeds threshold");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use crate::layer1::pop::Pop;

#[derive(Component, Clone)]
pub struct Rumor {
    pub topic: String,
    pub intensity: f32,
}

#[derive(Resource, Default)]
pub struct WhisperNetwork {
    pub active_rumors: HashMap<String, f32>,
}

#[derive(Component)]
pub struct WhisperEvent {
    pub topic: String,
}

pub fn spread_rumor_system(
    mut network: ResMut<WhisperNetwork>,
    mut commands: Commands,
    mut pops: Query<(Entity, &Transform, Option<&Rumor>), With<Pop>>,
) {
    let mut new_rumors = Vec::new();

    // N^2 naive check for now
    for (entity1, t1, rumor1_opt) in pops.iter() {
        if let Some(rumor1) = rumor1_opt {
            for (entity2, t2, rumor2_opt) in pops.iter() {
                if entity1 != entity2 && rumor2_opt.is_none() {
                    let dist = ((t1.translation.x - t2.translation.x).powi(2) +
                               (t1.translation.y - t2.translation.y).powi(2)).sqrt();
                    if dist < 2.0 {
                        new_rumors.push((entity2, rumor1.clone()));

                        // Increase global intensity
                        let entry = network.active_rumors.entry(rumor1.topic.clone()).or_insert(0.0);
                        *entry += rumor1.intensity * 0.5; // Mutates/loses a bit of intensity per hop
                    }
                }
            }
        }
    }

    for (entity, rumor) in new_rumors {
        // Just insert it for now, can get fancy with mutation later
        commands.entity(entity).insert(rumor);
    }
}

pub fn evaluate_whispers_system(
    mut commands: Commands,
    mut network: ResMut<WhisperNetwork>,
) {
    let mut resolved_topics = Vec::new();

    for (topic, intensity) in network.active_rumors.iter() {
        if *intensity > 100.0 {
            commands.spawn(WhisperEvent { topic: topic.clone() });
            resolved_topics.push(topic.clone());
        }
    }

    // Reset/remove triggered whispers
    for topic in resolved_topics {
        network.active_rumors.remove(&topic);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Mutation:** The `intensity` should vary. More importantly, the `topic` could occasionally mutate (e.g., "Food Shortage" -> "Poisoned Food") when spreading.
- **Panic Hook:** `WhisperEvent` should hook into the Pop AI to cause actual gameplay effects, like fleeing a sector, abandoning jobs, or rioting.
- **Performance:** Replace N^2 distance checking with a spatial hash or just check pops that are currently occupying the same "Room" or "SocialZone".

## 6. Acceptance Criteria
- [ ] Tests pass
- [ ] Rumors spread to adjacent Pops without the rumor.
- [ ] `WhisperNetwork` accumulates intensity.
- [ ] `WhisperEvent` is generated at threshold.
- [ ] Coverage >85%.

## 7. Technical Guidance
- Integrate into `src/layer1/social/whisper.rs`.
- Be careful with spatial queries; Bevy might have better ways to find nearby entities if KD-trees or spatial grids are already implemented in `layer1/map`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
