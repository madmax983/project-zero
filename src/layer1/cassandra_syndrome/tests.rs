use crate::layer1::cassandra_syndrome::*;
use crate::layer1::entities::pop::Pop;
use crate::layer1::environment::disasters::DisasterType;
use crate::layer1::social::morale::Morale;
use bevy_app::Update;
use bevy_app::App;

#[test]
fn test_prophetic_pop_generates_warning() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, generate_doomsday_warning);
    app.add_event::<DoomsdayWarningEvent>();

    // Spawn a pop with the Prophetic trait
    let pop_id = app.world_mut().spawn((
        Pop,
        Prophetic { cooldown: 0.0 },
    )).id();

    // Act
    app.update();

    // Assert
    let warning_events = app.world().resource::<Events<DoomsdayWarningEvent>>();
    let mut reader = warning_events.get_cursor();
    let events: Vec<_> = reader.read(warning_events).collect();
    assert_eq!(events.len(), 1, "Prophet should generate a warning");
    assert_eq!(events[0].prophet_entity, pop_id);
}

#[test]
fn test_ignored_warning_drops_prophet_morale() {
    // Arrange
    let mut app = App::new();
    app.add_event::<DoomsdayWarningEvent>();
    app.add_systems(Update, handle_ignored_warning);

    let prophet = app.world_mut().spawn((
        Pop,
        Morale { value: 1.0, ..Default::default() },
        Prophetic { cooldown: 10.0 },
    )).id();

    let warning = DoomsdayWarningEvent {
        prophet_entity: prophet,
        disaster_type: DisasterType::MassiveEarthquake,
    };

    // Act
    app.world_mut().resource_mut::<Events<DoomsdayWarningEvent>>().send(warning);
    app.update();

    // Assert
    let morale = app.world().get::<Morale>(prophet).unwrap();
    assert!(morale.value < 1.0, "Ignored warning should drop prophet's morale");
}

#[test]
fn test_disaster_occurrence_spawns_cult() {
    // Arrange
    let mut app = App::new();
    app.add_event::<DisasterOccurredEvent>();
    app.add_systems(Update, validate_prophecy);

    let _prophet = app.world_mut().spawn((
        Pop,
        Prophetic { cooldown: 10.0 },
        ActiveProphecy { disaster_type: DisasterType::MassiveEarthquake },
    )).id();

    let disaster = DisasterOccurredEvent {
        disaster_type: DisasterType::MassiveEarthquake,
    };

    // Act
    app.world_mut().resource_mut::<Events<DisasterOccurredEvent>>().send(disaster);
    app.update();

    // Assert
    let query = app.world_mut().query::<&CultLeader>().get_single(app.world());
    assert!(query.is_ok(), "True prophecy should make the prophet a cult leader");
}
