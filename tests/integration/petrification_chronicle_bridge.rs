use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::petrification::{
    petrification_transformation_system, PetrificationSickness, PopPetrifiedEvent,
};
use scale::layer1::pop::{Pop, PopName};
use scale::layer1::core::integration::petrification_chronicle_bridge;
use bevy::prelude::*;

#[test]
fn test_petrification_chronicle_event() {
    let mut app = App::new();
    app.add_event::<PopPetrifiedEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(
        Update,
        (
            petrification_transformation_system,
            petrification_chronicle_bridge,
        ).chain(),
    );

    let _pop = app
        .world_mut()
        .spawn((
            Pop,
            PopName("Jim".to_string()),
            PetrificationSickness {
                stage: 100,
                max_stage: 100,
            },
        ))
        .id();

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<_> = cursor.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert!(emitted[0].text.contains("fully petrified"));
    assert!(emitted[0].text.contains("Jim"));
}
