use bevy_ecs::prelude::*;
use scale::layer1::building::{Building, BuildingType};
use scale::layer1::chronicle::Chronicle;
use scale::layer1::energy::{
    power_grid_system, update_auroral_output_system, Battery, Conduit, PowerConsumer, PowerSource,
};
use scale::layer1::map::GridPosition;
use scale::layer1::seasons::{Season, SeasonState};
use scale::layer1::weather::{update_weather_system, WeatherState, WeatherType};
use scale::shared::time::SimulationTime;

// Helper to setup world
fn setup_weather_test_world() -> World {
    let mut world = World::new();
    world.insert_resource(WeatherState::default());
    world.insert_resource(SeasonState::default());
    world.insert_resource(Chronicle::default());
    world.insert_resource(SimulationTime::default());
    // Add events
    world.init_resource::<Events<scale::layer1::chronicle::AddChronicleEvent>>();
    world.init_resource::<Events<scale::layer1::energy::GridOverloadEvent>>();
    // Add resources needed for energy system
    world.init_resource::<scale::layer1::resources::ColonyResources>();
    world.init_resource::<scale::layer1::energy::BlackoutProtocol>();

    world
}

#[test]
fn auroral_collector_activates_in_storm() {
    let mut world = setup_weather_test_world();

    // 1. Force Magnetic Storm
    world.resource_mut::<WeatherState>().current_weather = WeatherType::MagneticStorm;

    // 2. Spawn Auroral Collector
    let collector = world
        .spawn((
            Building {
                building_type: BuildingType::AuroralCollector,
            },
            PowerSource {
                output: 0.0,
                active: true,
            },
            scale::layer1::lighting::LightSource::default(),
            GridPosition { x: 0, y: 0 },
        ))
        .id();

    // 3. Run auroral update system
    let mut schedule = Schedule::default();
    schedule.add_systems(update_auroral_output_system);
    schedule.run(&mut world);

    // 4. Verify Output is 50.0
    let output = world.get::<PowerSource>(collector).unwrap().output;
    assert_eq!(
        output, 50.0,
        "Auroral Collector should produce 50.0 power during Magnetic Storm"
    );
}

#[test]
fn storm_increases_grid_load() {
    let mut world = setup_weather_test_world();

    // 1. Force Magnetic Storm
    world.resource_mut::<WeatherState>().current_weather = WeatherType::MagneticStorm;

    // 2. Spawn Grid: Generator (10), Consumer (8), Battery (100)
    // Normal: 8 demand < 10 production. Surplus +2. Battery Charges.
    // Storm: 8 * 1.5 = 12 demand > 10 production. Deficit -2. Battery Discharges.

    // Generator
    world.spawn((
        Building {
            building_type: BuildingType::Generator,
        },
        PowerSource {
            output: 10.0,
            active: true,
        },
        GridPosition { x: 0, y: 0 },
        // Health needed for overload damage check internal logic
        scale::layer1::health::Health {
            current: 100.0,
            max: 100.0,
        },
    ));

    // Consumer (8 Demand)
    world.spawn((
        Building {
            building_type: BuildingType::Smelter,
        },
        PowerConsumer {
            demand: 8.0,
            active: true,
        },
        GridPosition { x: 0, y: 1 },
        Conduit,
    ));

    // Battery (Full)
    let battery = world
        .spawn((
            Building {
                building_type: BuildingType::Battery,
            },
            Battery {
                capacity: 100.0,
                charge: 100.0,
                max_throughput: 10.0,
            },
            GridPosition { x: 0, y: 2 },
            Conduit,
        ))
        .id();

    // 3. Run grid system
    power_grid_system(&mut world);

    // 4. Verify Battery Discharged
    let bat_state = world.get::<Battery>(battery).unwrap();
    // With storm implemented: 100.0 - 2.0 = 98.0
    // Without storm implemented: 100.0 (stays full)

    // For Red Phase, we ASSERT FAILURE if implementation is missing.
    // Wait, tests should assert EXPECTED behavior. If implementation is missing, test FAILS.

    assert!(
        bat_state.charge < 100.0,
        "Battery should discharge due to storm usage (1.5x demand)"
    );
    assert!(
        (bat_state.charge - 98.0).abs() < 0.001,
        "Expected 98.0, got {}",
        bat_state.charge
    );
}

#[test]
fn weather_can_produce_storm() {
    let mut world = setup_weather_test_world();

    // We try to trigger a storm via natural generation.
    // Winter has highest chance (in my proposed fix).
    world.resource_mut::<SeasonState>().current_season = Season::Winter;

    let mut found_storm = false;
    for _ in 0..1000 {
        // Force update
        world.resource_mut::<WeatherState>().duration_remaining = 0;
        update_weather_system(&mut world);

        if world.resource::<WeatherState>().current_weather == WeatherType::MagneticStorm {
            found_storm = true;
            break;
        }
    }

    assert!(found_storm, "Magnetic Storm should be possible in Winter");
}
