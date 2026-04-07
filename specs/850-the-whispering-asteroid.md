# 850 - The Whispering Asteroid

## 1. Overview
Mining a rock that seems to have a mind of its own, slowly driving the crew mad. Certain high-yield asteroids emit a sub-frequency that slowly increases the "Paranoia" need of Pops working on or near it.

## 2. Dependencies
- `src/layer2/mining.rs` or `src/layer1/mining.rs`
- `src/layer1/needs.rs` (Paranoia / Stress)
- `src/layer1/events.rs` (Sabotage events)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::Needs;

    fn setup_app() -> App {
        let mut app = App::new();
        // Setup minimal systems
        app
    }

    #[test]
    fn test_whispering_asteroid_increases_paranoia() {
        let mut app = setup_app();
        // Arrange: Miner working on Whispering Asteroid
        // Act: Run paranoia system over time
        // Assert: Miner's paranoia/stress increases significantly
    }

    #[test]
    fn test_high_paranoia_triggers_sabotage() {
        let mut app = setup_app();
        // Arrange: Miner with maxed out paranoia near orbital tether
        // Act: Run sabotage trigger system
        // Assert: Sabotage event is fired targeting the tether
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Components
#[derive(Component)]
pub struct WhisperingAsteroid {
    pub paranoia_rate: f32,
}

// Systems
pub fn whispering_asteroid_paranoia_system(
    time: Res<Time>,
    asteroids: Query<&WhisperingAsteroid>,
    mut miners: Query<(&mut crate::layer1::needs::Needs, &Transform)>, // Adjust based on how mining is linked
) {
    // Increase paranoia for pops near/working on the asteroid
}

pub fn paranoia_sabotage_system(
    miners: Query<&crate::layer1::needs::Needs>,
    mut sabotage_events: EventWriter<crate::layer1::events::SabotageEvent>,
) {
    // If paranoia > threshold, trigger sabotage on nearby critical infrastructure
}
```

## 5. REFACTOR Phase: Quality & Design
- Make the `WhisperingAsteroid` component generic enough that it could be used for other "cursed" resource nodes.
- Ensure the sabotage targets make sense contextually (e.g., they target the tether because they think the ore is trying to escape).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Working on a Whispering Asteroid increases paranoia.
- [ ] High paranoia leads to sabotage events.

## 7. Technical Guidance
- The connection between Layer 2 (Asteroid) and Layer 1 (Pops) needs to be handled carefully depending on how orbital mining is implemented. If pops are physically on the asteroid, use proximity. If they are in a station, link the station to the asteroid.
- Use `SimulationTime` instead of `Time` for deterministic behavior.

## 8. Questions
*Builder: add questions here if spec is unclear.*
