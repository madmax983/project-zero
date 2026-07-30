use scale::layer1::core::chronicle::AddChronicleEvent;
use bevy::prelude::*;
use scale::layer1::economy::resources::ColonyResources;
use scale::layer1::tech::symbiotic_data_weavers::{DataForest, FloraState, WaterSupply, process_data_forest_system};

#[test]
fn test_data_forest_processing() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, process_data_forest_system);

    app.world_mut().spawn((
        DataForest {
            processing_power: 10.0,
        },
        FloraState::Healthy,
        WaterSupply {
            current: 100.0,
            required: 10.0,
        },
    ));

    app.world_mut().insert_resource(ColonyResources {
        knowledge: 0.0,
        max_knowledge: 1000.0,
        ..Default::default()
    });

    app.update();

    let resources = app.world().get_resource::<ColonyResources>().unwrap();
    assert_eq!(resources.knowledge, 10.0);

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    assert_eq!(events.len(), 0);
}

#[test]
fn test_data_forest_wilting() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, process_data_forest_system);

    app.world_mut().spawn((
        DataForest {
            processing_power: 10.0,
        },
        FloraState::Wilting,
        WaterSupply {
            current: 0.0,
            required: 10.0,
        },
    ));

    app.world_mut().insert_resource(ColonyResources {
        knowledge: 50.0,
        max_knowledge: 1000.0,
        ..Default::default()
    });

    app.update();

    let resources = app.world().get_resource::<ColonyResources>().unwrap();
    // Progress erased due to wilting
    assert_eq!(resources.knowledge, 0.0);

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    assert_eq!(events.len(), 1);
}
