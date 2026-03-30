#[cfg(test)]
mod tests {
use crate::layer1::justice::{check_crime_system, evaluate_warden_action, Wanted};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::unrest::{MentalBreakType, MentalState};
    use crate::layer1::utility_eval_types::ScorableCandidate;
    use crate::layer1::zone::{ZoneGrid, ZoneType};
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        let mut world = World::new();
        let mut zone_grid = ZoneGrid::new(10, 10);

        // Define Sanctuary Zone at (5,5)
        zone_grid.set(5, 5, ZoneType::Sanctuary);

        world.insert_resource(zone_grid);
        world
    }

    #[test]
    fn test_crime_in_sanctuary_ignored() {
        let mut world = setup_world();

        // Vandal inside Sanctuary
        let vandal = world
            .spawn((
                Pop,
                MentalState::Broken(MentalBreakType::Vandalize),
                GridPosition { x: 5, y: 5 }, // Inside Sanctuary
            ))
            .id();

        // Run detection system
        let mut schedule = Schedule::default();
        schedule.add_systems(check_crime_system);
        schedule.run(&mut world);

        // Assert NOT Wanted
        assert!(
            world.get::<Wanted>(vandal).is_none(),
            "Pop in Sanctuary should not be marked Wanted"
        );
    }

    #[test]
    fn test_crime_outside_sanctuary_punished() {
        let mut world = setup_world();

        // Vandal outside Sanctuary
        let vandal = world
            .spawn((
                Pop,
                MentalState::Broken(MentalBreakType::Vandalize),
                GridPosition { x: 0, y: 0 }, // Outside
            ))
            .id();

        // Run detection system
        let mut schedule = Schedule::default();
        schedule.add_systems(check_crime_system);
        schedule.run(&mut world);

        // Assert Wanted
        assert!(
            world.get::<Wanted>(vandal).is_some(),
            "Pop outside Sanctuary SHOULD be marked Wanted"
        );
    }

    #[test]
    fn test_warden_ignores_sanctuary_fugitive() {
        let mut world = setup_world();

        // Wanted criminal hiding in Sanctuary
        let fugitive = world
            .spawn((
                Pop,
                Wanted { severity: 1.0 },
                GridPosition { x: 5, y: 5 }, // Inside Sanctuary
            ))
            .id();

        let criminals = vec![ScorableCandidate::new(
            fugitive,
            GridPosition { x: 5, y: 5 },
        )];

        // Warden outside
        let warden_pos = GridPosition { x: 4, y: 5 };

        // Evaluate action
        let result = evaluate_warden_action(&warden_pos, &criminals, world.resource::<ZoneGrid>());

        // Assert NO target
        assert!(
            result.is_none(),
            "Warden should ignore fugitive in Sanctuary"
        );
    }

    #[test]
    fn test_warden_pursues_outside_fugitive() {
        let mut world = setup_world();

        // Wanted criminal outside
        let fugitive = world
            .spawn((Pop, Wanted { severity: 1.0 }, GridPosition { x: 0, y: 0 }))
            .id();

        let criminals = vec![ScorableCandidate::new(
            fugitive,
            GridPosition { x: 0, y: 0 },
        )];

        // Warden outside
        let warden_pos = GridPosition { x: 1, y: 0 };

        // Evaluate action
        let result = evaluate_warden_action(&warden_pos, &criminals, world.resource::<ZoneGrid>());

        // Assert TARGET found
        assert!(
            result.is_some(),
            "Warden should pursue fugitive outside Sanctuary"
        );
    }


    #[test]
    fn test_pardon_removes_wanted_but_adds_corruption() {
        use crate::layer1::justice::{process_pardons_system, PardonIssuedEvent};
        use crate::layer1::black_market::ColonyStats;
        use crate::layer1::chronicle::AddChronicleEvent;

        let mut world = setup_world();
        world.insert_resource(ColonyStats { corruption: 0.0, ..Default::default() });
        world.insert_resource(Events::<PardonIssuedEvent>::default());
        world.insert_resource(Events::<AddChronicleEvent>::default());

        let criminal_entity = world.spawn((
            Pop,
            Wanted { severity: 0.5 },
            crate::layer1::justice::Inmate { sentence_ticks: 100 },
        )).id();

        world.send_event(PardonIssuedEvent { target: criminal_entity });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_pardons_system);
        schedule.run(&mut world);

        assert!(world.get::<Wanted>(criminal_entity).is_none(), "Pardon should remove wanted status");
        assert!(world.get::<crate::layer1::justice::Inmate>(criminal_entity).is_none(), "Pardon should release from jail");

        let stats = world.resource::<ColonyStats>();
        assert!(stats.corruption > 0.0, "Pardoning should increase corruption");

        let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
        let mut reader = chronicle_events.get_cursor();
        let events: Vec<_> = reader.read(chronicle_events).collect();
        assert_eq!(events.len(), 1, "Pardoning should generate a chronicle event");
    }
}
