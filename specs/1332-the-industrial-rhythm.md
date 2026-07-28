# 1332: The Industrial Rhythm

## 1. Overview
The factory is a symphony. Machines have "Cycle Times". If adjacent machines are synchronized (finishing at the same time), they create "Rhythm", which boosts worker Morale. Discordant noise (mismatched cycle times) increases Stress.

## 2. Dependencies
- Layer 1 Core Architecture (`Building`, `GridPosition`)
- Layer 1 Population (`Pop`, `Morale`, `Stress`)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_synchronized_machines_generate_rhythm() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_industrial_rhythm_system);

        let machine1 = app.world_mut().spawn((
            Machine { cycle_time: 10, current_tick: 5 },
            GridPosition { x: 10, y: 10, z: 0 },
        )).id();

        let machine2 = app.world_mut().spawn((
            Machine { cycle_time: 10, current_tick: 5 },
            GridPosition { x: 10, y: 11, z: 0 },
        )).id();

        app.update();

        let rhythm = app.world().get::<RhythmGenerator>(machine1);
        assert!(rhythm.is_some(), "Synchronized adjacent machines should generate rhythm");
    }

    #[test]
    fn test_discordant_machines_generate_noise() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_industrial_rhythm_system);

        let machine1 = app.world_mut().spawn((
            Machine { cycle_time: 10, current_tick: 5 },
            GridPosition { x: 10, y: 10, z: 0 },
        )).id();

        let machine2 = app.world_mut().spawn((
            Machine { cycle_time: 15, current_tick: 5 },
            GridPosition { x: 10, y: 11, z: 0 },
        )).id();

        app.update();

        let noise = app.world().get::<NoiseGenerator>(machine1);
        assert!(noise.is_some(), "Discordant adjacent machines should generate noise");
    }

    #[test]
    fn test_rhythm_boosts_morale() {
        let mut app = App::new();
        app.add_systems(Update, apply_rhythm_effects_system);

        let pop = app.world_mut().spawn((
            Pop,
            GridPosition { x: 10, y: 10, z: 0 },
            Morale(50.0),
        )).id();

        let _machine = app.world_mut().spawn((
            RhythmGenerator,
            GridPosition { x: 10, y: 10, z: 0 },
        )).id();

        app.update();

        let morale = app.world().get::<Morale>(pop).unwrap();
        assert!(morale.0 > 50.0, "Rhythm should boost morale of nearby pops");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Component)]
pub struct Morale(pub f32);

#[derive(Component)]
pub struct Stress(pub f32);

#[derive(Component)]
pub struct Machine {
    pub cycle_time: u32,
    pub current_tick: u32,
}

#[derive(Component)]
pub struct RhythmGenerator;

#[derive(Component)]
pub struct NoiseGenerator;

pub fn evaluate_industrial_rhythm_system(
    mut commands: Commands,
    machines: Query<(Entity, &Machine, &GridPosition)>,
) {
    let machines_vec: Vec<_> = machines.iter().collect();
    for i in 0..machines_vec.len() {
        let (e1, m1, p1) = machines_vec[i];
        let mut is_sync = false;
        let mut is_discordant = false;

        for j in 0..machines_vec.len() {
            if i == j { continue; }
            let (_e2, m2, p2) = machines_vec[j];

            let dist = (p1.x - p2.x).abs() + (p1.y - p2.y).abs();
            if dist <= 1 {
                if m1.cycle_time == m2.cycle_time {
                    is_sync = true;
                } else {
                    is_discordant = true;
                }
            }
        }

        if is_sync && !is_discordant {
            commands.entity(e1).insert(RhythmGenerator);
            commands.entity(e1).remove::<NoiseGenerator>();
        } else if is_discordant {
            commands.entity(e1).insert(NoiseGenerator);
            commands.entity(e1).remove::<RhythmGenerator>();
        }
    }
}

pub fn apply_rhythm_effects_system(
    mut pops: Query<(&GridPosition, &mut Morale, &mut Stress)>,
    rhythm_generators: Query<&GridPosition, With<RhythmGenerator>>,
    noise_generators: Query<&GridPosition, With<NoiseGenerator>>,
) {
    for (pop_pos, mut morale, mut stress) in pops.iter_mut() {
        for rhythm_pos in rhythm_generators.iter() {
            let dist = (pop_pos.x - rhythm_pos.x).abs() + (pop_pos.y - rhythm_pos.y).abs();
            if dist <= 2 {
                morale.0 += 1.0;
            }
        }

        for noise_pos in noise_generators.iter() {
            let dist = (pop_pos.x - noise_pos.x).abs() + (pop_pos.y - noise_pos.y).abs();
            if dist <= 2 {
                stress.0 += 1.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Grid:** Use a spatial hash or spatial index rather than an O(N^2) comparison for machine adjacency.
- **Modifiers:** Implement maximum threshold caps so Morale doesn't scale to infinity.
- **Rhythm Definition:** Refine "synchronized" to mean modulo of tick rather than just equal cycle time so we account for harmonic cycles.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Adjacent synchronized machines generate `RhythmGenerator`.
- [ ] Adjacent discordant machines generate `NoiseGenerator`.
- [ ] Pops near generators receive correct mood alterations.

## 7. Technical Guidance
- Adhere to the existing `GridPosition` if defined in `src/layer1/map.rs`. Replace mock structures in GREEN phase with actual domain structures from the codebase.
- Rhythm check system could be scheduled in `SimulationSchedule::Update`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
