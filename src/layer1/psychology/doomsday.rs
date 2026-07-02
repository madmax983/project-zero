use bevy_ecs::prelude::*;

use crate::layer1::economy::WorkEfficiency;
use crate::shared::time::SimulationTime;

#[derive(Component)]
pub struct DoomsdayBeliever {
    pub apocalypse_time: u64,
    pub original_productivity: f32,
}

#[derive(Component)]
pub struct NihilismDebuff {
    pub expires_at: u64,
    pub original_productivity: f32,
}

pub fn apply_doomsday_panic_effects(
    time: Res<SimulationTime>,
    mut query: Query<(&DoomsdayBeliever, &mut WorkEfficiency), Changed<DoomsdayBeliever>>,
) {
    for (believer, mut eff) in query.iter_mut() {
        if time.tick < believer.apocalypse_time {
            eff.multiplier = believer.original_productivity * 1.5;
        }
    }
}

pub fn resolve_doomsday_event(
    mut commands: Commands,
    time: Res<SimulationTime>,
    mut query: Query<(Entity, &DoomsdayBeliever, &mut WorkEfficiency)>,
) {
    for (entity, believer, mut eff) in query.iter_mut() {
        if time.tick >= believer.apocalypse_time {
            eff.multiplier = believer.original_productivity * 0.5;
            commands.entity(entity).remove::<DoomsdayBeliever>();
            commands.entity(entity).insert(NihilismDebuff {
                expires_at: time.tick + 50,
                original_productivity: believer.original_productivity,
            });
        }
    }
}

pub fn cleanup_nihilism_debuffs(
    mut commands: Commands,
    time: Res<SimulationTime>,
    mut query: Query<(Entity, &NihilismDebuff, &mut WorkEfficiency)>,
) {
    for (entity, debuff, mut eff) in query.iter_mut() {
        if time.tick >= debuff.expires_at {
            eff.multiplier = debuff.original_productivity;
            commands.entity(entity).remove::<NihilismDebuff>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::WorkEfficiency;
    use crate::layer1::entities::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_app() -> World {
        World::new()
    }

    #[test]
    fn test_doomsday_rumor_increases_productivity() {
        let mut world = setup_app();

        let pop = world.spawn((Pop, WorkEfficiency { multiplier: 1.0 })).id();

        world
            .entity_mut(pop)
            .insert(crate::layer1::psychology::doomsday::DoomsdayBeliever {
                apocalypse_time: 100,
                original_productivity: 1.0,
            });

        let time = crate::shared::time::SimulationTime { tick: 50, ..Default::default() };
        world.insert_resource(time);
        let _ = world
            .run_system_once(crate::layer1::psychology::doomsday::apply_doomsday_panic_effects);

        let eff = world.entity(pop).get::<WorkEfficiency>().unwrap();
        assert!(
            eff.multiplier > 1.0,
            "Productivity should increase during panic"
        );
    }

    #[test]
    fn test_doomsday_clock_hits_zero_causes_crash() {
        let mut world = setup_app();

        let pop = world
            .spawn((
                Pop,
                WorkEfficiency { multiplier: 1.5 },
                crate::layer1::psychology::doomsday::DoomsdayBeliever {
                    apocalypse_time: 100,
                    original_productivity: 1.0,
                },
            ))
            .id();

        let time = crate::shared::time::SimulationTime { tick: 101, ..Default::default() };
        world.insert_resource(time);
        let _ = world.run_system_once(crate::layer1::psychology::doomsday::resolve_doomsday_event);

        assert!(!world
            .entity(pop)
            .contains::<crate::layer1::psychology::doomsday::DoomsdayBeliever>());
        assert!(world
            .entity(pop)
            .contains::<crate::layer1::psychology::doomsday::NihilismDebuff>());

        let eff = world.entity(pop).get::<WorkEfficiency>().unwrap();
        assert!(
            eff.multiplier < 1.0,
            "Productivity should crash after false apocalypse"
        );
    }

    #[test]
    fn test_doomsday_crash_duration() {
        let mut world = setup_app();

        let pop = world
            .spawn((
                Pop,
                crate::layer1::psychology::doomsday::NihilismDebuff {
                    expires_at: 120,
                    original_productivity: 1.0,
                },
                WorkEfficiency { multiplier: 0.5 },
            ))
            .id();

        let time = crate::shared::time::SimulationTime { tick: 121, ..Default::default() };
        world.insert_resource(time);
        let _ =
            world.run_system_once(crate::layer1::psychology::doomsday::cleanup_nihilism_debuffs);

        assert!(!world
            .entity(pop)
            .contains::<crate::layer1::psychology::doomsday::NihilismDebuff>());
        let eff = world.entity(pop).get::<WorkEfficiency>().unwrap();
        assert_eq!(
            eff.multiplier, 1.0,
            "Original productivity should be restored"
        );
    }
}
