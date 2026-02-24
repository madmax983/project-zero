#[cfg(test)]
mod tests {
    use scale::layer1::justice::{Inmate, Wanted};
    use scale::layer1::map::GridPosition;
    use scale::layer1::pop::Pop;
    use scale::layer1::zone::{ZoneGrid, ZoneType};
    use scale::layer1::contraband::ContrabandPossession;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(scale::shared::time::SimulationTime::default());
        world.insert_resource(ZoneGrid::new(10, 10));
        world
    }

    #[test]
    fn test_contraband_triggers_wanted() {
        let mut world = setup_world();
        let pop = world
            .spawn((
                Pop,
                ContrabandPossession,
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Call the system under test
        world.run_system_once(scale::layer1::justice::check_contraband_crime_system).unwrap();

        // Assert Wanted component is added
        assert!(world.get::<Wanted>(pop).is_some());
    }

    #[test]
    fn test_inmates_ignored() {
        let mut world = setup_world();
        let inmate = world
            .spawn((
                Pop,
                ContrabandPossession,
                Inmate { sentence_ticks: 100 },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Call the system
        world.run_system_once(scale::layer1::justice::check_contraband_crime_system).unwrap();

        // Assert Wanted component is NOT added
        assert!(world.get::<Wanted>(inmate).is_none());
    }

    #[test]
    fn test_sanctuary_ignored() {
        let mut world = setup_world();
        world.resource_mut::<ZoneGrid>().set(0, 0, ZoneType::Sanctuary);

        let pop = world
            .spawn((
                Pop,
                ContrabandPossession,
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Call the system
        world.run_system_once(scale::layer1::justice::check_contraband_crime_system).unwrap();

        // Assert Wanted component is NOT added
        assert!(world.get::<Wanted>(pop).is_none());
    }
}
