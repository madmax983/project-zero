# 1224: The Industrial Rhythm

## 1. Overview
**Layer:** 1

**Fantasy:** The factory is a symphony.

**Mechanic:** Machines have "Cycle Times". If adjacent machines are synchronized (finishing at the same time), they create "Rhythm". High Rhythm boosts worker Morale. Discordant noise increases Stress.

**Emergence:** You build a factory that sounds like a drum circle. Workers love it. Then you upgrade one machine, breaking the beat, and everyone gets a headache.

**Tension:** Upgrade individual machines (Efficiency) vs. Maintain the beat (Morale).

## 2. Dependencies
- Industrial/Production system
- Morale / Unrest system

## 3. RED Phase: Tests First
```rust
#[test]
fn test_synchronized_machines_grant_rhythm_buff() {
    let mut app = App::new();
    app.add_systems(Update, process_industrial_rhythm);

    // Spawn two machines with the same cycle time nearby
    app.world_mut().spawn((
        Building { type_: BuildingType::Factory },
        MachineCycle { current_tick: 0, max_ticks: 10 },
        GridPosition { x: 5, y: 5 },
    ));

    app.world_mut().spawn((
        Building { type_: BuildingType::Factory },
        MachineCycle { current_tick: 0, max_ticks: 10 },
        GridPosition { x: 6, y: 5 }, // Adjacent
    ));

    // Worker nearby
    let worker = app.world_mut().spawn((
        Pop,
        Morale { value: 50.0 },
        GridPosition { x: 5, y: 5 },
    )).id();

    app.update();

    // The worker's morale should increase due to rhythm
    let morale = app.world().get::<Morale>(worker).unwrap();
    assert!(morale.value > 50.0);
}

#[test]
fn test_discordant_machines_grant_stress_debuff() {
    let mut app = App::new();
    app.add_systems(Update, process_industrial_rhythm);

    // Spawn two machines with different cycle times nearby
    app.world_mut().spawn((
        Building { type_: BuildingType::Factory },
        MachineCycle { current_tick: 0, max_ticks: 10 },
        GridPosition { x: 5, y: 5 },
    ));

    app.world_mut().spawn((
        Building { type_: BuildingType::Factory },
        MachineCycle { current_tick: 0, max_ticks: 11 }, // Out of sync!
        GridPosition { x: 6, y: 5 }, // Adjacent
    ));

    // Worker nearby
    let worker = app.world_mut().spawn((
        Pop,
        CabinFever { stress: 0.0 },
        GridPosition { x: 5, y: 5 },
    )).id();

    app.update();

    // The worker's stress should increase
    let fever = app.world().get::<CabinFever>(worker).unwrap();
    assert!(fever.stress > 0.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_industrial_rhythm(
    machine_query: Query<(&GridPosition, &MachineCycle)>,
    mut pop_query: Query<(&GridPosition, Option<&mut Morale>, Option<&mut CabinFever>), With<Pop>>,
) {
    let mut grid_rhythm = std::collections::HashMap::new();

    // Map machine cycles to positions
    for (pos, cycle) in machine_query.iter() {
        grid_rhythm.insert((pos.x, pos.y), cycle.max_ticks);
    }

    for (pop_pos, mut morale_opt, mut fever_opt) in pop_query.iter_mut() {
        // Find adjacent machines
        let mut local_cycles = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(cycle) = grid_rhythm.get(&(pop_pos.x + dx, pop_pos.y + dy)) {
                    local_cycles.push(*cycle);
                }
            }
        }

        if local_cycles.len() > 1 {
            let first = local_cycles[0];
            let all_sync = local_cycles.iter().all(|c| *c == first);

            if all_sync {
                if let Some(ref mut morale) = morale_opt {
                    morale.value += 1.0;
                }
            } else {
                if let Some(ref mut fever) = fever_opt {
                    fever.stress += 1.0;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an actual audio manager hook so the player hears the beat syncing up or falling apart.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Hook into existing Production chains.

## 8. Questions
*Builder: add questions here if spec is unclear.*
