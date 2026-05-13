use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::architecture::building::{Building, BuildingType};
use scale::layer1::core::events::BuildingCompletedEvent;
use scale::layer1::core::map::GridPosition;
use scale::layer1::needs::Needs;
use scale::layer1::pop::Pop;
use scale::layer2::governance::Governor;
use scale::layer3::bureaucracy_of_vanity::{
    vanity_building_listener_system, vanity_sabotage_system, ActiveDemands, GlobalEfficiency,
    ImperialStanding, VainGovernor, VanityProject,
};

#[test]
fn test_vanity_integration() {
    let mut app = App::new();
    app.add_event::<BuildingCompletedEvent>();

    // Systems
    app.add_systems(
        Update,
        (vanity_building_listener_system, vanity_sabotage_system),
    );

    // Initial resources
    app.world_mut()
        .insert_resource(ImperialStanding { value: 50 });
    app.world_mut()
        .insert_resource(GlobalEfficiency { value: 1.0 });

    let gov_pop = app
        .world_mut()
        .spawn((Pop, GridPosition { x: 0, y: 0 }, Needs::default()))
        .id();

    let governor = app
        .world_mut()
        .spawn((
            Governor {
                pop_entity: gov_pop,
                assigned_at: 0,
            },
            VainGovernor { is_vain: true },
        ))
        .id();

    app.world_mut().insert_resource(ActiveDemands {
        vanity_demand_active: true,
        time_since_demand: 100.0, // Immediately trigger sabotage threshold
        active_governor_entity: Some(governor),
    });

    // Run once -> Sabotage applies
    app.update();

    let eff = app.world().resource::<GlobalEfficiency>().value;
    assert!(
        eff < 1.0,
        "Efficiency should have dropped due to ignored demand."
    );

    // Now satisfy the demand
    let b = app
        .world_mut()
        .spawn((
            Building {
                building_type: BuildingType::Statue,
            },
            VanityProject,
        ))
        .id();

    app.world_mut()
        .resource_mut::<Events<BuildingCompletedEvent>>()
        .send(BuildingCompletedEvent { entity: b });

    app.update();

    // ImperialStanding increases
    let standing = app.world().resource::<ImperialStanding>().value;
    assert_eq!(
        standing, 60,
        "Imperial Standing should increase upon building completion."
    );

    // Demand is no longer active
    assert!(
        !app.world().resource::<ActiveDemands>().vanity_demand_active,
        "Demand should be satisfied."
    );
}
