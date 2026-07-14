use bevy::prelude::*;
use scale::layer1::architecture::BuildingType;
use scale::layer1::architecture::Building;
use scale::layer1::map::GridPosition;
use scale::layer1::pop::Pop;
use scale::layer1::traits::{Trait, Traits};
use scale::layer1::psychology::stress::StressTracker;
use scale::layer1::social::propaganda_graffitists::{RebelliousGraffiti, propaganda_graffiti_system, graffiti_aura_system};
use scale::layer1::economy::WorkEfficiency;
use scale::layer1::morale::Morale;
use scale::layer1::core::integration::graffiti_chronicle_bridge;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};

#[test]
fn test_propaganda_graffitists_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, (
        propaganda_graffiti_system,
        graffiti_aura_system,
        graffiti_chronicle_bridge
    ).chain());

    let building = app.world_mut().spawn((
        Building { building_type: BuildingType::Farm },
        GridPosition { x: 5, y: 5 },
    )).id();

    let mut traits = Traits::default();
    traits.add(Trait::Creative);

    app.world_mut().spawn((
        Pop,
        traits,
        StressTracker { accumulated_stress: 90.0 },
        GridPosition { x: 5, y: 5 },
    ));

    app.world_mut().spawn((
        Pop,
        GridPosition { x: 5, y: 5 },
        WorkEfficiency { multiplier: 1.0 },
        Morale::default(),
    ));

    app.update();

    assert!(app.world().get::<RebelliousGraffiti>(building).is_some());

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert!(events[0].text.contains("Rebellious graffiti has appeared"));
    assert_eq!(events[0].importance, EventImportance::Minor);
}
