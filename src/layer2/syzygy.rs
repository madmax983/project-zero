use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct SyzygyActive {
    pub duration: u32,
}

#[derive(Component)]
pub struct MutatedTrait;

use crate::layer1::chronicle::{Chronicle, EventImportance};
use crate::layer2::alignment::PlanetaryAlignment;
use crate::layer1::entities::pop::Speed;
use crate::layer1::utility_types::ActionType;
use crate::layer1::utility_types::PopAction;
use rand::Rng;

pub fn trigger_syzygy(world: &mut World) {
    let entities: Vec<Entity> = world.query_filtered::<Entity, With<PlanetaryAlignment>>().iter(world).collect();
    for entity in entities {
        if let Some(alignment) = world.get::<PlanetaryAlignment>(entity) {
            if alignment.days_until == 0 {
                world.insert_resource(SyzygyActive { duration: 30 });
                let mut chronicle = world.get_resource_mut::<Chronicle>().unwrap();
                chronicle.add_event(0, "syzygy_begins".to_string(), EventImportance::Standard);
            }
        }
    }
}

pub fn apply_syzygy_effects(world: &mut World) {
    let mut duration = 0;
    if let Some(mut active) = world.get_resource_mut::<SyzygyActive>() {
        if active.duration > 0 {
            active.duration -= 1;
            duration = active.duration;
        }
    }

    if duration == 0 {
        world.remove_resource::<SyzygyActive>();
    }

    if world.contains_resource::<SyzygyActive>() {
        for (action, mut speed) in world.query::<(&PopAction, &mut Speed)>().iter_mut(world) {
            if action.current == ActionType::Haul {
                speed.current = speed.base + 5.0; // Boost hauling speed
            } else {
                speed.current = speed.base;
            }
        }
    } else {
        // Reset speeds
        for mut speed in world.query::<&mut Speed>().iter_mut(world) {
            speed.current = speed.base;
        }
    }
}

pub fn trigger_random_mutation(world: &mut World) {
    let entities: Vec<Entity> = world.query_filtered::<Entity, With<crate::layer1::pop::Pop>>().iter(world).collect();
    for entity in entities {
        if world.contains_resource::<SyzygyActive>() {
            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.1) { // 10% chance
                world.entity_mut(entity).insert(MutatedTrait);
            }
        }
    }
}

#[derive(Resource)]
pub struct PlanetaryGravity {
    pub current: f32,
    pub base: f32,
}

impl Default for PlanetaryGravity {
    fn default() -> Self {
        Self {
            current: 1.0,
            base: 1.0,
        }
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

pub fn update_syzygy_cycle_system(time: Res<SimulationTime>, mut cycle: ResMut<SyzygyCycle>) {
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
        tide.current = tide.base * 2.0; // Double tides
    } else {
        gravity.current = gravity.base;
        tide.current = tide.base;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer2::alignment::PlanetaryAlignment;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::chronicle::Chronicle;

    fn setup_test_world() -> World {
        let mut world = World::new();
        world.insert_resource(Chronicle::default());
        world
    }

    #[test]
    fn test_syzygy_trigger() {
        let mut world = setup_test_world();
        let _alignment = world.spawn(PlanetaryAlignment { days_until: 0 }).id();

        trigger_syzygy(&mut world);

        assert!(world.contains_resource::<SyzygyActive>());
        assert!(world.resource::<Chronicle>().events.iter().any(|e| e.text == "syzygy_begins"));
    }

    #[test]
    fn test_syzygy_hauling_boost() {
        let mut world = setup_test_world();
        world.insert_resource(SyzygyActive { duration: 10 });
        let hauler = world.spawn((crate::layer1::entities::pop::Speed { base: 1.0, current: 1.0, accumulator: 0.0 }, crate::layer1::utility_types::PopAction { current: crate::layer1::utility_types::ActionType::Haul, ..Default::default() })).id();

        apply_syzygy_effects(&mut world);

        let boosted_speed = world.get::<crate::layer1::entities::pop::Speed>(hauler).unwrap().current;
        assert!(boosted_speed > 1.0);
    }

    #[test]
    fn test_syzygy_mutation_chance() {
        let mut world = setup_test_world();
        world.insert_resource(SyzygyActive { duration: 10 });
        let pop = world.spawn((Pop, GridPosition { x: 5, y: 5 })).id();

        // Trigger it many times to ensure the 10% chance procs at least once in tests
        for _ in 0..100 {
            trigger_random_mutation(&mut world);
        }

        assert!(world.get::<MutatedTrait>(pop).is_some());
    }
    use crate::shared::time::SimulationTime;


    #[test]
    fn test_syzygy_trigger_reduces_gravity_penalty() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 100,
            speed: crate::shared::time::SimSpeed::Normal,
        });
        world.insert_resource(SyzygyCycle {
            current_phase: SyzygyPhase::Active,
            next_syzygy_tick: 1000,
        });
        world.insert_resource(TidalForce::default());
        world.insert_resource(PlanetaryGravity {
            current: 2.0,
            base: 2.0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_syzygy_effects_system);

        // Act
        schedule.run(&mut world);

        // Assert
        let gravity = world.resource::<PlanetaryGravity>();
        assert!(
            gravity.current < gravity.base,
            "Syzygy should reduce gravity"
        );
    }

    #[test]
    fn test_syzygy_trigger_amplifies_tides() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SyzygyCycle {
            current_phase: SyzygyPhase::Active,
            next_syzygy_tick: 1000,
        });
        world.insert_resource(TidalForce::default());
        world.insert_resource(PlanetaryGravity::default());
        world.insert_resource(TidalForce {
            current: 1.0,
            base: 1.0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_syzygy_effects_system);

        // Act
        schedule.run(&mut world);

        // Assert
        let tide = world.resource::<TidalForce>();
        assert!(
            tide.current > tide.base,
            "Syzygy should amplify tidal forces"
        );
    }

    #[test]
    fn test_syzygy_phase_progression() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 999,
            speed: crate::shared::time::SimSpeed::Normal,
        });
        world.insert_resource(SyzygyCycle {
            current_phase: SyzygyPhase::Inactive,
            next_syzygy_tick: 1000,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_syzygy_cycle_system);

        // Act: Move to tick 1000
        world.resource_mut::<SimulationTime>().tick = 1000;
        schedule.run(&mut world);

        // Assert
        let cycle = world.resource::<SyzygyCycle>();
        assert_eq!(
            cycle.current_phase,
            SyzygyPhase::Active,
            "Cycle should transition to Active"
        );
    }
}
