use bevy_app::App;
use bevy_ecs::prelude::*;
use scale::layer1::psionics::{FireEvent, WorkFailedEvent, AwakenedPsionic};
use scale::layer1::map::GridPosition;
use scale::layer1::fire::Fire;
use scale::layer1::core::integration::psionics_fire_event_bridge_system;
use scale::layer1::psionics::pyrokinesis_power_activation_system;
use scale::layer1::chronicle::AddChronicleEvent;

#[test]
fn test_integration_psionics_fire_bridge() {
    let mut app = App::new();

    app.add_event::<WorkFailedEvent>();
    app.add_event::<FireEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(bevy_app::Update, (pyrokinesis_power_activation_system, psionics_fire_event_bridge_system).chain());

    let pos = GridPosition { x: 5, y: 5 };
    let pop = app
        .world_mut()
        .spawn((AwakenedPsionic::Pyrokinesis, pos))
        .id();

    app.world_mut().send_event(WorkFailedEvent { entity: pop });
    app.update();

    let fires: Vec<&Fire> = app.world_mut().query::<&Fire>().iter(app.world()).collect();
    assert_eq!(fires.len(), 1, "A Fire entity should be spawned from the Pyrokinesis event chain");
}


use scale::layer1::execution::general_work::work_execution_system;
use scale::layer1::execution::components::{AtTarget, MovementTarget};
use scale::layer1::utility_types::{ActionType, PopAction};
use scale::layer1::designation::{Designation, DesignationType};
use scale::layer1::pop::Pop;

#[test]
fn test_integration_work_failed_event_emission() {
    let mut app = App::new();
    app.add_event::<WorkFailedEvent>();
    app.add_systems(bevy_app::Update, work_execution_system);

    let pos = GridPosition { x: 5, y: 5 };

    // Setup target
    let designation = app.world_mut().spawn((
        Designation {
            designation_type: DesignationType::Repair,
        },
        scale::layer1::structure::Structure { current_hp: 50.0, max_hp: 100.0, ..Default::default() },
        pos,
    )).id();

    // Setup worker
    let pop = app.world_mut().spawn((
        Pop,
        pos,
        AtTarget, // At target!
        MovementTarget { target_entity: designation, target_position: pos, for_action: ActionType::Work },
        PopAction {
            current: ActionType::Work,
            ..Default::default()
        }
    )).id();

    // Run multiple times to trigger the 0.5% probabilistic event
    let mut failed = false;
    for _ in 0..2000 {
        app.update();
        let events = app.world().resource::<Events<WorkFailedEvent>>();
        let mut reader = events.get_cursor();
        if reader.read(events).count() > 0 {
            failed = true;
            break;
        }

        // Re-insert designation to prevent it from despawning and invalidating the test early
        if let Ok(mut e) = app.world_mut().get_entity_mut(designation) {
            e.insert(Designation { designation_type: DesignationType::Repair });
            e.insert(scale::layer1::structure::Structure { current_hp: 50.0, max_hp: 100.0, ..Default::default() });
        }
        if let Ok(mut e) = app.world_mut().get_entity_mut(pop) {
             e.insert((
                 PopAction { current: ActionType::Work, ..Default::default() },
                 MovementTarget { target_entity: designation, target_position: pos, for_action: ActionType::Work },
             ));
        }
    }

    assert!(failed, "A WorkFailedEvent should eventually be emitted when pops work continuously");
}
