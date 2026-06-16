use bevy::prelude::*;
use scale::layer1::architecture::building::{Building, BuildingType};
use scale::layer1::core::events::BuildingCompletedEvent;
use scale::layer1::nature::subterranean_smog::HeavyIndustry;

#[test]
fn test_building_completed_adds_heavy_industry() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<BuildingCompletedEvent>();
    app.add_systems(Update, scale::layer1::core::integration::subterranean_smog_bridge_system);

    let building = app.world_mut().spawn(Building {
        building_type: BuildingType::Refinery,
    }).id();

    app.world_mut().send_event(BuildingCompletedEvent { entity: building });
    app.update();

    assert!(app.world().get::<HeavyIndustry>(building).is_some(), "Refinery should be marked as HeavyIndustry");
}
