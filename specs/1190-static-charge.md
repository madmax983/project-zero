# 1190: Static Charge

**1. Overview**
In Layer 1 of SCALE, the dry and sterile environment of a space station or colony poses hidden hazards. Specifically, movement over certain floor types (like Plastic or Carpet) causes a buildup of static electricity. If a Pop touches sensitive electronics or machinery without first being properly grounded, they will discharge this energy, potentially causing damage or glitches. This introduces a tension between movement efficiency (speed) and safety, as well as a strategic choice in base-building materials.

**2. Dependencies**
- Base Layer 1 grid and movement systems.
- Pop components (including basic pathfinding/movement actions).
- Floor/Terrain types (specifically those that generate charge vs. those that act as a ground).
- Interactable machinery or electronics systems that can take damage or trigger failure states.

**3. RED Phase: Tests First**
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_pop_builds_charge_on_insulating_floors() {
    // Arrange: Create a Pop and move them across Plastic/Carpet tiles.
    // Act: Advance simulation time.
    // Assert: Pop's StaticCharge component increases.
}

#[test]
fn test_pop_does_not_build_charge_on_grounded_floors() {
    // Arrange: Create a Pop and move them across Metal/Grounded tiles.
    // Act: Advance simulation time.
    // Assert: Pop's StaticCharge component remains zero.
}

#[test]
fn test_pop_discharges_when_touching_electronics_with_charge() {
    // Arrange: Pop with high StaticCharge interacts with an Electronics component.
    // Act: Process the interaction.
    // Assert: Pop's charge drops to zero, and the Electronics component takes damage/status effect.
}

#[test]
fn test_pop_grounds_safely_when_touching_grounded_surface_first() {
    // Arrange: Pop with high StaticCharge moves onto a Grounded floor tile.
    // Act: Advance simulation time.
    // Assert: Pop's StaticCharge drops to zero safely without causing damage.
}

#[test]
fn test_charge_decay_over_time() {
    // Arrange: Pop with some StaticCharge stands still.
    // Act: Advance simulation time over a long period.
    // Assert: StaticCharge naturally decays slowly, even without grounding.
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN

// Minimal implementation steps:
// 1. Add `StaticCharge` component to Pops.
// 2. Add system to check Pop movement and floor type, incrementing `StaticCharge` for insulating floors.
// 3. Add system/event for when a Pop interacts with machinery, checking their `StaticCharge` and applying damage if high enough.
// 4. Add system to quickly reset `StaticCharge` when stepping on grounded tiles.
// 5. Add slow decay system for `StaticCharge` over time.
```

**5. REFACTOR Phase: Quality & Design**
- Consider creating a generic `Conductivity` trait or component for tiles rather than hardcoding floor types.
- Ensure the interaction damage logic scales appropriately with the amount of built-up charge.
- Add visual or UI indicators (like a spark effect or a warning icon) when a Pop is dangerously charged.
- Optimize the movement check to only run when the Pop actually changes tiles.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops build charge on specific tiles and discharge on others.
- [ ] Machinery can be damaged by a static discharge event.

**7. Technical Guidance**
- Integrate tightly with the existing movement and grid systems to detect tile transitions efficiently.
- Use an Event-driven approach for the discharge (e.g., `StaticDischargeEvent`) to decouple the charge logic from the specific machinery damage logic.
- Keep the charge buildup frame-rate independent by scaling with `time.delta_secs()`.

**8. Questions**
*Builder: add questions here if spec is unclear.*
