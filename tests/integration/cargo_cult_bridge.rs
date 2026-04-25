use bevy::prelude::*;
use scale::layer1::items::ItemType;
use scale::layer1::logistics::orbital_drop::OrbitalDropEvent;
use scale::layer1::map::GridPosition;
use scale::layer1::mind::utility_types::{ActionType, PopAction};
use scale::layer1::social::cargo_cult::{CargoCultBelief, EfficiencyDebuff};
use scale::layer1::social::morale::Morale;

#[test]
fn test_cargo_cult_integration_end_to_end() {
    let mut app = App::new();

    // Setup the basic plugins and event streams
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Events<OrbitalDropEvent>>();

    // Instead of registering the entire observation schedule which requires many more events to be set up,
    // we verify the systems execute correctly when appended, simulating how they'll work in observation module.
    // The test is verifying the seam between the events and the systems themselves.
    app.add_systems(
        Update,
        (
            scale::layer1::social::cargo_cult::apply_cargo_cult_belief_system,
            scale::layer1::social::cargo_cult::process_ritual_actions_system,
        )
            .chain(),
    );

    // Spawn a Pop with the necessary components doing an action
    let pop_entity = app
        .world_mut()
        .spawn((
            PopAction {
                current: ActionType::Socialize, // Just pick a non-Idle action
                current_utility: 1.0,
                ticks_committed: 10,
            },
            GridPosition { x: 5, y: 5 },
            Morale {
                value: 0.5,
                modifiers: vec![],
            },
        ))
        .id();

    // Trigger the orbital drop at the exact location
    app.world_mut()
        .resource_mut::<Events<OrbitalDropEvent>>()
        .send(OrbitalDropEvent {
            target: GridPosition { x: 5, y: 5 },
            items: vec![ItemType::Scrap],
            scatter_radius: 0,
        });

    // We must ensure random generation eventually creates the belief (10% chance)
    // Run the system loop many times to trigger it
    let mut found_belief = false;
    for _ in 0..1000 {
        app.update();
        if app.world().entity(pop_entity).contains::<CargoCultBelief>() {
            found_belief = true;
            break;
        }
        // Resend event if not yet believed to give it another 10% chance
        app.world_mut()
            .resource_mut::<Events<OrbitalDropEvent>>()
            .send(OrbitalDropEvent {
                target: GridPosition { x: 5, y: 5 },
                items: vec![ItemType::Scrap],
                scatter_radius: 0,
            });
    }

    assert!(
        found_belief,
        "Cargo cult belief was not assigned by the system schedule integration"
    );

    // Once belief is assigned, process_ritual_actions_system should kick in next update and apply buffs/debuffs
    app.update();

    let debuff = app.world().entity(pop_entity).get::<EfficiencyDebuff>();
    assert!(
        debuff.is_some(),
        "Efficiency debuff was not applied during ritual actions"
    );

    let morale = app.world().entity(pop_entity).get::<Morale>().unwrap();
    assert!(
        morale.modifiers.iter().any(|m| m.label == "Ritual Comfort"),
        "Ritual Comfort morale modifier was not applied"
    );
}
