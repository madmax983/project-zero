use bevy_ecs::prelude::*;

use crate::shared::time::SimulationTime;

#[derive(Component, Debug, Clone)]
pub struct DoomsdayBeliever {
    pub apocalypse_time: u64,
}

#[derive(Component, Debug, Clone)]
pub struct NihilismDebuff {
    pub expires_at: u64,
}

#[derive(Component, Debug, Clone)]
pub struct DoomsdayWorkBuff {
    pub multiplier: f32,
}

pub fn apply_doomsday_panic_effects(
    time: Res<SimulationTime>,
    mut commands: Commands,
    query: Query<(Entity, &DoomsdayBeliever), Without<DoomsdayWorkBuff>>,
) {
    for (entity, believer) in query.iter() {
        if time.tick < believer.apocalypse_time {
            // Panic productivity boost
            commands.entity(entity).insert(DoomsdayWorkBuff {
                multiplier: 1.5,
            });
        }
    }
}

pub fn resolve_doomsday_event(
    mut commands: Commands,
    time: Res<SimulationTime>,
    query: Query<(Entity, &DoomsdayBeliever)>,
) {
    for (entity, believer) in query.iter() {
        if time.tick >= believer.apocalypse_time {
            // Event didn't happen! Crash productivity and apply debuff
            commands.entity(entity).remove::<DoomsdayBeliever>();
            commands.entity(entity).insert(DoomsdayWorkBuff {
                multiplier: 0.5,
            });
            commands.entity(entity).insert(NihilismDebuff {
                expires_at: time.tick + 50, // Arbitrary duration
            });
        }
    }
}

pub fn cleanup_nihilism_debuffs(
    mut commands: Commands,
    time: Res<SimulationTime>,
    query: Query<(Entity, &NihilismDebuff)>,
) {
    for (entity, debuff) in query.iter() {
        if time.tick >= debuff.expires_at {
            commands.entity(entity).remove::<NihilismDebuff>();
            commands.entity(entity).remove::<DoomsdayWorkBuff>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world
    }

    #[test]
    fn test_doomsday_rumor_increases_productivity() {
        let mut world = setup_world();

        let pop = world.spawn(
            DoomsdayBeliever {
                apocalypse_time: 100,
            },
        ).id();

        world.resource_mut::<SimulationTime>().tick = 50;
        let _ = world.run_system_once(apply_doomsday_panic_effects);

        let buff = world.get::<DoomsdayWorkBuff>(pop).unwrap();
        assert!(buff.multiplier > 1.0, "Productivity should increase during panic");
    }

    #[test]
    fn test_doomsday_clock_hits_zero_causes_crash() {
        let mut world = setup_world();

        let pop = world.spawn(
            DoomsdayBeliever {
                apocalypse_time: 100,
            },
        ).id();

        world.resource_mut::<SimulationTime>().tick = 101;
        let _ = world.run_system_once(resolve_doomsday_event);

        assert!(world.get::<DoomsdayBeliever>(pop).is_none());
        assert!(world.get::<NihilismDebuff>(pop).is_some());

        let buff = world.get::<DoomsdayWorkBuff>(pop).unwrap();
        assert!(buff.multiplier < 1.0, "Productivity should crash after false apocalypse");
    }

    #[test]
    fn test_doomsday_crash_duration() {
         let mut world = setup_world();

         let pop = world.spawn(
             NihilismDebuff {
                 expires_at: 120,
             }
         ).id();
         world.entity_mut(pop).insert(DoomsdayWorkBuff { multiplier: 0.5 });

         world.resource_mut::<SimulationTime>().tick = 121;
         let _ = world.run_system_once(cleanup_nihilism_debuffs);

         assert!(world.get::<NihilismDebuff>(pop).is_none());
         assert!(world.get::<DoomsdayWorkBuff>(pop).is_none());
    }
}
