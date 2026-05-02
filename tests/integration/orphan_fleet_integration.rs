use bevy::prelude::*;
use scale::layer2::orphan_fleet::{hack_orphan_fleet_system, orphan_fleet_defection_check_system, process_orphan_defection_system};

#[test]
fn test_orphan_fleet_integration_schedule() {
    let mut app = App::new();
    app.add_systems(Update, (
        hack_orphan_fleet_system,
        orphan_fleet_defection_check_system,
        process_orphan_defection_system
            .after(orphan_fleet_defection_check_system)
    ));
}
