# The Syzygy

**1. Overview**
Planetary orbits periodically align to form a "Grand Conjunction" known as The Syzygy. During this window, celestial forces amplify specific mechanics: Psionic powers are boosted, Gravity is reduced (enabling cheap launches), and Tides are extreme. Players must balance the immense benefits of timing launches with the catastrophic risks of extreme environmental phenomena like flooding.

**2. Dependencies**
- `src/layer2/orbits.rs` (Planetary orbital cycles)
- `src/layer1/tides.rs` (Tidal systems and flooding)
- `src/shared/time.rs` (Simulation time)
- `src/layer2/trade/escape_velocity.rs` (Launch costs)

**3. RED Phase: Tests First**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::time::SimulationTime;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_syzygy_trigger_reduces_gravity_penalty() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 100 });
        world.insert_resource(SyzygyCycle { current_phase: SyzygyPhase::Active });
        world.insert_resource(PlanetaryGravity { current: 2.0, base: 2.0 });

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_syzygy_effects_system);

        // Act
        schedule.run(&mut world);

        // Assert
        let gravity = world.resource::<PlanetaryGravity>();
        assert!(gravity.current < gravity.base, "Syzygy should reduce gravity");
    }

    #[test]
    fn test_syzygy_trigger_amplifies_tides() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SyzygyCycle { current_phase: SyzygyPhase::Active });
        world.insert_resource(TidalForce { current: 1.0, base: 1.0 });

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_syzygy_effects_system);

        // Act
        schedule.run(&mut world);

        // Assert
        let tide = world.resource::<TidalForce>();
        assert!(tide.current > tide.base, "Syzygy should amplify tidal forces");
    }

    #[test]
    fn test_syzygy_phase_progression() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 999 });
        world.insert_resource(SyzygyCycle { current_phase: SyzygyPhase::Inactive, next_syzygy_tick: 1000 });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_syzygy_cycle_system);

        // Act: Move to tick 1000
        world.resource_mut::<SimulationTime>().tick = 1000;
        schedule.run(&mut world);

        // Assert
        let cycle = world.resource::<SyzygyCycle>();
        assert_eq!(cycle.current_phase, SyzygyPhase::Active, "Cycle should transition to Active");
    }
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
use bevy_ecs::prelude::*;
use crate::shared::time::SimulationTime;

#[derive(Resource, Default)]
pub struct PlanetaryGravity {
    pub current: f32,
    pub base: f32,
}

#[derive(Resource, Default)]
pub struct TidalForce {
    pub current: f32,
    pub base: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyzygyPhase {
    Inactive,
    Active,
}

#[derive(Resource)]
pub struct SyzygyCycle {
    pub current_phase: SyzygyPhase,
    pub next_syzygy_tick: u64,
}

impl Default for SyzygyCycle {
    fn default() -> Self {
        Self {
            current_phase: SyzygyPhase::Inactive,
            next_syzygy_tick: 10000,
        }
    }
}

pub fn update_syzygy_cycle_system(
    time: Res<SimulationTime>,
    mut cycle: ResMut<SyzygyCycle>,
) {
    if time.tick >= cycle.next_syzygy_tick {
        cycle.current_phase = SyzygyPhase::Active;
    } else {
        cycle.current_phase = SyzygyPhase::Inactive;
    }
}

pub fn apply_syzygy_effects_system(
    cycle: Res<SyzygyCycle>,
    mut gravity: ResMut<PlanetaryGravity>,
    mut tide: ResMut<TidalForce>,
) {
    if cycle.current_phase == SyzygyPhase::Active {
        gravity.current = gravity.base * 0.5; // Halve gravity
        tide.current = tide.base * 2.0;       // Double tides
    } else {
        gravity.current = gravity.base;
        tide.current = tide.base;
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- **Performance:** `apply_syzygy_effects_system` currently runs every tick. It can be optimized with `run_if` conditions based on phase changes (e.g., listening to an event rather than querying the state constantly).
- **Extensibility:** The `SyzygyCycle` could emit a `SyzygyEvent` that multiple systems (e.g., Psionics, Tides, Launch Costs) listen to. This decouples the effects logic.
- **Lore Integration:** Link to the Chronicle system when Syzygy starts to record the grand alignment.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Specific feature behavior (gravity reduction and tide amplification) verified via tests.

**7. Technical Guidance**
- Integrate with `SimulationSchedule` in `src/simulation.rs`.
- Place the core logic under `src/layer2/syzygy.rs`.
- Create a bridge system in `src/layer2/integration.rs` if Layer 1 needs to know about the alignment (e.g., UI notifications or flooding trigger).

**8. Questions**
*Builder: add questions here if spec is unclear.*
