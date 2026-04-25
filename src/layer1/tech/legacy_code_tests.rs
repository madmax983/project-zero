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
