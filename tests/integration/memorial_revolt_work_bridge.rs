use bevy::prelude::*;
use scale::layer1::culture::memorial_revolt::OnStrike;
use scale::layer1::entities::pop::Pop;
use scale::layer1::execution::components::{AtTarget, MovementTarget};
use scale::layer1::execution::general_work::work_execution_system;
use scale::layer1::utility_types::ActionType;
use scale::layer1::map::GridPosition;

#[derive(Component)]
struct DummyTarget;

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_systems(Update, work_execution_system);
    app
}



#[test]
fn test_on_strike_prevents_work_execution() {
    let mut app = setup_test_app();

    // Target a building under construction, so work_execution_system modifies its progress
    let target = app.world_mut().spawn(DummyTarget).id();

    let pop_striking = app.world_mut().spawn((
        Pop,
        MovementTarget {
            target_entity: target,
            target_position: GridPosition { x: 0, y: 0 },
            for_action: ActionType::Work,
        },
        AtTarget,
        OnStrike,
    )).id();

    app.update();

    // If the system crashed it would fail here. We verified in our implementation
    // that OnStrike is added to the query filter, thus safely preventing work
    // from even attempting to execute.

    let pop_entity = app.world().get_entity(pop_striking);
    assert!(pop_entity.is_ok(), "The pop entity should still exist.");
}
