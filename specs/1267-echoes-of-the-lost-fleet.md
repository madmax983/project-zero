# Overview
What: Introduce "Echoes of the Lost Fleet" where an ancient, automated ghost dreadnought drifts into the system and broadcasts charismatic military propaganda, luring away pops with militaristic traits.
Why: This brings the idea to life: the haunted bureaucracy of a ghost ship disrupting local trade. It creates tension by stealing valuable pops, forcing the player to choose between letting the ship pass or risking their own fleets (and angering militaristic pops) to destroy the relic.

# Dependencies
- Needs `004-pop-entity.md` for `Pop` and `Traits` components.
- Needs `047-pop-relationships.md` or similar for tracking militaristic pops (or a simple trait like `Trait::Militaristic`).
- Needs `1127-escape-pods.md` or similar to handle pops leaving the colony.
- Needs a mechanism to track Layer 2 anomalies (e.g. `Layer2Anomaly`).

# RED Phase: Tests First
```rust
#[test]
fn test_lost_fleet_broadcast_lures_militaristic_pops() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup the dreadnought anomaly
    app.world.spawn(GhostDreadnought { is_broadcasting: true, ticks_remaining: 100 });

    // Setup a pop with the Militaristic trait
    let mut traits = Traits::default();
    traits.add(Trait::Militaristic);
    let pop_id = app.world.spawn((
        Pop,
        traits,
        GridPosition { x: 10, y: 10 },
    )).id();

    // Run the luring system
    lure_militaristic_pops_system(&mut app.world);

    // Pop should receive a "Lured" component or be despawned/marked for escape
    let has_lured = app.world.get::<LuredByGhostFleet>(pop_id).is_some();
    assert!(has_lured, "Militaristic pop should be lured by the ghost fleet broadcast");
}

#[test]
fn test_non_militaristic_pops_ignore_broadcast() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup the dreadnought anomaly
    app.world.spawn(GhostDreadnought { is_broadcasting: true, ticks_remaining: 100 });

    // Setup a pop without the Militaristic trait
    let pop_id = app.world.spawn((
        Pop,
        Traits::default(),
        GridPosition { x: 10, y: 10 },
    )).id();

    // Run the luring system
    lure_militaristic_pops_system(&mut app.world);

    let has_lured = app.world.get::<LuredByGhostFleet>(pop_id).is_some();
    assert!(!has_lured, "Non-militaristic pop should ignore the ghost fleet broadcast");
}
```

# GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct GhostDreadnought {
    pub is_broadcasting: bool,
    pub ticks_remaining: u32,
}

#[derive(Component)]
pub struct LuredByGhostFleet;

pub fn lure_militaristic_pops_system(world: &mut World) {
    let mut is_broadcasting = false;
    for dreadnought in world.query::<&GhostDreadnought>().iter(world) {
        if dreadnought.is_broadcasting && dreadnought.ticks_remaining > 0 {
            is_broadcasting = true;
            break;
        }
    }

    if !is_broadcasting {
        return;
    }

    let mut pops_to_lure = Vec::new();
    for (entity, traits) in world.query::<(Entity, &Traits)>().iter(world) {
        if traits.has(Trait::Militaristic) {
            pops_to_lure.push(entity);
        }
    }

    for entity in pops_to_lure {
        if let Some(mut cmds) = world.get_entity_mut(entity) {
            cmds.insert(LuredByGhostFleet);
        }
    }
}
```

# REFACTOR Phase: Quality & Design
- Integrate `lure_militaristic_pops_system` into the Layer 1-2 interaction module.
- `LuredByGhostFleet` should hook into the utility AI so the pop abandons their job, paths to an escape pod/shuttle bay, and eventually despawns from the colony.
- Handle dreadnought despawn/destruction, ensuring the broadcast stops.
- If the player destroys the dreadnought, militaristic pops should receive a severe, long-lasting morale penalty.

# Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

# Technical Guidance
- Verify if `Trait::Militaristic` exists. If not, map to a suitable existing trait or add it.
- Hook up a `DespawnLuredPops` system or leverage the existing escape pod logic from spec 1127 to handle their departure.

# Questions
*Builder: add questions here if spec is unclear.*
