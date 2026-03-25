# The Generational Grudge

## 1. Overview
A cross-layer mechanic where personal grievances echo through centuries. When a Pop is severely wronged by a faction, they form a "Grudge" memory that is inherited by their descendants. If a descendant rises to a position of power (Layer 3 governor/leader), they will unilaterally sabotage diplomacy or declare war on that faction.

## 2. Dependencies
- `src/layer1/memory.rs` (Pop memories and heritage)
- `src/layer3/diplomacy.rs` (Inter-faction relations and war declarations)
- `src/layer1/population.rs` (Descendants and inheritance)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_grudge_memory_inheritance() {
    let mut app = App::new();
    // Setup Parent Pop with Grudge(Faction::Pirates)
    // act: spawn child Pop
    // assert: child Pop inherits Grudge(Faction::Pirates)
}

#[test]
fn test_leader_with_grudge_forces_war() {
    let mut app = App::new();
    // Setup a Layer 3 Governor who has Grudge(Faction::Empire)
    // act: run diplomacy AI tick
    // assert: Diplomacy state with Empire changes to War, bypassing standard strategic logic
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// In src/layer1/memory.rs
#[derive(Clone, Component)]
pub struct Grudge(pub Entity); // Entity represents the offending faction

pub fn inherit_grudges_system(
    mut commands: Commands,
    births: EventReader<PopBornEvent>,
    parents: Query<&Grudge>,
) {
    for event in births.read() {
        if let Ok(grudge) = parents.get(event.parent) {
            commands.entity(event.child).insert(grudge.clone());
        }
    }
}

// In src/layer3/diplomacy.rs
pub fn grudge_diplomacy_override_system(
    mut diplomacy: ResMut<DiplomacyState>,
    governors: Query<(&Governor, &Grudge)>,
) {
    for (_, grudge) in governors.iter() {
        // Governor unilaterally declares war on the grudged faction
        diplomacy.set_stance(grudge.0, DiplomaticStance::War);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Add an event when a Grudge forces a war declaration to update the UI and Chronicle.
- Add ways to slowly "decay" a grudge over multiple generations, or through extreme positive diplomatic events.
- Ensure the governor overriding diplomacy is visible to the player (e.g., "Governor X has unilaterally declared war on Y due to a Generational Grudge").

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Grudge components are passed from parent to child on birth.
- [ ] If a Pop with a Grudge becomes a Governor/Leader, they unilaterally force a hostile diplomatic stance against the target faction.

## 7. Technical Guidance
- Modify the birth event/system to copy `Grudge` components to offspring.
- In Layer 3 diplomacy evaluations, intercept the normal AI or player logic if the leading entity possesses a `Grudge` against the evaluating faction.

## 8. Questions
*Builder: add questions here if spec is unclear.*
