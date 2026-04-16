# Spec 979: The Propaganda Contagion

## 1. Overview
"The Propaganda Contagion" introduces memetic warfare as a cross-layer mechanic. Players can deploy a "Memetic Virus" via a Layer 3 espionage action. When this payload is introduced to a Layer 1 colony (e.g., through a designated trade or diplomatic event), it infects a starting Pop. Infected Pops stop their normal resource production and instead attempt to "share" the virus with adjacent Pops, while simultaneously receiving a massive artificial Morale boost. If left unchecked, the contagion spreads geometrically, shutting down the colony's economy while throwing its population into forced euphoria.

## 2. Dependencies
- `Pop` entity and systems.
- Spatial grid systems (`GridPosition`).
- Job systems and `Morale` components.
- Existing Layer 3 espionage/event bridges (if any, otherwise we define a standalone event).

## 3. RED Phase: Tests First

```rust
// tests/propaganda_contagion_tests.rs
use bevy::prelude::*;
use crate::layer1::population::Pop;
use crate::layer1::map::GridPosition;
use crate::layer1::jobs::{JobType, Worker};
use crate::layer1::social::morale::Morale;
use crate::layer3::espionage::{
    MemeticInfection, DeployMemeticVirusEvent,
    deploy_memetic_virus_system, memetic_infection_spread_system,
    memetic_production_block_system
};

#[test]
fn test_deploy_memetic_virus_infects_pop() {
    let mut app = App::new();
    app.add_event::<DeployMemeticVirusEvent>();
    app.add_systems(Update, deploy_memetic_virus_system);

    let pop_entity = app.world_mut().spawn((Pop, GridPosition { x: 5, y: 5 })).id();

    app.world_mut().send_event(DeployMemeticVirusEvent {
        target_colony: Entity::PLACEHOLDER, // Assuming standard colony targeting
        target_pop: pop_entity,
    });

    app.update();

    assert!(app.world().get::<MemeticInfection>(pop_entity).is_some(), "Pop should be infected after deployment.");
}

#[test]
fn test_memetic_infection_spreads_to_adjacent_pops() {
    let mut app = App::new();
    app.add_systems(Update, memetic_infection_spread_system);

    // Infected pop at 5,5
    app.world_mut().spawn((Pop, GridPosition { x: 5, y: 5 }, MemeticInfection));

    // Uninfected pop at 5,6 (adjacent)
    let target_pop = app.world_mut().spawn((Pop, GridPosition { x: 5, y: 6 })).id();

    // Uninfected pop at 10,10 (far)
    let far_pop = app.world_mut().spawn((Pop, GridPosition { x: 10, y: 10 })).id();

    // Needs multiple ticks or a high probability for testing
    for _ in 0..100 {
        app.update();
    }

    assert!(app.world().get::<MemeticInfection>(target_pop).is_some(), "Infection should spread to adjacent pop.");
    assert!(app.world().get::<MemeticInfection>(far_pop).is_none(), "Infection should not spread to distant pop.");
}

#[test]
fn test_memetic_infection_blocks_production_and_boosts_morale() {
    let mut app = App::new();
    app.add_systems(Update, memetic_production_block_system);

    let pop_entity = app.world_mut().spawn((
        Pop,
        MemeticInfection,
        Worker { active_job: Some(JobType::Mining), efficiency: 1.0 },
        Morale { value: 50.0 }
    )).id();

    app.update();

    let worker = app.world().get::<Worker>(pop_entity).unwrap();
    let morale = app.world().get::<Morale>(pop_entity).unwrap();

    assert_eq!(worker.efficiency, 0.0, "Infected pop should have 0 production efficiency.");
    assert_eq!(morale.value, 100.0, "Infected pop should have max morale.");
}
```

## 4. GREEN Phase: Minimal Implementation
- Create `src/layer3/espionage.rs` (or add to existing).
- Define `MemeticInfection` component.
- Define `DeployMemeticVirusEvent` containing the `target_pop`.
- `deploy_memetic_virus_system`: Reads `DeployMemeticVirusEvent` and inserts `MemeticInfection` onto the target pop.
- `memetic_infection_spread_system`: Queries all Pops with `MemeticInfection`. For each, query all Pops without `MemeticInfection`. If distance <= 1 (adjacent), roll a probability check (e.g., 5%). If successful, insert `MemeticInfection` on the target.
- `memetic_production_block_system`: Queries all Pops with `MemeticInfection`, `Worker`, and `Morale`. Sets `Worker.efficiency` to 0.0 and `Morale.value` to 100.0.

## 5. REFACTOR Phase: Quality & Design
- **Spatial Hash:** The spread system should use spatial hashing (or the `GridMap`) to find adjacent Pops instead of a O(N^2) query against all uninfected Pops.
- **Gradual Spread:** Add a cooldown or timer to `MemeticInfection` so pops don't instantly infect everyone around them in a single frame.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Coverage >= 85% for the new memetic logic.
- [ ] A virus can be deployed via event.
- [ ] The virus spreads geometrically to adjacent pops.
- [ ] Infected pops stop producing and have maxed morale.

## 7. Technical Guidance
- **Utility AI Integration:** Instead of just setting efficiency to 0, if the game uses a Utility AI decider (like `PopDecider`), `MemeticInfection` could override their current action to a `ShareMeme` action, effectively stalling their job naturally.

## 8. Questions
*Builder: Add questions here if integration with existing job/AI systems is unclear.*

*Architect:* The simplest GREEN implementation is fine: if the `MemeticInfection` component is present, a high-priority Utility AI action should preempt their normal job and force the `ShareMeme` behavior.
