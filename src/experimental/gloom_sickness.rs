#![allow(clippy::type_complexity)]
//! Gloom Sickness (Nova Feature).
//!
//! # The Spark
//! We have a `LightMap` and `Pop` entities with `Speed`. What if pops standing in darkness for too long contract an ailment?
//!
//! # The Feature
//! A system that tracks how long pops are in the dark, giving them a `GloomSickness` component when they hit a threshold.
//! This ailment passively lowers their speed until they either die or wait it out (for the MVP, it slowly decays in the light).

use crate::layer1::entities::pop::Pop;
use crate::layer1::lighting::LightMap;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Speed;
use bevy_ecs::prelude::*;

#[derive(Component, Default)]
pub struct GloomSickness {
    pub severity: f32, // 0.0 to 1.0
}

/// Applies gloom sickness to pops standing in the dark.
pub fn apply_gloom_sickness_system(
    mut commands: Commands,
    light_map: Res<LightMap>,
    mut query: Query<(Entity, &GridPosition, Option<&mut GloomSickness>), With<Pop>>,
) {
    for (entity, pos, sickness_opt) in query.iter_mut() {
        let x = pos.x as u32;
        let y = pos.y as u32;
        let light = light_map.get(x, y);

        if light < 0.1 {
            // It's dark
            if let Some(mut sickness) = sickness_opt {
                sickness.severity = (sickness.severity + 0.05).min(1.0);
            } else {
                commands
                    .entity(entity)
                    .insert(GloomSickness { severity: 0.05 });
            }
        } else {
            // It's bright
            if let Some(mut sickness) = sickness_opt {
                sickness.severity = (sickness.severity - 0.01).max(0.0);
                if sickness.severity <= 0.0 {
                    commands.entity(entity).remove::<GloomSickness>();
                }
            }
        }
    }
}

/// Penalizes speed for pops with gloom sickness.
pub fn gloom_sickness_speed_penalty_system(mut query: Query<(&GloomSickness, &mut Speed)>) {
    for (sickness, mut speed) in query.iter_mut() {
        // Up to 50% speed penalty
        let penalty = 1.0 - (sickness.severity * 0.5);
        speed.current *= penalty;
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((
        apply_gloom_sickness_system,
        gloom_sickness_speed_penalty_system.after(crate::layer1::pop::reset_speed_system),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_gloom_sickness_application() {
        let mut world = World::new();

        let mut light_map = LightMap::new(10, 10);
        // Set tile (0, 0) to pitch black
        light_map.set(0, 0, 0.0);
        // Set tile (1, 1) to bright
        light_map.set(1, 1, 1.0);

        world.insert_resource(light_map);

        let pop1 = world.spawn((Pop, GridPosition { x: 0, y: 0 })).id();
        let pop2 = world.spawn((Pop, GridPosition { x: 1, y: 1 })).id();

        world.run_system_once(apply_gloom_sickness_system).unwrap();

        // pop1 should have contracted GloomSickness
        assert!(world.get::<GloomSickness>(pop1).is_some());

        // pop2 should be healthy
        assert!(world.get::<GloomSickness>(pop2).is_none());
    }

    #[test]
    fn test_gloom_sickness_speed_penalty() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                GloomSickness { severity: 1.0 },
                Speed {
                    base: 1.0,
                    current: 1.0,
                    accumulator: 0.0,
                },
            ))
            .id();

        world
            .run_system_once(gloom_sickness_speed_penalty_system)
            .unwrap();

        let speed = world.get::<Speed>(pop).unwrap();
        assert!((speed.current - 0.5).abs() < f32::EPSILON);
    }
}
