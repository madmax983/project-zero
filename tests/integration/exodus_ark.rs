use bevy_app::App;
use bevy_ecs::prelude::*;
use scale::layer1::building::{Building, BuildingType};
use scale::layer1::exodus::{build_ark_system, cannibalize_infrastructure_system, ArkShipProject, Cannibalizable};
use scale::layer1::resources::ColonyResources;

#[test]
fn test_exodus_integration() {
    let mut app = App::new();

    app.add_systems(Update, (cannibalize_infrastructure_system, build_ark_system).chain());

    app.insert_resource(ColonyResources {
        scrap: 0.0,
        ..Default::default()
    });

    let ark_entity = app.world_mut().spawn(ArkShipProject {
        progress: 0,
        target: 1000,
    }).id();

    let building_entity = app.world_mut().spawn((
        Building {
            building_type: BuildingType::Housing,
        },
        Cannibalizable { yield_amount: 150 },
    )).id();

    app.update();

    assert!(app.world().get::<Building>(building_entity).is_none(), "Building should be cannibalized");

    let scrap = app.world().resource::<ColonyResources>().scrap;
    assert_eq!(scrap, 50.0, "150 scrap yielded, 100 consumed by Ark");

    let ark = app.world().get::<ArkShipProject>(ark_entity).unwrap();
    assert_eq!(ark.progress, 100, "Ark should progress by 100");
}
