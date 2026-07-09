# 1323: The Industrial Rhythm

## 1. Overview
**Layer:** 1

**Fantasy:** The factory is a symphony.

**Mechanic:** Machines have "Cycle Times". If adjacent machines are synchronized (finishing at the same time), they create "Rhythm". High Rhythm boosts worker Morale. Discordant noise increases Stress.

**Emergence:** You build a factory that sounds like a drum circle. Workers love it. Then you upgrade one machine, breaking the beat, and everyone gets a headache.

**Tension:** Upgrade individual machines (Efficiency) vs. Maintain the beat (Morale).

## 2. Dependencies
- ECS (`bevy_ecs`)
- Pop needs system (`src/layer1/needs.rs`)
- Morale component (`crate::layer1::social::morale::Morale`)
- Machine simulation (`src/layer1/infrastructure/machine.rs`)

## 3. RED Phase: Tests First

```rust
// tests/industrial_rhythm_tests.rs
use bevy::prelude::*;

#[test]
fn test_synchronized_machines_boost_morale() {
    let mut app = App::new();
    app.add_systems(Update, rhythm_morale_system);

    // Two adjacent machines with matching cycle times
    app.world_mut().spawn((
        Machine { cycle_time: 2.0, timer: 0.0, active: true },
        GridPosition { x: 5, y: 5 },
    ));
    app.world_mut().spawn((
        Machine { cycle_time: 2.0, timer: 0.0, active: true },
        GridPosition { x: 6, y: 5 },
    ));

    // A pop nearby
    let pop_id = app.world_mut().spawn((
        Morale { value: 0.5 },
        GridPosition { x: 5, y: 6 },
    )).id();

    app.update();

    let morale = app.world().get::<Morale>(pop_id).unwrap();
    assert!(morale.value > 0.5, "Synchronized adjacent machines should increase pop morale.");
}

#[test]
fn test_discordant_machines_reduce_morale() {
    let mut app = App::new();
    app.add_systems(Update, rhythm_morale_system);

    // Two adjacent machines with mismatched cycle times
    app.world_mut().spawn((
        Machine { cycle_time: 2.0, timer: 0.0, active: true },
        GridPosition { x: 5, y: 5 },
    ));
    app.world_mut().spawn((
        Machine { cycle_time: 3.1, timer: 0.0, active: true },
        GridPosition { x: 6, y: 5 },
    ));

    // A pop nearby
    let pop_id = app.world_mut().spawn((
        Morale { value: 0.5 },
        GridPosition { x: 5, y: 6 },
    )).id();

    app.update();

    let morale = app.world().get::<Morale>(pop_id).unwrap();
    assert!(morale.value < 0.5, "Discordant adjacent machines should decrease pop morale (increase stress).");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/industrial_rhythm.rs
use bevy::prelude::*;

#[derive(Component)]
pub struct Machine {
    pub cycle_time: f32,
    pub timer: f32,
    pub active: bool,
}

#[derive(Component)]
pub struct Morale {
    pub value: f32,
}

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

pub fn rhythm_morale_system(
    machine_query: Query<(&Machine, &GridPosition)>,
    mut pop_query: Query<(&mut Morale, &GridPosition)>,
) {
    for (mut morale, p_pos) in pop_query.iter_mut() {
        let mut local_machines = Vec::new();

        // Find nearby active machines
        for (machine, m_pos) in machine_query.iter() {
            if !machine.active { continue; }
            let dx = (p_pos.x - m_pos.x).abs();
            let dy = (p_pos.y - m_pos.y).abs();
            if dx <= 2 && dy <= 2 {
                local_machines.push(machine);
            }
        }

        if local_machines.len() >= 2 {
            // Check rhythm vs discord
            let base_cycle = local_machines[0].cycle_time;
            let is_rhythmic = local_machines.iter().all(|m| (m.cycle_time - base_cycle).abs() < 0.1);

            if is_rhythmic {
                morale.value = (morale.value + 0.05).min(1.0);
            } else {
                morale.value = (morale.value - 0.05).max(0.0);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Harmonic Ratios:** Pure identical cycles are rare. It should check if cycles are multiples of each other (e.g., 2s and 4s create a polyrhythm, not discord).
- **Performance:** Iterating all machines for every pop is slow. Calculate a 'Rhythm Field' on the `TerrainGrid` and have pops sample their current tile.
- **Clippy Fixes:** Handle floating point comparisons carefully (`abs() < epsilon`).

## 6. Acceptance Criteria
- [ ] All RED tests pass.
- [ ] Coverage >= 85%.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Pops near identically-timed machines gain Morale.
- [ ] Pops near out-of-sync machines lose Morale.

## 7. Technical Guidance
- When querying Morale, use `crate::layer1::social::morale::Morale`.
- Remember Unrest is a global Resource, so do not check Unrest on individual pops; check their `Morale` instead (morale < 0.2 is high unrest).

## 8. Questions
*Builder: Add any questions here.*
