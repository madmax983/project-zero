# 652: The Void-Whisper Resonance

## Overview

Navigating the creeping paranoia of a distant cosmic anomaly that subtly rewrites your colony's gossip and work orders. A spatial anomaly in Layer 2 emits a background frequency that intercepts and subtly alters Layer 1 communications (like social gossip and automated job assignments). Pops passing rumors occasionally receive "Void-Whispers"—hallucinatory orders or terrifying gossip that never actually happened, which they then sincerely spread as truth. The anomaly causes a rumor to manifest that the primary atmospheric scrubber is venting poison. A mass panic breaks out, Pops refuse to enter the sector, and riots start over perfectly clean air. Alternatively, a fake "work order" tells all the colony's engineers to simultaneously dismantle the main reactor, which they dutifully and disastrously begin doing.

## Dependencies

- None explicitly required beyond core rumor and job systems.

## RED Phase: Tests First

```rust
// src/layer1/events_new/void_whisper_resonance.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::pre::App;

    #[test]
    fn test_void_whisper_alters_rumors() {
        // Arrange
        let mut app = App::new();
        // Setup a rumor network and a background anomaly

        // Act
        // Pops pass a rumor near the anomaly

        // Assert
        // Expect the rumor to be occasionally intercepted and rewritten as a terrifying Void-Whisper
    }

    #[test]
    fn test_fake_work_order_generated() {
        // Arrange

        // Act
        // A work order is generated near the anomaly

        // Assert
        // Expect a hallucinatory work order (e.g., dismantling a reactor) to be occasionally generated
    }

    #[test]
    fn test_void_whisper_spreads_truthfully() {
        // Arrange

        // Act
        // Pops receive a Void-Whisper rumor

        // Assert
        // Expect Pops to spread the fake rumor as truth, causing localized panic or strange behavior
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer1/events_new/void_whisper_resonance.rs

// Minimal event and system structures to make the tests pass.
// e.g., define VoidWhisperEvent, AnomalyFrequency component, etc.
```

## REFACTOR Phase: Quality & Design

- Ensure the probability of a Void-Whisper occurring is balanced against the strength of the anomaly and the distance from the source.
- Implement the long-term consequences of widespread false rumors on colony morale and stability.
- Consider how the player can mitigate the effects of the anomaly, such as building psychic insulation or policing free speech.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] A background anomaly occasionally alters Layer 1 communications.
- [ ] Pops receive and spread hallucinatory rumors as truth.
- [ ] Fake work orders are generated, causing Pops to perform disastrous actions.

## Technical Guidance

- Use Bevy's ECS to manage the propagation of rumors and work orders, intercepting them when near an anomaly.
- The `Anomaly` component can be added to Layer 2 entities or specific regions in Layer 1.
- The Utility AI system will need to be updated to consider the option of acting on fake work orders.

## Questions

*Builder: add questions here if spec is unclear.*
