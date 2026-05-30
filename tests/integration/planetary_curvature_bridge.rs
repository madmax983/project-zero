use bevy::prelude::*;
use scale::layer1::architecture::building::{Building, BuildingType};
use scale::layer1::architecture::turret::{Turret, turret_fire_system};
use scale::layer1::combat::{AttackProperties, CombatState};
use scale::layer1::map::GridPosition;
use scale::layer1::fauna::{Fauna, FaunaType};
use scale::layer1::health::Health;
use scale::layer1::physics::curvature::{Elevation, PlanetCurvature};
use scale::layer1::resources::{ColonyResources, ResourceType};

#[test]
fn test_turret_blocked_by_horizon() {
    let mut app = App::new();
    app.insert_resource(ColonyResources { waste: 10.0, ..Default::default() });
    app.insert_resource(scale::shared::time::SimulationTime::default());
    app.insert_resource(scale::layer1::map::ScreenShake::default());
    // Strict horizon
    app.insert_resource(PlanetCurvature { horizon_distance_base: 5.0 });

    let _turret = app.world_mut().spawn((
        Building { building_type: BuildingType::TrashCannon },
        Turret {
            attack: AttackProperties {
                damage: 10.0,
                range: 15.0, // Very long range
                cooldown: 0,
                accuracy: 1.0,
            },
            ammo_cost: 1.0,
            ammo_type: ResourceType::Waste,
        },
        GridPosition { x: 0, y: 0 },
        Elevation(0.0),
        CombatState::default(),
    )).id();

    let target = app.world_mut().spawn((
        Fauna { fauna_type: FaunaType::Wolf, ..Default::default() },
        GridPosition { x: 10, y: 0 }, // Out of horizon (5.0), but in range (15.0)
        Elevation(0.0),
        Health { current: 100.0, max: 100.0, has_rust_lung: false },
    )).id();

    turret_fire_system(app.world_mut());

    let health = app.world().get::<Health>(target).unwrap();
    // Should take NO damage because it's over the horizon
    assert_eq!(health.current, 100.0, "Turret should not fire on target beyond horizon");
}

#[test]
fn test_turret_elevated_sees_over_horizon() {
    let mut app = App::new();
    app.insert_resource(ColonyResources { waste: 10.0, ..Default::default() });
    app.insert_resource(scale::shared::time::SimulationTime::default());
    app.insert_resource(scale::layer1::map::ScreenShake::default());
    // Strict horizon
    app.insert_resource(PlanetCurvature { horizon_distance_base: 5.0 });

    let _turret = app.world_mut().spawn((
        Building { building_type: BuildingType::TrashCannon },
        Turret {
            attack: AttackProperties {
                damage: 10.0,
                range: 15.0, // Very long range
                cooldown: 0,
                accuracy: 1.0,
            },
            ammo_cost: 1.0,
            ammo_type: ResourceType::Waste,
        },
        GridPosition { x: 0, y: 0 },
        Elevation(6.0), // High elevation adds to horizon (base 5.0 + elev 6.0 = 11.0 horizon)
        CombatState::default(),
    )).id();

    let target = app.world_mut().spawn((
        Fauna { fauna_type: FaunaType::Wolf, ..Default::default() },
        GridPosition { x: 10, y: 0 }, // Distance 10.0, which is within the extended horizon of 11.0
        Elevation(0.0),
        Health { current: 100.0, max: 100.0, has_rust_lung: false },
    )).id();

    turret_fire_system(app.world_mut());

    let health = app.world().get::<Health>(target).unwrap();
    // Should take damage
    assert_eq!(health.current, 90.0, "Elevated turret should fire on target within extended horizon");
}
