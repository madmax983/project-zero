use bevy::prelude::*;
use scale::layer1::law::justice::{
    process_crimes_system, process_pardons_system, sheriff_arrest_system, CrimeCommittedEvent,
    CrimeRecord, CrimeType, PardonIssuedEvent,
};
use scale::layer1::map::GridPosition;
use scale::layer1::mind::utility_types::AssignmentType;
use scale::layer1::pop::{Job, Pop};
use scale::layer1::zone::{ZoneGrid, ZoneType};
use scale::layer1::black_market::ColonyStats;

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Register events manually for test (normally done via event buffer cleanup)
    app.add_event::<CrimeCommittedEvent>();
    app.add_event::<PardonIssuedEvent>();

    // Resources
    let mut zone_grid = ZoneGrid::new(20, 20);
    zone_grid.set(5, 5, ZoneType::Jail);
    app.world_mut().insert_resource(zone_grid);

    app.world_mut().insert_resource(ColonyStats {
        corruption: 0.0,
        ..Default::default()
    });

    // Schedule the systems as they would be in the game loop
    app.add_systems(
        Update,
        (
            process_crimes_system,
            sheriff_arrest_system.after(process_crimes_system),
            process_pardons_system.after(sheriff_arrest_system),
        ),
    );

    app
}

#[test]
fn test_justice_system_end_to_end() {
    let mut app = setup_app();

    let criminal = app
        .world_mut()
        .spawn((
            Pop,
            GridPosition { x: 10, y: 10 },
            CrimeRecord::default(),
        ))
        .id();

    let _sheriff = app
        .world_mut()
        .spawn((
            Pop,
            Job {
                workplace: Entity::PLACEHOLDER,
                job_type: AssignmentType::Sheriff,
            },
            GridPosition { x: 9, y: 10 }, // Adjacent
        ))
        .id();

    // 1. Commit Crime
    app.world_mut().send_event(CrimeCommittedEvent {
        perpetrator: criminal,
        crime_type: CrimeType::Theft,
    });
    app.update();

    // After one tick, CrimeRecord should be wanted and sheriff should have arrested them
    let record = app.world().get::<CrimeRecord>(criminal).unwrap();
    assert!(!record.is_wanted(), "Wanted status is cleared upon arrest");
    assert!(record.is_arrested, "Criminal should be arrested");

    let pos = app.world().get::<GridPosition>(criminal).unwrap();
    assert_eq!(pos.x, 5, "Criminal should be in jail");
    assert_eq!(pos.y, 5, "Criminal should be in jail");

    // 2. Issue Pardon
    app.world_mut().send_event(PardonIssuedEvent { target: criminal });
    app.update();

    let record = app.world().get::<CrimeRecord>(criminal).unwrap();
    assert!(!record.is_arrested, "Pardon releases from jail");
    assert_eq!(record.severity, 0, "Severity cleared");

    let stats = app.world().resource::<ColonyStats>();
    assert!(stats.corruption > 0.0, "Corruption should increase");
}
