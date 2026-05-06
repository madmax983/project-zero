//! The "Radio Nostalgia" Module
//!
//! The `radio_nostalgia` module governs the effects of deep space broadcasts
//! on the morale of the colony. It handles the reception of news (both real and fabricated)
//! and the consequences of censorship when propaganda is eventually exposed.
use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::edicts::{ColonyPolicies, Policy};
use crate::layer1::social::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BroadcastType {
    Victory,
    Defeat,
    Propaganda,
}

#[derive(Event)]
pub struct BroadcastReceivedEvent {
    pub colony: Entity, // Usually irrelevant since morale is per-pop, but kept for spec parity
    pub broadcast_type: BroadcastType,
}

/// Handles incoming deep space broadcasts and their effects on morale.
///
/// # Examples
///
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::radio_nostalgia::{BroadcastReceivedEvent, BroadcastType, handle_broadcasts_system};
/// use scale::layer1::social::morale::Morale;
/// use scale::layer1::edicts::ColonyPolicies;
/// use scale::layer1::chronicle::AddChronicleEvent;
///
/// let mut app = bevy_app::App::new();
/// app.init_resource::<Events<BroadcastReceivedEvent>>();
/// app.init_resource::<Events<AddChronicleEvent>>();
/// app.init_resource::<ColonyPolicies>();
///
/// app.add_systems(bevy_app::Update, handle_broadcasts_system);
///
/// let pop_entity = app.world_mut().spawn(Morale::default()).id();
/// let colony_entity = app.world_mut().spawn_empty().id();
///
/// app.world_mut().resource_mut::<Events<BroadcastReceivedEvent>>().send(BroadcastReceivedEvent {
///     colony: colony_entity,
///     broadcast_type: BroadcastType::Victory,
/// });
///
/// app.update();
///
/// let morale = app.world().get::<Morale>(pop_entity).unwrap();
/// assert!(!morale.modifiers.is_empty());
/// ```
pub fn handle_broadcasts_system(
    mut events: EventReader<BroadcastReceivedEvent>,
    mut morale_query: Query<&mut Morale>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
    policies: Option<Res<ColonyPolicies>>,
) {
    let censor_active = policies.is_some_and(|p| p.is_active(Policy::CensorBroadcasts));

    for event in events.read() {
        if censor_active {
            // Under censorship, no major morale effects apply (or slight distrust),
            // but we skip the large swings and the chronicle event.
            continue;
        }

        match event.broadcast_type {
            BroadcastType::Victory => {
                for mut morale in morale_query.iter_mut() {
                    morale.add_modifier(MoodModifier {
                        label: "Victory Broadcast".to_string(),
                        value: 0.15,
                        duration: 100, // Arbitrary duration
                    });
                }
                chronicle_events.send(AddChronicleEvent {
                    text: "Received news of a great Victory from the Homeworld.".to_string(),
                    importance: EventImportance::Major,
                });
            }
            BroadcastType::Defeat => {
                for mut morale in morale_query.iter_mut() {
                    morale.add_modifier(MoodModifier {
                        label: "Defeat Broadcast".to_string(),
                        value: -0.20,
                        duration: 100,
                    });
                }
                chronicle_events.send(AddChronicleEvent {
                    text: "Received news of a terrible Defeat from the Homeworld.".to_string(),
                    importance: EventImportance::Major,
                });
            }
            BroadcastType::Propaganda => {
                // Truth revealed: Original victory was a lie
                for mut morale in morale_query.iter_mut() {
                    morale.add_modifier(MoodModifier {
                        label: "Propaganda Revealed".to_string(),
                        value: -0.30,
                        duration: 200,
                    });
                }
                chronicle_events.send(AddChronicleEvent {
                    text: "Learned that the previous Victory was propaganda. Trust is broken."
                        .to_string(),
                    importance: EventImportance::Legendary,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_colony_morale(world: &mut World, _colony: Entity) -> f32 {
        // Average morale across all pops
        let mut total = 0.0;
        let mut count = 0;
        for morale in world.query::<&Morale>().iter(world) {
            // We'll calculate the sum of modifiers for the test since the base value update
            // requires other complex systems, or just check modifier existence
            let mod_sum: f32 = morale.modifiers.iter().map(|m| m.value).sum();
            total += morale.value + mod_sum;
            count += 1;
        }
        if count == 0 {
            return 0.0;
        }
        total / count as f32
    }

    fn trigger_delayed_broadcast(world: &mut World, colony: Entity, btype: BroadcastType) {
        world
            .resource_mut::<Events<BroadcastReceivedEvent>>()
            .send(BroadcastReceivedEvent {
                colony,
                broadcast_type: btype,
            });
    }

    fn trigger_refugee_arrival_with_truth(world: &mut World, colony: Entity) {
        // Equivalent to receiving the Propaganda truth
        trigger_delayed_broadcast(world, colony, BroadcastType::Propaganda);
    }

    fn get_chronicle_events(world: &World) -> Vec<AddChronicleEvent> {
        let events = world.resource::<Events<AddChronicleEvent>>();
        #[allow(deprecated)]
        let mut reader = events.get_reader();
        reader.read(events).cloned().collect()
    }

    fn setup_test_app() -> bevy_app::App {
        let mut app = bevy_app::App::new();
        app.init_resource::<Events<BroadcastReceivedEvent>>();
        app.init_resource::<Events<AddChronicleEvent>>();
        app.init_resource::<ColonyPolicies>();
        app.add_systems(bevy_app::Update, handle_broadcasts_system);
        app
    }

    #[test]
    fn test_radio_nostalgia_broadcast_received() {
        let mut app = setup_test_app();
        let colony_entity = app.world_mut().spawn_empty().id();
        // CommsConsole removed as part of Razor reduction
        let _pop = app.world_mut().spawn(Morale::default()).id();

        let initial_morale = get_colony_morale(app.world_mut(), colony_entity);

        trigger_delayed_broadcast(app.world_mut(), colony_entity, BroadcastType::Victory);
        app.update();

        let events = get_chronicle_events(app.world());
        assert!(events.iter().any(|e| e.text.contains("Victory")));

        let new_morale = get_colony_morale(app.world_mut(), colony_entity);
        assert!(
            new_morale > initial_morale,
            "Victory broadcast should increase morale"
        );
    }

    #[test]
    fn test_radio_nostalgia_truth_revealed() {
        let mut app = setup_test_app();
        let colony_entity = app.world_mut().spawn_empty().id();
        let _pop = app.world_mut().spawn(Morale::default()).id();

        trigger_delayed_broadcast(app.world_mut(), colony_entity, BroadcastType::Victory);
        app.update();

        let initial_morale = get_colony_morale(app.world_mut(), colony_entity);

        trigger_refugee_arrival_with_truth(app.world_mut(), colony_entity);
        app.update();

        let new_morale = get_colony_morale(app.world_mut(), colony_entity);
        assert!(
            new_morale < initial_morale,
            "Truth revealing propaganda should decrease morale significantly"
        );
    }

    #[test]
    fn test_radio_nostalgia_censorship() {
        let mut app = setup_test_app();
        let colony_entity = app.world_mut().spawn_empty().id();
        let _pop = app.world_mut().spawn(Morale::default()).id();

        app.world_mut()
            .resource_mut::<ColonyPolicies>()
            .toggle(Policy::CensorBroadcasts);

        let initial_morale = get_colony_morale(app.world_mut(), colony_entity);

        trigger_delayed_broadcast(app.world_mut(), colony_entity, BroadcastType::Defeat);
        app.update();

        let new_morale = get_colony_morale(app.world_mut(), colony_entity);
        assert!(
            (new_morale - initial_morale).abs() < f32::EPSILON,
            "Censored broadcasts should not immediately drop morale"
        );
    }
}
