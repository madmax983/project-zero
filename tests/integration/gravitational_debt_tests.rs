use bevy::prelude::*;
use scale::layer1::building::{Building, BuildingType};
use scale::layer1::energy::PowerConsumer;
use scale::layer1::gravitational_debt::{
    gravitational_debt_accumulation_system, gravitational_debt_release_system, AntiGravGenerator,
    DebtReleaseEvent, GravitationalDebt,
};
use scale::layer1::map::GridPosition;

#[test]
fn test_gravitational_debt_accumulates_when_powered() {
    let mut app = App::new();
    app.add_systems(Update, gravitational_debt_accumulation_system);

    // Arrange
    let pos = GridPosition { x: 10, y: 10 };
    let generator_entity = app
        .world_mut()
        .spawn((
            Building {
                building_type: BuildingType::AntiGravGenerator,
            },
            AntiGravGenerator {
                debt_generation_rate: 5.0,
                max_safe_debt: 100.0,
            },
            PowerConsumer {
                active: true,
                demand: 10.0,
            },
            GravitationalDebt {
                accumulated_debt: 0.0,
            },
            pos,
        ))
        .id();

    // Act
    app.update();

    // Assert
    let debt = app
        .world()
        .get::<GravitationalDebt>(generator_entity)
        .unwrap();
    assert!(
        debt.accumulated_debt > 0.0,
        "Debt should accumulate when generator is powered"
    );
}

#[test]
fn test_gravitational_debt_releases_when_unpowered() {
    let mut app = App::new();
    app.add_event::<DebtReleaseEvent>();
    app.add_systems(Update, gravitational_debt_release_system);

    // Arrange
    let pos = GridPosition { x: 10, y: 10 };
    let generator_entity = app
        .world_mut()
        .spawn((
            Building {
                building_type: BuildingType::AntiGravGenerator,
            },
            AntiGravGenerator {
                debt_generation_rate: 5.0,
                max_safe_debt: 100.0,
            },
            PowerConsumer {
                active: false,
                demand: 10.0,
            },
            GravitationalDebt {
                accumulated_debt: 50.0,
            }, // Accumulated some debt
            pos,
        ))
        .id();

    // Act
    app.update();

    // Assert
    let events = app.world().resource::<Events<DebtReleaseEvent>>();
    let mut reader = events.get_cursor();
    let release_events: Vec<_> = reader.read(events).collect();

    assert_eq!(
        release_events.len(),
        1,
        "DebtReleaseEvent should be fired when generator loses power"
    );
    assert_eq!(release_events[0].source_entity, generator_entity);
    assert_eq!(release_events[0].debt_amount, 50.0);

    // Check debt is reset
    let debt = app
        .world()
        .get::<GravitationalDebt>(generator_entity)
        .unwrap();
    assert_eq!(
        debt.accumulated_debt, 0.0,
        "Debt should reset after release"
    );
}

#[test]
fn test_gravitational_debt_releases_when_exceeding_max() {
    let mut app = App::new();
    app.add_event::<DebtReleaseEvent>();
    app.add_systems(Update, gravitational_debt_release_system);

    // Arrange
    let pos = GridPosition { x: 10, y: 10 };
    let generator_entity = app
        .world_mut()
        .spawn((
            Building {
                building_type: BuildingType::AntiGravGenerator,
            },
            AntiGravGenerator {
                debt_generation_rate: 5.0,
                max_safe_debt: 100.0,
            },
            PowerConsumer {
                active: true,
                demand: 10.0,
            }, // Powered, but over max debt
            GravitationalDebt {
                accumulated_debt: 105.0,
            },
            pos,
        ))
        .id();

    // Act
    app.update();

    // Assert
    let events = app.world().resource::<Events<DebtReleaseEvent>>();
    let mut reader = events.get_cursor();
    let release_events: Vec<_> = reader.read(events).collect();

    assert_eq!(
        release_events.len(),
        1,
        "DebtReleaseEvent should be fired when debt exceeds max_safe_debt"
    );
    assert_eq!(release_events[0].debt_amount, 105.0);
    assert_eq!(release_events[0].source_entity, generator_entity);
}
