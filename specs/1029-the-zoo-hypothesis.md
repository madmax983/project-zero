# 1029: The Zoo Hypothesis

## 1. Overview
The Zoo Hypothesis reveals that the colony is being observed by an advanced Alien race. These observers reward "Interesting" behavior (War, Disaster, Art) with resources or technology drops, and punish "Boring" stability. This creates an emergent meta-game where players might intentionally start fires or cause riots to entertain their alien overlords and receive care packages of much-needed food.

## 2. Dependencies
- Layer 3 `GalaxyMap` or a hidden "Observer" entity/resource.
- Layer 1 `Events` (tracking disasters, art creation, combat).
- Layer 1 `Economy` (Resource drops/care packages).
- `Chronicle` system (to track historical "interest" levels).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::disasters::DisasterEvent;
    use crate::layer1::economy::RawResources;
    use crate::layer1::crafting::{CraftEvent, Quality};

    #[test]
    fn test_interesting_events_increase_entertainment_score() {
        let mut app = App::new();
        app.insert_resource(AlienObservers { entertainment_score: 0.0, threshold: 100.0 });
        app.add_event::<DisasterEvent>();
        app.add_systems(Update, evaluate_colony_entertainment_system);

        app.world_mut().resource_mut::<Events<DisasterEvent>>().send(DisasterEvent {
            position: Default::default(),
            disaster_type: Default::default(),
        });

        app.update();

        let observers = app.world().resource::<AlienObservers>();
        assert!(observers.entertainment_score > 0.0, "Disasters should increase the alien entertainment score.");
    }

    #[test]
    fn test_high_entertainment_triggers_resource_reward() {
        let mut app = App::new();
        app.insert_resource(AlienObservers { entertainment_score: 150.0, threshold: 100.0 });
        app.insert_resource(RawResources { amount: 0 });
        app.add_systems(Update, trigger_alien_reward_system);

        app.update();

        let resources = app.world().resource::<RawResources>();
        let observers = app.world().resource::<AlienObservers>();

        assert!(resources.amount > 0, "Hitting the entertainment threshold should reward the colony with resources.");
        assert_eq!(observers.entertainment_score, 50.0, "Score should reduce by the threshold amount after a reward.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer3/zoo_hypothesis.rs
use bevy::prelude::*;
use crate::layer1::disasters::DisasterEvent;
use crate::layer1::economy::RawResources;

#[derive(Resource)]
pub struct AlienObservers {
    pub entertainment_score: f32,
    pub threshold: f32,
}

pub fn evaluate_colony_entertainment_system(
    mut events: EventReader<DisasterEvent>,
    mut observers: ResMut<AlienObservers>,
) {
    for _event in events.read() {
        // MVP: Any disaster gives a flat score boost
        observers.entertainment_score += 25.0;
    }
}

pub fn trigger_alien_reward_system(
    mut observers: ResMut<AlienObservers>,
    mut resources: ResMut<RawResources>,
) {
    if observers.entertainment_score >= observers.threshold {
        // Reward the colony
        resources.amount += 500; // Flat reward for MVP

        // Reset/reduce score
        observers.entertainment_score -= observers.threshold;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Boredom Penalty:** The `entertainment_score` should slowly decay every tick. If it drops below zero, the aliens should punish the colony (e.g., laser strikes, abducting Pops) to spur action.
- **Event Variety:** The evaluation system needs to listen to multiple event types (Combat, Art Creation, Edicts) and weight them differently. A famine is funny; a nuclear war is hilarious.
- **Reward Spawning:** Instead of just adding to `RawResources`, the reward should literally spawn an unowned `DropPod` entity on the Layer 1 map that Pops have to go retrieve.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_interesting_events_increase_entertainment_score` passes.
- [ ] Test `test_high_entertainment_triggers_resource_reward` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- Be careful not to make the rewards so good that intentionally ruining the colony becomes the optimal way to play on a macro scale—unless that's the intended grimdark tone.
- The `AlienObservers` resource could be hidden from the player initially, only revealing itself via a cryptic message when the first reward drops.

## 8. Questions
*Builder: add questions here if spec is unclear.*
