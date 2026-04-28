#[cfg(test)]
mod tests {

    use scale::layer1::architecture::building::{Building, BuildingType};
    use scale::layer1::core::events::BuildingCompletedEvent;
    use scale::layer3::bureaucracy_of_vanity::{
        ActiveDemands, GlobalEfficiency,
        ImperialStanding, VanityProject,
    };
    use scale::prelude::*;

    #[test]
    fn test_bureaucracy_of_vanity_fulfilled_in_simulation() {
        let mut world = setup_world_with_config(SetupConfig::default());
        // Since setup_world_with_config sets up everything, the schedule is in Schedules.

        world.insert_resource(ActiveDemands {
            vanity_demand_active: true,
            time_since_demand: 0.0,
            active_governor_entity: None,
        });

        let initial_standing = world.resource::<ImperialStanding>().value;

        // Spawn a Statue building that was just completed.
        let statue = world
            .spawn((
                Building {
                    building_type: BuildingType::Statue,
                },
                VanityProject,
            ))
            .id();

        world.send_event(BuildingCompletedEvent { entity: statue });

        // Advance tick to process events
        run_simulation_tick(&mut world);

        // Assert standing increased and demand cleared.
        let final_standing = world.resource::<ImperialStanding>().value;
        assert_eq!(final_standing, initial_standing + 10);
        assert!(!world.resource::<ActiveDemands>().vanity_demand_active);
    }

    #[test]
    fn test_bureaucracy_of_vanity_sabotage_in_simulation() {
        let mut world = setup_world_with_config(SetupConfig::default());

        world.insert_resource(ActiveDemands {
            vanity_demand_active: true,
            time_since_demand: 99.0,
            active_governor_entity: None,
        });

        world.insert_resource(GlobalEfficiency { value: 1.0 });

        // In simulation, Time delta is 1/60th of a second in headless by default (0.016)
        // Wait, time might not tick naturally in simulation tick for this.
        // Let's modify time resource directly.
        let mut time = world.resource_mut::<bevy_time::Time>();
        // advance by 2 seconds
        let dt = std::time::Duration::from_secs(2);
        time.advance_by(dt);

        run_simulation_tick(&mut world);

        // Time since demand should exceed 100 and efficiency should be reduced.
        let eff = world.resource::<GlobalEfficiency>().value;
        assert!(eff < 1.0, "Efficiency should have been sabotaged");
    }
}
