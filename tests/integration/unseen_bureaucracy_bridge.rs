use bevy::prelude::*;
use scale::layer1::administration::designation::{Designation, DesignationType};
use scale::layer1::architecture::structure::Structure;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::day_night::{DayNightCycle, TimeOfDay};
use scale::layer1::economy::resources::ColonyResources;
use scale::layer1::integration::phantom_shift_chronicle_bridge;
use scale::layer1::pop::Pop;
use scale::layer1::unseen_bureaucracy::{
    phantom_shift_system, Desperation, PhantomShiftEvent, ShadowEconomy,
};

#[test]
fn test_unseen_bureaucracy_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);

    app.add_event::<PhantomShiftEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(
        Update,
        (phantom_shift_system, phantom_shift_chronicle_bridge).chain(),
    );

    app.insert_resource(DayNightCycle {
        time_of_day: TimeOfDay::Night,
        ..Default::default()
    });

    app.insert_resource(ColonyResources {
        metal: 50.0,
        stone: 50.0,
        ..Default::default()
    });

    app.insert_resource(ShadowEconomy::default());

    app.world_mut().spawn((Pop, Desperation { value: 90.0 }));

    app.world_mut().spawn((
        Designation {
            designation_type: DesignationType::Repair,
        },
        Structure {
            current_hp: 50.0,
            max_hp: 100.0,
        },
    ));

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<_> = cursor.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit exactly one chronicle event");
    assert_eq!(
        emitted[0].importance,
        EventImportance::Minor,
        "Importance should be minor"
    );
    assert_eq!(
        emitted[0].text,
        "We noticed missing resources. The desperate toil in the dark to fix our neglected infrastructure."
    );
}
