# 926 - The Sentient Rumor

## 1. Overview

**Layer:** Cross-layer
**Fantasy:** An idea taking on a life of its own, independent of the truth, and spreading across worlds.
**Mechanic:** A minor gossip event mutates as it spreads between Pops and via interstellar trade routes. If unchecked by Official Broadcasts, it can evolve into a full-blown systemic panic.
**Emergence:** Ignoring a small complaint about a faulty hydro-processor on a mining outpost could result, three months later, in a heavily armed faction on the capital world storming a water treatment plant, convinced they are saving the empire from mind control.
**Tension:** Do you waste valuable administrative resources continuously debunking trivial rumors, or ignore them and risk them blossoming into violent interstellar conspiracy theories?

## 2. Dependencies

- Social interaction systems (`SocialInfluence` / `UtilityWeights`)
- Chronicle / Event System
- Logistics / Trade Routes (for cross-colony spread)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct Rumor { severity: f32, topic: String }

    #[derive(Component)]
    struct PopRumorTracker { known_rumors: Vec<Entity> }

    #[derive(Event)]
    struct BroadcastDebunkEvent { topic: String, effectiveness: f32 }

    #[test]
    fn test_rumor_severity_mutates_over_time() {
        let mut app = App::new();
        app.add_systems(Update, mutate_rumor_severity_system);

        let rumor_id = app.world_mut().spawn(Rumor {
            severity: 1.0,
            topic: "Water tastes funny".to_string()
        }).id();

        app.update();

        let mutated_rumor = app.world().get::<Rumor>(rumor_id).unwrap();
        // Severity should increase as rumor spreads unaddressed
        assert!(mutated_rumor.severity > 1.0);
    }

    #[test]
    fn test_broadcast_debunks_rumor() {
        let mut app = App::new();
        app.add_event::<BroadcastDebunkEvent>();
        app.add_systems(Update, process_debunk_broadcast_system);

        let rumor_id = app.world_mut().spawn(Rumor {
            severity: 5.0,
            topic: "Water tastes funny".to_string()
        }).id();

        app.world_mut().send_event(BroadcastDebunkEvent {
            topic: "Water tastes funny".to_string(),
            effectiveness: 4.0
        });

        app.update();

        let debunked_rumor = app.world().get::<Rumor>(rumor_id).unwrap();
        // Severity should decrease due to the debunk broadcast
        assert!(debunked_rumor.severity <= 1.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// pub fn mutate_rumor_severity_system(...) { ... }
// pub fn process_debunk_broadcast_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design

- Consider introducing a decaying multiplier for rumor severity so rumors naturally die out if not validated by Pop interactions.
- Consolidate rumors by topic to avoid spawning thousands of Rumor entities.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Rumors increase in severity over time if unaddressed, and can be mitigated by `BroadcastDebunkEvent`.

## 7. Technical Guidance

- Use time deltas when calculating the mutation/increase of rumor severity.
- Make sure to consider the bounds of `f32` when mutating severity to avoid infinite growth.

## 8. Questions
*Builder: add questions here if spec is unclear.*
