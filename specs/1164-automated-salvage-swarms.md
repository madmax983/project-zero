# 1164: Automated Salvage Swarms

## Overview

Self-replicating drones sent by a long-dead empire occasionally drift into your system, aggressively breaking down anything they consider "wreckage." They don't attack Pops, but they will latch onto and rapidly disassemble damaged buildings, disabled ships, or unpowered infrastructure, converting them into raw materials and leaving them in neat, compressed cubes.

## Dependencies

- `1162` Orbital Debris Cascades

## RED Phase: Tests First

```rust
#[test]
fn test_salvage_swarm_dismantles_unpowered_building() {
    // Arrange: Setup swarm and unpowered building
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, process_salvage_swarms);

    let building = app.world_mut().spawn((
        Building,
        Unpowered,
        Health::new(100.0),
    )).id();

    let swarm = app.world_mut().spawn((
        SalvageSwarm { dismantling_rate: 10.0 },
        Target(building),
    )).id();

    // Act: Tick simulation
    app.update();

    // Assert: Building health reduced, resources spawned
    assert!(app.world().get::<Health>(building).unwrap().current < 100.0);
    // Assert resources spawned
}

#[test]
fn test_salvage_swarm_ignores_powered_building() {
    // Arrange: Setup swarm and powered building
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, process_salvage_swarms);

    let building = app.world_mut().spawn((
        Building,
        Powered, // Building is powered, should be ignored
        Health::new(100.0),
    )).id();

    let swarm = app.world_mut().spawn((
        SalvageSwarm { dismantling_rate: 10.0 },
        Target(building),
    )).id();

    // Act: Tick simulation
    app.update();

    // Assert: Building health untouched
    assert_eq!(app.world().get::<Health>(building).unwrap().current, 100.0);
}
```

## GREEN Phase: Minimal Implementation

```rust
fn process_salvage_swarms(
    mut commands: Commands,
    mut swarms: Query<(&SalvageSwarm, &Target)>,
    mut buildings: Query<(&mut Health, Option<&Unpowered>), With<Building>>,
) {
    for (swarm, target) in swarms.iter_mut() {
        if let Ok((mut health, unpowered)) = buildings.get_mut(target.0) {
            if unpowered.is_some() {
                health.current -= swarm.dismantling_rate;
                // Resource spawning logic...
                if health.current <= 0.0 {
                    commands.entity(target.0).despawn();
                }
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- The `process_salvage_swarms` system needs to spawn compressed resource cubes based on the building's composition and cost.
- Handle targeting logic: Swarms should actively search for unpowered/damaged buildings using a spatial query if their current target is invalid or destroyed.
- Add specific visual/audio events when a swarm is actively dismantling to notify the player.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Salvage swarms correctly identify and dismantle unpowered/damaged buildings.
- [ ] Salvage swarms spawn resource cubes after dismantling.
- [ ] Swarms ignore perfectly functioning, powered buildings.

## Technical Guidance

- Use existing `Health` and `Unpowered` (or equivalent power state) components.
- The resource cube spawning should hook into the existing resource/inventory system (Layer 1).

## Questions

*Builder: add questions here if spec is unclear.*
