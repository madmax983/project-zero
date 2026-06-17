#[cfg(test)]
mod tests {
    use crate::layer1::tech::legacy_code::{update_bloat_system, Bloat};
    use bevy_ecs::prelude::*;
    // use crate::layer1::research::ResearchRate; // Not yet implemented
    use crate::shared::time::SimulationTime;

    use crate::shared::time::SimSpeed;

    #[test]
    fn test_bloat_accumulation() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 100,
            speed: SimSpeed::Normal,
        });

        let mainframe = world
            .spawn(Bloat {
                current: 0.0,
                rate: 0.1,
                reboot_ticks: 0,
            })
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_bloat_system);
        schedule.run(&mut world);

        let bloat = world.get::<Bloat>(mainframe).unwrap();
        assert!(bloat.current > 0.0);
    }

    #[test]
    fn test_bloat_slows_research() {
        // This test assumes a system modifies ResearchRate based on Bloat
        // or we test the helper function `calculate_efficiency`

        let bloat_low = Bloat {
            current: 10.0,
            rate: 0.0,
            reboot_ticks: 0,
        };
        assert!(bloat_low.efficiency() > 0.9);

        let bloat_high = Bloat {
            current: 90.0,
            rate: 0.0,
            reboot_ticks: 0,
        };
        assert!(bloat_high.efficiency() < 0.2);
    }

    #[test]
    fn test_reformat_clears_bloat_but_disables_system() {
        let mut world = World::new();
        let mainframe = world
            .spawn(Bloat {
                current: 100.0,
                rate: 0.1,
                reboot_ticks: 0,
            })
            .id();

        // Trigger Reformat
        if let Some(mut bloat) = world.get_mut::<Bloat>(mainframe) {
            bloat.reboot_ticks = 500; // 500 ticks duration
        }

        let bloat = world.get::<Bloat>(mainframe).unwrap();
        assert_eq!(bloat.reboot_ticks, 500); // 500 ticks duration

        // Advance time to finish
        // (Mocking system update for reboot logic, setting to 1 so the system clears it this tick)
        if let Some(mut bloat) = world.get_mut::<Bloat>(mainframe) {
            if bloat.reboot_ticks > 0 {
                bloat.reboot_ticks = 1;
            }
        }

        let mut schedule = Schedule::default();
        schedule.add_systems(update_bloat_system);
        schedule.run(&mut world);

        let bloat = world.get::<Bloat>(mainframe).unwrap();
        assert_eq!(bloat.current, 0.0);
        assert_eq!(bloat.reboot_ticks, 0);
    }
}

#[cfg(test)]
mod legacy_code_spec_tests {
    use crate::layer1::architecture::turret::Turret;
    use crate::layer1::combat::AttackProperties;
    use crate::layer1::tech::legacy_code::*;
    use crate::shared::time::SimulationTime;
    use bevy_app::prelude::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_bloat_accumulation_increases_turret_latency() {
        let mut app = App::new();
        app.insert_resource(SimulationTime {
            tick: 0,
            speed: crate::shared::time::SimSpeed::Normal,
        });
        app.add_systems(
            Update,
            (accumulate_bloat_system, apply_latency_system).chain(),
        );

        // Setup Core
        app.world_mut().spawn((
            ComputerCore,
            SystemBloat {
                amount: 0.0,
                accumulation_rate: 1.0,
            },
            CoreStatus::Online,
        ));

        // Setup Turret
        let turret = app
            .world_mut()
            .spawn((
                Turret {
                    attack: AttackProperties {
                        damage: 10.0,
                        range: 10.0,
                        cooldown: 10,
                        accuracy: 1.0,
                    },
                    ammo_cost: 1.0,
                    ammo_type: crate::layer1::economy::resources::ResourceType::Waste,
                },
                ActionLatency { delay_ticks: 0 },
            ))
            .id();

        // Simulate 100 ticks passing
        for _ in 0..100 {
            app.world_mut().resource_mut::<SimulationTime>().tick += 1;
            app.update();
        }

        let latency = app.world().get::<ActionLatency>(turret).unwrap();
        assert!(
            latency.delay_ticks > 0,
            "Turret latency should increase as System Bloat accumulates."
        );
    }

    #[test]
    fn test_reformatting_clears_bloat_but_disables_core() {
        let mut app = App::new();
        app.add_event::<ReformatCommand>();
        app.add_systems(Update, process_reformat_system);

        let core = app
            .world_mut()
            .spawn((
                ComputerCore,
                SystemBloat {
                    amount: 100.0,
                    accumulation_rate: 1.0,
                },
                CoreStatus::Online,
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<ReformatCommand>>()
            .send(ReformatCommand { core_entity: core });

        app.update();

        let bloat = app.world().get::<SystemBloat>(core).unwrap();
        let status = app.world().get::<CoreStatus>(core).unwrap();

        assert_eq!(bloat.amount, 0.0, "Reformatting should clear all bloat.");
        assert_eq!(
            *status,
            CoreStatus::OfflineRebooting,
            "Reformatting must take the core offline."
        );
    }
}
