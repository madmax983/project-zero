#[cfg(test)]
mod tests {

    use crate::layer1::justice::{
        process_crimes_system, process_pardons_system, sheriff_arrest_system, CrimeCommittedEvent,
        CrimeRecord, CrimeType, PardonIssuedEvent,
    };

    use crate::layer1::black_market::ColonyStats;
    use crate::layer1::map::GridPosition;
    use crate::layer1::mind::utility_types::AssignmentType;
    use crate::layer1::pop::Job;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_crime_generates_wanted_token() {
        let mut app = bevy::prelude::App::new();
        app.add_plugins(bevy::prelude::MinimalPlugins);

        app.add_event::<CrimeCommittedEvent>();

        let criminal_entity = app.world_mut().spawn((Pop, CrimeRecord::default())).id();

        // Simulating a crime event
        app.world_mut().send_event(CrimeCommittedEvent {
            perpetrator: criminal_entity,
            crime_type: CrimeType::Theft,
        });

        app.add_systems(bevy::prelude::Update, process_crimes_system);
        app.update();

        let record = app.world().get::<CrimeRecord>(criminal_entity).unwrap();
        assert!(
            record.is_wanted(),
            "Pop should have a wanted token after a crime"
        );
        assert_eq!(record.severity, 50, "Theft should generate severity 50");
    }

    #[test]
    fn test_sheriff_arrests_wanted_pop() {
        let mut app = bevy::prelude::App::new();
        app.add_plugins(bevy::prelude::MinimalPlugins);

        let mut zone_grid = ZoneGrid::new(20, 20);
        zone_grid.set(5, 5, ZoneType::Jail);
        app.world_mut().insert_resource(zone_grid);

        let criminal_entity = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 10, y: 10 },
                CrimeRecord {
                    wanted: true,
                    severity: 50,
                    is_arrested: false,
                },
            ))
            .id();

        let _sheriff_entity = app
            .world_mut()
            .spawn((
                Pop,
                Job {
                    workplace: bevy::prelude::Entity::PLACEHOLDER,
                    job_type: AssignmentType::Sheriff,
                },
                GridPosition { x: 9, y: 10 },
            ))
            .id();

        // Simulating sheriff arresting pop
        app.add_systems(bevy::prelude::Update, sheriff_arrest_system);
        app.update();

        // Assert: Criminal should be moved to jail zone and marked as arrested
        let record = app.world().get::<CrimeRecord>(criminal_entity).unwrap();
        assert!(record.is_arrested, "Criminal should be marked arrested");
        assert_eq!(
            app.world().get::<GridPosition>(criminal_entity).unwrap().x,
            5,
            "Criminal moved to jail X"
        );
        assert_eq!(
            app.world().get::<GridPosition>(criminal_entity).unwrap().y,
            5,
            "Criminal moved to jail Y"
        );
    }

    #[test]
    fn test_pardon_removes_wanted_but_adds_corruption() {
        let mut app = bevy::prelude::App::new();
        app.add_plugins(bevy::prelude::MinimalPlugins);

        app.world_mut().insert_resource(ColonyStats {
            corruption: 0.0,
            ..Default::default()
        });
        app.add_event::<PardonIssuedEvent>();

        let criminal_entity = app
            .world_mut()
            .spawn((
                Pop,
                CrimeRecord {
                    wanted: true,
                    severity: 50,
                    is_arrested: true,
                },
            ))
            .id();

        // Player issues a pardon
        app.world_mut().send_event(PardonIssuedEvent {
            target: criminal_entity,
        });

        app.add_systems(bevy::prelude::Update, process_pardons_system);
        app.update();

        let record = app.world().get::<CrimeRecord>(criminal_entity).unwrap();
        assert!(!record.is_wanted(), "Pardon should remove wanted status");
        assert!(!record.is_arrested, "Pardon should release from jail");

        let stats = app.world().resource::<ColonyStats>();
        assert!(
            stats.corruption > 0.0,
            "Pardoning should increase corruption"
        );
    }

    use crate::layer1::justice::{check_crime_system, evaluate_warden_action, Wanted};

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
}
