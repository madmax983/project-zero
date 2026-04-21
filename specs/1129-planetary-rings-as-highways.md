# Planetary Rings as Highways

## 1. Overview
The "Planetary Rings as Highways" feature adds a strategic element to layer 2 space travel. Fleets can use planetary rings to gain significant movement speed and stealth bonuses. However, traveling through these dense fields of debris introduces a risk of collision damage. This creates a tension between taking a fast, stealthy, but dangerous route versus a slow, safe, and visible one.

## 2. Dependencies
- Layer 2 Fleet system
- Layer 2 Map/Node system (Planetary Rings need to be represented)
- Stealth/Visibility system for Fleets

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_fleet_stealth_in_rings() {
        // Arrange
        let mut app = App::new();
        // Setup fleet and planetary ring entities

        // Act
        // Move fleet into planetary ring
        app.update();

        // Assert
        // Verify fleet visibility is reduced/stealth is increased
    }

    #[test]
    fn test_fleet_speed_in_rings() {
        // Arrange
        let mut app = App::new();
        // Setup fleet and planetary ring entities

        // Act
        // Move fleet within planetary ring
        app.update();

        // Assert
        // Verify fleet movement speed is increased
    }

    #[test]
    fn test_ring_collision_damage() {
        // Arrange
        let mut app = App::new();
        // Setup fleet with initial health and planetary ring

        // Act
        // Move fleet through planetary ring for several ticks
        for _ in 0..10 {
            app.update();
        }

        // Assert
        // Verify fleet has a chance to take damage
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation to make tests pass
// Add stealth and speed modifiers when a fleet is in a ring
// Apply random damage tick when a fleet is in a ring
```

## 5. REFACTOR Phase: Quality & Design
- Consider how the collision chance scales with fleet size or speed.
- Abstract the stealth/speed modifier logic to be reusable for other terrain types.
- Ensure the collision damage calculation is deterministic or uses a seeded RNG for predictability in simulation.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified (fleets in rings are faster, stealthier, and occasionally take damage).

## 7. Technical Guidance
- Implement a `PlanetaryRing` component or terrain modifier.
- Update the fleet movement system to check for the presence of a `PlanetaryRing` at the current location.
- Use a robust RNG for the collision damage chance, ensuring it integrates well with the deterministic simulation goals.

## 8. Questions
*Builder: add questions here if spec is unclear.*
