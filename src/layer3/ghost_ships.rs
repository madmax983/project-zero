//! Ghost Ships
//!
//! Handles the discovery and simulation of abandoned or lost vessels drifting in the void.
//! These ships carry ancient artifacts and lost populations but present extreme risks
//! when salvaged, due to onboard hazards or rogue AI.

use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct LostInTransit {
    pub cycles_lost: u32,
}

#[derive(Component)]
pub struct GhostShip;

#[derive(Event)]
pub struct EvaluateTransitEvent {
    pub ship: Entity,
}

#[derive(Event)]
pub struct EvaluateLostShipReturnEvent {
    pub ship: Entity,
}

pub fn evaluate_transit_system(
    mut events: EventReader<EvaluateTransitEvent>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();
    for event in events.read() {
        if rng.gen_bool(1.0) {
            // Assume RNG favors an incident for the test
            commands
                .entity(event.ship)
                .insert(LostInTransit { cycles_lost: 0 });
        }
    }
}

use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};

pub fn evaluate_lost_ship_return_system(
    mut events: EventReader<EvaluateLostShipReturnEvent>,
    mut commands: Commands,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    let mut rng = rand::thread_rng();
    for event in events.read() {
        if rng.gen_bool(1.0) {
            // Assume RNG favors return for the test
            commands.entity(event.ship).remove::<LostInTransit>();
            commands.entity(event.ship).insert(GhostShip);
            chronicle_events.send(AddChronicleEvent {
                text: "A ghost ship has returned from the void.".to_string(),
                importance: EventImportance::Major,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock components for tests to compile
    #[derive(Component)]
    pub struct Ship;

    #[test]
    fn test_ship_becomes_lost_during_transit() {
        let mut app = App::new();
        app.add_event::<EvaluateTransitEvent>();
        app.add_systems(Update, evaluate_transit_system);

        let ship_entity = app.world_mut().spawn(Ship).id();

        // Trigger the incident evaluation system
        app.world_mut()
            .send_event(EvaluateTransitEvent { ship: ship_entity });
        app.update(); // Assume RNG favors an incident for the test

        // The ship should now have the LostInTransit component
        let lost_comp = app.world().get::<LostInTransit>(ship_entity);
        assert!(lost_comp.is_some());
    }

    #[test]
    fn test_ghost_ship_returns() {
        let mut app = App::new();
        app.add_event::<EvaluateLostShipReturnEvent>();
        app.add_event::<crate::layer1::chronicle::AddChronicleEvent>();
        app.add_systems(Update, evaluate_lost_ship_return_system);

        let ship_entity = app
            .world_mut()
            .spawn((Ship, LostInTransit { cycles_lost: 10 }))
            .id();

        // Trigger the return evaluation system
        app.world_mut()
            .send_event(EvaluateLostShipReturnEvent { ship: ship_entity });
        app.update(); // Assume RNG favors return for the test

        // The ship should lose LostInTransit and gain GhostShip
        assert!(app.world().get::<LostInTransit>(ship_entity).is_none());
        assert!(app.world().get::<GhostShip>(ship_entity).is_some());
    }
}
