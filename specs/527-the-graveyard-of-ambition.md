# Overview

**The Graveyard of Ambition**
**Layer:** 2
**Fantasy:** You aren't the first to try and tame this sector. The ruins of failed empires are hazards and opportunities.
**Mechanic:** Certain Layer 2 nodes are "Hulks"—massive, dead megastructures from previous civilizations. They offer incredible salvage, but navigating them is perilous. They possess dormant automated defenses, unstable containment fields, or lingering bio-hazards.
**Emergence:** You send a fleet to salvage a Dyson Sphere fragment. They accidentally trigger a dormant stellar-manipulation engine, causing a localized gravity well that traps your fleet and disrupts trade routes in the entire sector.
**Tension:** The lure of endgame technology and massive resource payouts vs. the high probability of waking something that should have stayed asleep.

# Dependencies

- `094-system-view.md`
- `099-fleet-movement.md`
- `158-fleet-management.md`

# RED Phase: Tests First

```rust
// tests/layer2/graveyard_of_ambition_tests.rs

#[test]
fn test_hulk_generation() {
    let mut app = setup_world();

    // Act: Generate system
    app.world_mut().send_event(SystemGenerationEvent);
    app.update();

    // Assert: At least one Hulk exists
    let mut hulk_query = app.world_mut().query::<&Hulk>();
    assert!(hulk_query.iter(app.world()).count() > 0);
}

#[test]
fn test_salvage_fleet_triggers_hazard() {
    let mut app = setup_world();

    // Arrange: A fleet at a Hulk node
    let hulk = app.world_mut().spawn((
        Node::default(),
        Hulk { hazard_level: 0.8 },
    )).id();

    let fleet = app.world_mut().spawn((
        Fleet::default(),
        Location { node: hulk },
        Salvaging { progress: 0.0 },
        Health { current: 100, max: 100 },
    )).id();

    // Act: Process salvaging
    app.update(); // Simulate ticks

    // Assert: Fleet took damage due to hazard trigger
    let health = app.world().get::<Health>(fleet).unwrap();
    assert!(health.current < 100);
}

#[test]
fn test_salvage_success_yields_tech() {
    let mut app = setup_world();

    // Arrange
    let hulk = app.world_mut().spawn((
        Node::default(),
        Hulk { hazard_level: 0.0 }, // Safe for test
    )).id();

    let fleet = app.world_mut().spawn((
        Fleet { owner: FactionId::Player },
        Location { node: hulk },
        Salvaging { progress: 99.0 },
    )).id();

    let initial_tech = app.world().resource::<EmpireTech>().points;

    // Act: Finish salvage
    app.world_mut().resource_mut::<SimulationTime>().tick += 1;
    app.update();

    // Assert: Tech points increased
    let new_tech = app.world().resource::<EmpireTech>().points;
    assert!(new_tech > initial_tech);
}
```

# GREEN Phase: Minimal Implementation

```rust
// src/layer2/graveyard.rs

use bevy::prelude::*;
use crate::layer2::map::{Node, Location};
use crate::layer2::fleet::{Fleet, Health};
use crate::layer3::empire::EmpireTech;
use rand::Rng;

#[derive(Component)]
pub struct Hulk {
    pub hazard_level: f32, // 0.0 to 1.0
}

#[derive(Component)]
pub struct Salvaging {
    pub progress: f32,
}

pub fn process_salvage_system(
    mut commands: Commands,
    mut fleets: Query<(Entity, &Location, &mut Salvaging, &mut Health), With<Fleet>>,
    hulks: Query<&Hulk>,
    mut tech: ResMut<EmpireTech>,
) {
    let mut rng = rand::thread_rng();

    for (fleet_entity, loc, mut salvaging, mut health) in fleets.iter_mut() {
        if let Ok(hulk) = hulks.get(loc.node) {
            // Hazard check
            if rng.gen::<f32>() < hulk.hazard_level * 0.1 {
                health.current -= 10; // Hazard triggers
            }

            salvaging.progress += 1.0;

            if salvaging.progress >= 100.0 {
                tech.points += 500; // Big payout
                commands.entity(fleet_entity).remove::<Salvaging>();
            }
        }
    }
}
```

# REFACTOR Phase: Quality & Design

- **Hazards**: Hazards shouldn't just be raw damage. We should spawn `HazardEvent`s that might trap the fleet (gravity well), disable sensors, or spawn hostile drone fleets.
- **Loot**: Instead of flat tech points, it should yield `Artifacts` or unlock specific late-game technologies directly.
- **Chronicle**: Generating a Hulk should create a pre-history Chronicle entry (Spec 010) about its fall. Successfully salvaging it should create a major event.

# Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer2/graveyard.rs`.
- [ ] Hulks spawn during Layer 2 generation.
- [ ] Fleets can salvage Hulks, taking damage randomly based on hazard level.
- [ ] Completing salvage yields rewards (Tech/Resources).

# Technical Guidance

- Integrate with `src/layer2/generation.rs` to spawn `Hulk` nodes at system edges or specific lagrange points.
- Create a specific UI command on Layer 2 to issue the "Salvage" order to a Fleet at a Hulk node.
- A fleet reduced to 0 health during salvage is destroyed (standard fleet combat rules apply).

# Questions

*Builder: add questions here if spec is unclear.*
