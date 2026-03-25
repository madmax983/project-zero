# The Ghost Fleet Protocol

## 1. Overview
The Ghost Fleet Protocol allows players to automate military ships, stripping them of crew requirements and upkeep. However, these automated fleets rely on a central communication array in the star system. If the array is destroyed or loses power, the Ghost Fleet goes rogue, becoming permanently hostile to all entities, including its creators.

## 2. Dependencies
- `src/layer2/combat.rs` (Ship components and combat systems)
- `src/layer2/structures.rs` (Communication arrays and system-level structures)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_ghost_fleet_activation() {
    let mut app = App::new();
    // Setup logic here
    // app.world.spawn((Ship, Fleet, Crew::new(100)));
    // act: trigger Ghost Fleet Protocol
    // assert: Crew component removed or set to 0, Upkeep component removed, Automated component added
}

#[test]
fn test_ghost_fleet_goes_rogue_on_comms_loss() {
    let mut app = App::new();
    // Setup a ship with Automated component and a CommsArray in the system
    // act: destroy CommsArray
    // assert: ship's Faction changes to Rogue/Hostile
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// In src/layer2/combat.rs
#[derive(Component)]
pub struct GhostFleetAutomated;

pub fn activate_ghost_fleet_system(
    mut commands: Commands,
    query: Query<(Entity, &Ship), With<ActivateGhostFleet>>,
) {
    for (entity, _) in query.iter() {
        commands.entity(entity)
            .remove::<Crew>()
            .remove::<Upkeep>()
            .insert(GhostFleetAutomated);
    }
}

pub fn ghost_fleet_rogue_system(
    mut commands: Commands,
    mut ships: Query<(Entity, &mut Faction), With<GhostFleetAutomated>>,
    comms_arrays: Query<&CommsArray, With<Active>>,
) {
    if comms_arrays.is_empty() {
        for (_, mut faction) in ships.iter_mut() {
            *faction = Faction::HostileRogue;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Centralize faction management to easily switch ships to hostile.
- Optimize the check for communication arrays (e.g., maintain a system-level resource or event when an array goes down).
- Ensure visual indicators (UI/particles) update when a fleet goes rogue.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Ships can be automated to remove crew and upkeep.
- [ ] Automated ships become permanently hostile if the system's active CommsArray is removed.

## 7. Technical Guidance
- Add a new state or component `GhostFleetAutomated` to indicate the ship's status.
- Ensure that destroying the comms array triggers an immediate state change, potentially using Bevy's `RemovedComponents` filter.
- Avoid iterating over all ships every frame if the comms array is healthy; use an event-driven approach when the array is destroyed.

## 8. Questions
*Builder: add questions here if spec is unclear.*
