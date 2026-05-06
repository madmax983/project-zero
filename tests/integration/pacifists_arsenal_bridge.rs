use bevy::prelude::*;
use scale::layer1::core::map::GridPosition;
use scale::layer3::fleets::pacifists_arsenal::*;
use scale::layer2::fleet::Fleet;


#[test]
fn test_pacifists_arsenal_bridge() {
    let mut app = App::new();

    // Instead of full run_simulation_tick which requires hundreds of resources,
    // we extract just the simulation schedule but we manually add the schedule
    // to run. We use minimal plugins or just a custom schedule to test the specific seam.
    // However, the systems are added directly to Update via build_simulation_schedule?
    // No, build_simulation_schedule returns a Schedule with the SimulationSchedule label.
    // The previous panic was due to missing event resources.

    app.add_systems(Update, (
        apply_empathy_broadcast_system,
        check_will_to_fight_mutiny_system.after(apply_empathy_broadcast_system)
    ));

    // Spawn pacifist broadcast ship
    let pacifist_pos = GridPosition { x: 5, y: 5 };
    app.world_mut().spawn((
        Fleet,
        PacifistBroadcast {
            strength: 20.0,
            radius: 5.0,
        },
        pacifist_pos,
    ));

    // Spawn enemy combat fleet in range
    let enemy = app
        .world_mut()
        .spawn((
            Fleet,
            Combatant,
            WillToFight(10.0),
            GridPosition { x: 6, y: 5 },
        ))
        .id();

    // Run the schedule
    app.update();

    // After 1 tick, the enemy's will to fight should drop by 20.
    // This reduces it to 0, which triggers mutiny
    assert!(
        app.world().get::<Mutinous>(enemy).is_some(),
        "Fleet should mutiny when will to fight reaches 0 due to integration"
    );
    assert!(
        app.world().get::<Combatant>(enemy).is_none(),
        "Mutinous fleet should lose Combatant status"
    );
}
