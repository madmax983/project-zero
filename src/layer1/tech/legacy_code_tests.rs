#[cfg(test)]
mod tests {
    use crate::layer1::core::colony::ComputerCore;
    use crate::layer1::tech::legacy_code::*;
    use crate::shared::time::SimulationTime;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_bloat_accumulation_increases_turret_latency() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        world.insert_resource(SimulationTime {
            tick: 0,
            ..Default::default()
        });
        schedule.add_systems((accumulate_bloat_system, apply_latency_system).chain());

        // Setup Core
        let _core = world
            .spawn((
                ComputerCore,
                SystemBloat {
                    amount: 0.0,
                    accumulation_rate: 1.0,
                },
                CoreStatus::Online,
            ))
            .id();

        // Setup Turret
        let turret = world.spawn((ActionLatency { delay_ticks: 0 },)).id();

        // Simulate 100 ticks passing
        for _ in 0..100 {
            world.resource_mut::<SimulationTime>().tick += 1;
            schedule.run(&mut world);
        }

        let latency = world.get::<ActionLatency>(turret).unwrap();
        assert!(
            latency.delay_ticks > 0,
            "Turret latency should increase as System Bloat accumulates."
        );
    }

    #[test]
    fn test_reformatting_clears_bloat_but_disables_core() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        world.init_resource::<Events<ReformatCommand>>();
        schedule.add_systems(process_reformat_system);

        let core = world
            .spawn((
                ComputerCore,
                SystemBloat {
                    amount: 100.0,
                    accumulation_rate: 1.0,
                },
                CoreStatus::Online,
            ))
            .id();

        world
            .resource_mut::<Events<ReformatCommand>>()
            .send(ReformatCommand { core_entity: core });

        schedule.run(&mut world);

        let bloat = world.get::<SystemBloat>(core).unwrap();
        let status = world.get::<CoreStatus>(core).unwrap();

        assert_eq!(bloat.amount, 0.0, "Reformatting should clear all bloat.");
        assert_eq!(
            *status,
            CoreStatus::OfflineRebooting(600),
            "Reformatting must take the core offline."
        );
    }
}
