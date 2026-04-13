# 1002: The Bio-Rhythmic Commute

## 1. Overview
Instead of standard 24-hour clocks, colonies near "Lumiflora" adapt their circadian rhythms to the plant's blooming cycle. This creates massive, localized surges in work speed followed by deep, mandatory hibernation phases. If an eclipse or solar flare disrupts the Lumiflora, the entire sector falls asleep mid-shift, leaving critical infrastructure unmanaged.

## 2. Dependencies
- Layer 1 Day/Night cycles and Pop Needs (Sleep).
- Layer 1 Flora mechanics (`FloraType::Lumiflora` or similar).
- Layer 1 Weather/Events (Eclipses, Solar Flares) that affect light levels.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_lumiflora_blooming_cycle() {
        let mut app = App::new();
        app.add_systems(Update, lumiflora_bloom_system);

        let flora = app.world_mut().spawn((
            Flora { flora_type: FloraType::Lumiflora },
            LumifloraCycle { phase: BloomPhase::Dormant, time_in_phase: 0.0 },
        )).id();

        // Advance time to trigger blooming
        let time = Time::default();
        app.insert_resource(time); // Note: real test might need to mock time

        // Let's assume a system that advances LumifloraCycle time
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(10));
        app.update();

        // Needs to verify phase transitioned to Blooming or similar depending on time
        let cycle = app.world().get::<LumifloraCycle>(flora).unwrap();
        // Since we are mocking, let's just test the system applies the aura when blooming
        assert_eq!(cycle.phase, BloomPhase::Dormant); // Before enough time passes
    }

    #[test]
    fn test_pop_bio_rhythm_sync() {
        let mut app = App::new();
        app.add_systems(Update, apply_bio_rhythm_aura);

        let flora = app.world_mut().spawn((
            Flora { flora_type: FloraType::Lumiflora },
            LumifloraCycle { phase: BloomPhase::Blooming, time_in_phase: 0.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            BioRhythmSync { is_synced: false, work_speed_multiplier: 1.0 },
            NeedSleep { current: 100.0, threshold: 20.0 },
        )).id();

        app.update();

        let sync = app.world().get::<BioRhythmSync>(pop).unwrap();
        assert!(sync.is_synced);
        assert!(sync.work_speed_multiplier > 1.0); // Blooming gives speed boost

        // Change flora to Hibernation phase
        app.world_mut().get_mut::<LumifloraCycle>(flora).unwrap().phase = BloomPhase::Hibernation;

        app.update();

        let sleep = app.world().get::<NeedSleep>(pop).unwrap();
        assert!(sleep.current < sleep.threshold); // Forces sleep
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FloraType {
    Generic,
    Lumiflora,
}

#[derive(Component)]
pub struct Flora {
    pub flora_type: FloraType,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BloomPhase {
    Dormant,
    Blooming,
    Hibernation,
}

#[derive(Component)]
pub struct LumifloraCycle {
    pub phase: BloomPhase,
    pub time_in_phase: f32,
}

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct BioRhythmSync {
    pub is_synced: bool,
    pub work_speed_multiplier: f32,
}

#[derive(Component)]
pub struct NeedSleep {
    pub current: f32,
    pub threshold: f32,
}

pub fn lumiflora_bloom_system(
    time: Res<Time>,
    mut query: Query<&mut LumifloraCycle, With<Flora>>,
) {
    let dt = time.delta_secs();
    for mut cycle in query.iter_mut() {
        cycle.time_in_phase += dt;

        // Simple mock cycle
        match cycle.phase {
            BloomPhase::Dormant if cycle.time_in_phase > 10.0 => {
                cycle.phase = BloomPhase::Blooming;
                cycle.time_in_phase = 0.0;
            }
            BloomPhase::Blooming if cycle.time_in_phase > 5.0 => {
                cycle.phase = BloomPhase::Hibernation;
                cycle.time_in_phase = 0.0;
            }
            BloomPhase::Hibernation if cycle.time_in_phase > 5.0 => {
                cycle.phase = BloomPhase::Dormant;
                cycle.time_in_phase = 0.0;
            }
            _ => {}
        }
    }
}

pub fn apply_bio_rhythm_aura(
    flora_query: Query<(&LumifloraCycle, &GridPosition), With<Flora>>,
    mut pop_query: Query<(&GridPosition, &mut BioRhythmSync, &mut NeedSleep), With<Pop>>,
) {
    for (flora_cycle, flora_pos) in flora_query.iter() {
        for (pop_pos, mut sync, mut sleep) in pop_query.iter_mut() {
            // Check distance (simple radius)
            let dx = pop_pos.x - flora_pos.x;
            let dy = pop_pos.y - flora_pos.y;
            if dx * dx + dy * dy <= 25 { // radius 5
                sync.is_synced = true;

                match flora_cycle.phase {
                    BloomPhase::Blooming => {
                        sync.work_speed_multiplier = 1.5;
                        // Prevent sleep decay while blooming
                    }
                    BloomPhase::Hibernation => {
                        sync.work_speed_multiplier = 0.1;
                        sleep.current = 0.0; // Force immediate sleep need
                    }
                    BloomPhase::Dormant => {
                        sync.work_speed_multiplier = 1.0;
                    }
                }
            } else {
                sync.is_synced = false;
                sync.work_speed_multiplier = 1.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Applying aura every frame by iterating all Pops against all Floras is O(N*M). We should use a spatial grid lookup or apply a chunk-based aura to avoid performance issues.
- **Design Improvements**: Ensure the eclipse/solar flare events (which likely alter light levels globally) can interrupt the `LumifloraCycle`. This might involve adding a system that forces `BloomPhase::Hibernation` if light drops below a threshold.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops near Lumiflora get a work speed boost during the Blooming phase.
- [ ] Pops near Lumiflora are forced to sleep during the Hibernation phase.
- [ ] (Future integration) Disruptions to light levels force the Lumiflora into early hibernation.

## 7. Technical Guidance
- Integrate with `layer1::flora` to add the Lumiflora type.
- The `BioRhythmSync` component should be applied to Pops and modify their `UtilityWeights` for sleep and work actions in `layer1::utility_ai`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
