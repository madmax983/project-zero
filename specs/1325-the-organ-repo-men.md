# 1325-the-organ-repo-men.md
## 1. Overview
The Organ Repo-Men is a Layer 1 mechanic where Pops can receive black-market cybernetics or organs on credit. Defaulting on payments spawns "Repo-Men" entities that forcefully extract the implants, ignoring base defenses and killing/maiming the debtor.

## 2. Dependencies
- Economy/Debt system
- Entity definitions
- Combat/Health mechanics

## 3. RED Phase: Tests First
```rust
#[test]
fn test_pop_default_spawns_repo_men() {
    // Arrange: Create a Pop with a black-market organ and high debt
    // Act: Advance simulation time to trigger default
    // Assert: Verify a RepoMan entity spawns targeting the Pop
}

#[test]
fn test_repo_man_extraction_kills_pop() {
    // Arrange: Create a Pop and a RepoMan targeting them
    // Act: Simulate RepoMan reaching the Pop
    // Assert: Verify the Pop dies (or is severely maimed) and the RepoMan holds the extracted organ
}

#[test]
fn test_repo_man_ignores_defenses() {
    // Arrange: Create base defenses between RepoMan and Pop
    // Act: Simulate RepoMan movement
    // Assert: Verify RepoMan successfully bypasses or ignores defenses
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Implement Pop debt tracking and default detection
// Implement RepoMan spawning logic
// Implement RepoMan movement and extraction logic (ignoring defenses)
// Implement Pop death/maiming upon extraction
```

## 5. REFACTOR Phase: Quality & Design
- Consider how to cleanly integrate the debt system with existing economy mechanics.
- Ensure RepoMan pathfinding logic is robust enough to always reach the target.
- Refine the consequences of extraction (death vs maiming based on organ type).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pop defaulting triggers RepoMan spawn
- [ ] RepoMan successfully extracts organ, killing/maiming Pop
- [ ] RepoMan ignores base defenses

## 7. Technical Guidance
- Integrate with existing debt or economy modules or create a specific module for implant debt.
- Use a specific component for "Black Market Implant" to track debt and trigger spawning.
- Ensure the RepoMan entity has a special tag to bypass normal pathfinding or collision rules with defenses.

## 8. Questions
*Builder: add questions here if spec is unclear.*
