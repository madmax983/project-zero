use bevy_ecs::prelude::*;
use crate::shared::time::SimulationTime;

#[derive(Resource)]
pub struct PlanetaryGravity {
    pub current: f32,
    pub base: f32,
}

impl Default for PlanetaryGravity {
    fn default() -> Self {
        Self { current: 1.0, base: 1.0 }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::time::SimulationTime;


    #[test]
    fn test_syzygy_trigger_reduces_gravity_penalty() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 100, speed: crate::shared::time::SimSpeed::Normal });
        world.insert_resource(SyzygyCycle { current_phase: SyzygyPhase::Active, next_syzygy_tick: 1000 });
        world.insert_resource(TidalForce::default());
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
        world.insert_resource(SyzygyCycle { current_phase: SyzygyPhase::Active, next_syzygy_tick: 1000 });
        world.insert_resource(TidalForce::default());
        world.insert_resource(PlanetaryGravity::default());
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
        world.insert_resource(SimulationTime { tick: 999, speed: crate::shared::time::SimSpeed::Normal });
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
