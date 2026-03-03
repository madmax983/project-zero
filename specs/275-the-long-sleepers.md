# Spec 275: The Long Sleepers

## 1. Overview
Ancient, automated fleets drift through the galaxy (Layer 3) and occasionally pass through your systems (Layer 2). They ignore borders and diplomacy. They broadcast a simple demand for a specific, often rare, resource as a "Toll" for their passage. Paying the toll grants a temporary system-wide buff or ancient tech. Refusing or failing to pay triggers an immediate, devastating orbital bombardment.

## 2. Dependencies
- `099` Fleet Movement
- `046` Notifications System
- `206` Orbital Crossfire

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sleeper_fleet_spawns_and_moves() {
        // Arrange: World with Layer 2 system
        // Act: Trigger sleeper fleet spawn event
        // Assert: Sleeper fleet entity exists and is moving along a path
    }

    #[test]
    fn test_sleeper_toll_payment_success() {
        // Arrange: Sleeper fleet demanding 500 Glow-Moss, Colony has 500 Glow-Moss
        // Act: Player pays toll
        // Assert: Resources deducted, Colony gains buff, Fleet becomes peaceful/departs
    }

    #[test]
    fn test_sleeper_toll_payment_failure_triggers_bombardment() {
        // Arrange: Sleeper fleet demanding resources, timer expires
        // Act: Advance time past toll deadline
        // Assert: Orbital bombardment event is triggered on the colony
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct SleeperFleet {
    pub demanded_resource: ResourceType,
    pub amount: u32,
    pub deadline_ticks: u64,
}

// System to handle the timer and trigger bombardment if unpaid
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: Tie the toll demand into the UI so the player is clearly notified.
- **Narrative**: Generate a Chronicle event when the fleet arrives, when the toll is paid, or when the bombardment begins.
- **Modularity**: The bombardment should reuse the logic from Spec 206 (Orbital Crossfire).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.
- [ ] Sleeper fleets spawn periodically or via specific trigger events.
- [ ] Player receives a clear demand for resources.
- [ ] Paying the demand provides a buff/reward and consumes the resources.
- [ ] Failing to pay within the time limit results in a damaging event to the colony.

## 7. Technical Guidance
- The `SleeperFleet` can be a distinct entity in Layer 2 that travels through the system.
- Hook into the `NotificationSystem` to alert the player.
- Provide an interaction (e.g., an action or UI button) to "Pay Toll".

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
