use bevy::prelude::*;
use scale::layer1::genetics::{GeneSplicingEvent, GeneMod, process_gene_splicing_system, GeneSplicingResultEvent};
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::integration::gene_splicing_chronicle_bridge;
use scale::layer1::health::Health;
use scale::layer1::traits::Traits;

#[test]
fn test_gene_splicing_success_emits_chronicle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<GeneSplicingEvent>();
    app.add_event::<GeneSplicingResultEvent>();
    app.add_event::<AddChronicleEvent>();

    let pop_entity = app
        .world_mut()
        .spawn((
            Traits::default(),
            Health {
                current: 100.0,
                max: 100.0,
            },
        ))
        .id();

    app.world_mut().send_event(GeneSplicingEvent {
        target: pop_entity,
        mod_type: GeneMod::StoneSkin,
        success_chance: 1.0,
    });

    app.add_systems(Update, (process_gene_splicing_system, gene_splicing_chronicle_bridge).chain());
    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_reader();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert!(events[0].text.contains("cut was successful") || events[0].text.contains("success"), "Text should mention success");
    assert_eq!(events[0].importance, EventImportance::Major);
}

#[test]
fn test_gene_splicing_failure_emits_chronicle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<GeneSplicingEvent>();
    app.add_event::<GeneSplicingResultEvent>();
    app.add_event::<AddChronicleEvent>();

    let pop_entity = app
        .world_mut()
        .spawn((
            Traits::default(),
            Health {
                current: 100.0,
                max: 100.0,
            },
        ))
        .id();

    app.world_mut().send_event(GeneSplicingEvent {
        target: pop_entity,
        mod_type: GeneMod::StoneSkin,
        success_chance: 0.0,
    });

    app.add_systems(Update, (process_gene_splicing_system, gene_splicing_chronicle_bridge).chain());
    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_reader();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert!(events[0].text.contains("twist") || events[0].text.contains("failed"), "Text should mention failure");
    assert_eq!(events[0].importance, EventImportance::Major);
}
