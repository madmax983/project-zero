use bevy_ecs::prelude::*;
use crate::layer1::economy::WorkEfficiency;
use crate::shared::time::SimulationTime;

#[derive(Resource)]
pub struct DoomsdayConfig {
    pub panic_multiplier: f32,
    pub crash_multiplier: f32,
    pub debuff_duration: f32,
}

impl Default for DoomsdayConfig {
    fn default() -> Self {
        Self {
            panic_multiplier: 1.5,
            crash_multiplier: 0.5,
            debuff_duration: 50.0,
        }
    }
}

#[derive(Component)]
pub struct DoomsdayBeliever {
    pub apocalypse_time: f32,
    pub original_productivity: f32,
}

#[derive(Component)]
pub struct NihilismDebuff {
    pub expires_at: f32,
    pub original_productivity: f32,
}

pub fn apply_doomsday_panic_effects(
    time: Res<SimulationTime>,
    config: Res<DoomsdayConfig>,
    mut query: Query<(&DoomsdayBeliever, &mut WorkEfficiency), Added<DoomsdayBeliever>>
) {
    for (believer, mut efficiency) in query.iter_mut() {
        if (time.tick as f32) < believer.apocalypse_time {
            // Apply Panic productivity boost ONLY when the rumor is first believed
            efficiency.multiplier = believer.original_productivity * config.panic_multiplier;
        }
    }
}

pub fn resolve_doomsday_event(
    mut commands: Commands,
    time: Res<SimulationTime>,
    config: Res<DoomsdayConfig>,
    mut query: Query<(Entity, &DoomsdayBeliever, &mut WorkEfficiency)>
) {
    for (entity, believer, mut efficiency) in query.iter_mut() {
        if (time.tick as f32) >= believer.apocalypse_time {
            // Event didn't happen! Crash productivity and apply debuff
            efficiency.multiplier = believer.original_productivity * config.crash_multiplier;

            let debuff = NihilismDebuff {
                expires_at: (time.tick as f32) + config.debuff_duration,
                original_productivity: believer.original_productivity,
            };

            commands.entity(entity).remove::<DoomsdayBeliever>();
            commands.entity(entity).insert(debuff);
        }
    }
}

pub fn cleanup_nihilism_debuffs(
    mut commands: Commands,
    time: Res<SimulationTime>,
    mut query: Query<(Entity, &NihilismDebuff, &mut WorkEfficiency)>
) {
     for (entity, debuff, mut efficiency) in query.iter_mut() {
         if (time.tick as f32) >= debuff.expires_at {
             efficiency.multiplier = debuff.original_productivity; // Restore original productivity
             commands.entity(entity).remove::<NihilismDebuff>();
         }
     }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;
    use crate::layer1::pop::Pop;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.init_resource::<DoomsdayConfig>();
        world
    }

    #[test]
    fn test_doomsday_rumor_increases_productivity() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            WorkEfficiency { multiplier: 1.0 },
        )).id();

        // Act: Apply a Doomsday Clock rumor counting down to time 100
        world.entity_mut(pop).insert(DoomsdayBeliever {
            apocalypse_time: 100.0,
            original_productivity: 1.0,
        });

        // Advance simulation time (e.g., current time = 50.0)
        world.resource_mut::<SimulationTime>().tick = 50;
        world.run_system_once(apply_doomsday_panic_effects).unwrap();

        // Assert: Productivity should be spiked during the countdown
        let job = world.entity(pop).get::<WorkEfficiency>().unwrap();
        assert!(job.multiplier > 1.0, "Productivity should increase during panic");
    }

    #[test]
    fn test_doomsday_clock_hits_zero_causes_crash() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            WorkEfficiency { multiplier: 1.5 }, // Boosted from panic
            DoomsdayBeliever {
                apocalypse_time: 100.0,
                original_productivity: 1.0,
            },
        )).id();

        // Act: Advance time past the apocalypse
        world.resource_mut::<SimulationTime>().tick = 101;
        world.run_system_once(resolve_doomsday_event).unwrap();

        // Assert: The belief component is removed, and a debuff is applied
        assert!(!world.entity(pop).contains::<DoomsdayBeliever>());
        assert!(world.entity(pop).contains::<NihilismDebuff>());

        let job = world.entity(pop).get::<WorkEfficiency>().unwrap();
        assert!(job.multiplier < 1.0, "Productivity should crash after false apocalypse");
    }

    #[test]
    fn test_doomsday_crash_duration() {
         let mut world = setup_world();

         let pop = world.spawn((
             Pop,
             NihilismDebuff {
                 expires_at: 120.0,
                 original_productivity: 1.0,
             },
             WorkEfficiency { multiplier: 0.5 }
         )).id();

         // Act: Time passes the expiration
         world.resource_mut::<SimulationTime>().tick = 121;
         world.run_system_once(cleanup_nihilism_debuffs).unwrap();

         // Assert: Debuff removed, stats normalized
         assert!(!world.entity(pop).contains::<NihilismDebuff>());

         let eff = world.entity(pop).get::<WorkEfficiency>().unwrap();
         assert_eq!(eff.multiplier, 1.0, "Productivity should be restored");
    }
}
