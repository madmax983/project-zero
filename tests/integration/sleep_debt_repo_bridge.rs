use bevy::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::mind::sleep_debt::{
    check_critical_sleep_debt_system, process_repo_men_action_system, ForcedComa,
    RepoManArrivalEvent, SleepDebt, SleepDebtConfig,
};
use scale::layer1::pop::Pop;

// Define the bridge we are about to implement so tests can use it
// In rust integration tests we can import it if we make it pub, but it's not implemented yet.
// For the RED phase, we will import it assuming it exists. If it doesn't exist, we must add an empty stub or simply let compilation fail in `cargo test`.
// Since we want `cargo test` to fail to compile or fail at runtime, importing the future function is fine.
use scale::layer1::core::integration::repo_man_arrival_bridge;

#[test]
fn test_sleep_debt_repo_bridge_spawns_repo_man_and_chronicle_event() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SleepDebtConfig>();
    app.add_event::<RepoManArrivalEvent>();
    app.add_event::<AddChronicleEvent>();

    // Chain the systems to ensure correct execution order
    app.add_systems(
        Update,
        (
            check_critical_sleep_debt_system,
            repo_man_arrival_bridge,
            process_repo_men_action_system,
        )
            .chain(),
    );

    // Setup: A Pop with critical sleep debt
    let pop_entity = app
        .world_mut()
        .spawn((
            Pop,
            SleepDebt {
                hours: 600.0,
                active_contract: true,
            },
        ))
        .id();

    // Act: run update cycle
    app.update();

    // Assert: Pop has ForcedComa component
    assert!(
        app.world().get::<ForcedComa>(pop_entity).is_some(),
        "Pop should receive ForcedComa from the repo man"
    );

    // Assert: A RepoMan was spawned temporarily (it despawns after applying coma, but the bridge spawned it)
    // Wait, process_repo_men_action despawns the RepoMan, so we only check the coma and the chronicle event.

    // Assert: Chronicle Event emitted
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let read_events = reader.read(chronicle_events).collect::<Vec<_>>();
    assert_eq!(
        read_events.len(),
        1,
        "Should emit exactly one chronicle event"
    );
    assert_eq!(
        read_events[0].importance,
        scale::layer1::core::chronicle::EventImportance::Major
    );
    assert_eq!(
        read_events[0].text,
        "Corporate Repo Men have arrived to collect unpaid sleep debt!"
    );
}
