# The Pyrite Mirage

## 1. Overview
A rare, high-value ore spawns that acts like Gold but has a hidden decay timer. After a certain duration, the ore (and any buildings/items crafted from it) crumbles into toxic dust, poisoning Pops and ruining the economy.

## 2. Dependencies
- `src/layer1/items.rs` (or resource definitions)
- `src/layer1/buildings.rs` (building materials and destruction)
- `src/layer1/health.rs` (toxic dust poisoning)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_pyrite_decay_into_toxic_dust() {
    let mut app = App::new();
    // Setup Pyrite item with a decay timer
    // act: advance simulation time past decay threshold
    // assert: Pyrite item is replaced by ToxicDust item
}

#[test]
fn test_pyrite_building_collapse() {
    let mut app = App::new();
    // Setup a building constructed from Pyrite
    // act: advance simulation time past decay threshold
    // assert: Building is destroyed, ToxicDust entity spawned in its place
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// In src/layer1/resources/pyrite.rs
#[derive(Component)]
pub struct Pyrite {
    pub decay_timer: Timer,
}

#[derive(Component)]
pub struct ToxicDust;

pub fn pyrite_decay_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    mut query: Query<(Entity, &mut Pyrite, Option<&Transform>)>,
) {
    for (entity, mut pyrite, transform) in query.iter_mut() {
        if pyrite.decay_timer.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn_recursive();
            let mut dust = commands.spawn(ToxicDust);
            if let Some(t) = transform {
                dust.insert(*t);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Ensure Pyrite can masquerade as Gold in trading systems initially.
- Centralize decay logic so it cleanly handles both raw items and constructed buildings.
- Add an event when Pyrite crumbles so the Chronicle system can record the "Pyrite Collapse".

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pyrite decays into Toxic Dust after a timer.
- [ ] Buildings made of Pyrite are destroyed when the material decays.

## 7. Technical Guidance
- Integrate with existing item/resource systems.
- You may need to add a material tag to buildings to track if they are made of Pyrite.

## 8. Questions
*Builder: add questions here if spec is unclear.*
