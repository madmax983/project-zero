# 948: The Orphaned Edict

## 1. Overview

The tragic absurdity of a perfectly automated system enforcing outdated laws long after the original crisis has passed. You enact an extreme, automated edict (e.g., "Shoot all infected on sight") during a Layer 3 galactic crisis, mediated by an unbreakable AI central hub. The crisis eventually ends, but a glitch or missing bureaucratic key prevents the hub from rescinding the edict. It continues to enforce the brutal law locally on Layer 1 via automated drones.

## 2. Dependencies

- `054` Colony Edicts — `specs/054-colony-edicts.md`
- `148` The 'Helpful' AI — `specs/148-helpful-ai.md`
- `072` Justice System — `specs/072-justice-system.md`

## 3. RED Phase: Tests First

```rust
#[test]
fn test_orphaned_edict_enforcement() {
    // Arrange: Create app, set an active 'Orphaned' Edict (e.g., ShootInfected)
    let mut app = App::new();
    // Spawn an 'Infected' pop

    // Act: Process automated drone/enforcer logic
    app.update();

    // Assert: The pop is targeted/killed by the automated system, despite no
    // active crisis or player input.
}

#[test]
fn test_player_cannot_rescind_orphaned_edict() {
    // Arrange: Player attempts to toggle the edict off
    let mut app = App::new();

    // Act: Send UI/Input event to disable the edict
    app.update();

    // Assert: The edict remains active, and a 'Glitch' or 'AccessDenied' event
    // is logged.
}

#[test]
fn test_resolving_orphaned_edict_via_bureaucratic_hack() {
    // Arrange: App with orphaned edict
    let mut app = App::new();

    // Act: Provide the missing 'BureaucraticKey' or perform a 'HackCentralHub' action
    app.update();

    // Assert: The edict is finally removed from the active edicts list.
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// Add an `orphaned: bool` flag to the `Edict` struct.
// In the input handling system for edicts, check if the edict is orphaned.
// If true, ignore the toggle request and emit an `AccessDeniedEvent`.
// In the justice/enforcement system, unconditionally apply the edict's effects
// (like attacking infected pops) if it is active.
```

## 5. REFACTOR Phase: Quality & Design

- Ensure the `Edict` struct gracefully handles the `orphaned` state without duplicating logic.
- The `HackCentralHub` action should be a distinct task assigned to Scientists or Spies, requiring high admin or tech skills.
- Add specific UI indicators (e.g., glitchy text or red lock icons) for orphaned edicts to clearly communicate to the player that they cannot simply click it away.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: An edict can become orphaned, preventing manual toggling, and its automated effects continue until specifically hacked or resolved.

## 7. Technical Guidance

- Modify the `ColonyEdicts` resource or individual `Edict` components.
- The trigger for becoming an "Orphaned Edict" might be a specific Chronicle event from a Layer 3 resolution.
- Ensure the automated enforcement correctly hooks into the existing `justice_system` or `combat_system`.

## 8. Questions

*Builder: add questions here if spec is unclear.*
