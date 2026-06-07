use bevy::prelude::*;
use scale::layer1::access_control::{AccessControl, AccessMode, check_access};
use scale::layer1::infrastructure::subconscious_grid::{Colony, ColonyStress, SmartGrid, GridState, apply_lockdown_system, ResidentOf};
use scale::layer1::pop::Pop;

#[test]
fn test_subconscious_grid_lockdown_affects_access_control() {
    let mut app = App::new();

    app.add_systems(Update, apply_lockdown_system);

    let colony = app.world_mut().spawn((
        Colony,
        ColonyStress { average_level: 96.0 }, // Would cause lockdown, but let's just set the state directly
        SmartGrid { state: GridState::Lockdown },
    )).id();

    let pop = app.world_mut().spawn(Pop).id();

    let door = app.world_mut().spawn((
        AccessControl {
            mode: AccessMode::Public,
            ..Default::default()
        },
        ResidentOf(colony),
    )).id();

    // initially open
    assert!(check_access(app.world(), door, pop));

    app.update();

    // should be in lockdown
    let door_ac = app.world().get::<AccessControl>(door).unwrap();
    assert_eq!(door_ac.mode, AccessMode::Lockdown);

    assert!(!check_access(app.world(), door, pop));
}
