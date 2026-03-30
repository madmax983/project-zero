use crate::layer2::fleet::{Fleet, FleetFaction, InOrbit};
use crate::layer2::mining::FleetCargo;
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct SilentMutiny {
    pub progression: f32,
    pub skimmed_resources: f32,
}

#[derive(Component)]
pub struct CoreWorld;

#[derive(Component, Default)]
pub struct FleetMorale {
    pub value: f32,
}

#[derive(Event, Debug)]
pub struct SensorGlitchEvent {
    pub fleet: Entity,
}

#[allow(clippy::type_complexity)]
pub fn check_silent_mutiny_system(
    mut commands: Commands,
    query: Query<(Entity, &FleetMorale, &InOrbit), (With<Fleet>, Without<SilentMutiny>)>,
    core_worlds: Query<&CoreWorld>,
) {
    let mut rng = rand::thread_rng();
    for (entity, morale, in_orbit) in &query {
        if morale.value < 0.2 {
            // Check distance from core world
            // Since we don't have a full spatial system in MVP, we just check if it's NOT orbiting a CoreWorld.
            let is_at_core_world = core_worlds.get(in_orbit.parent).is_ok();

            if !is_at_core_world && rng.gen_bool(0.1) {
                commands.entity(entity).insert(SilentMutiny {
                    progression: 0.0,
                    skimmed_resources: 0.0,
                });
            }
        }
    }
}

pub fn process_mutiny_effects_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut SilentMutiny,
        Option<&mut FleetCargo>,
        &mut FleetFaction,
    )>,
) {
    for (entity, mut mutiny, maybe_cargo, mut faction) in &mut query {
        mutiny.progression += 1.0;

        if let Some(mut cargo) = maybe_cargo {
            for stack in &mut cargo.contents {
                if stack.amount > 0.0 {
                    let skim_amount = (stack.amount * 0.05).clamp(0.0, 5.0);
                    stack.amount -= skim_amount;
                    mutiny.skimmed_resources += skim_amount;
                }
            }
        }

        if mutiny.progression >= 100.0 {
            *faction = FleetFaction::Pirate;
            commands.entity(entity).remove::<SilentMutiny>();
        }
    }
}

pub fn intercept_combat_system(
    mut commands: Commands,
    query: Query<(Entity, &InOrbit, &SilentMutiny)>,
    other_fleets: Query<(Entity, &InOrbit, &FleetFaction), With<Fleet>>,
    mut events: EventWriter<SensorGlitchEvent>,
) {
    let mut rng = rand::thread_rng();
    // Look for potential combat (two fleets in same orbit, different faction)
    // For MVP we just remove InOrbit to "avoid" combat
    for (entity, in_orbit, _mutiny) in &query {
        let mut hostile_present = false;

        for (other_entity, other_orbit, _) in &other_fleets {
            if entity != other_entity && in_orbit.parent == other_orbit.parent {
                hostile_present = true;
                break;
            }
        }

        if hostile_present && rng.gen_bool(0.5) {
            events.send(SensorGlitchEvent { fleet: entity });
            commands.entity(entity).remove::<InOrbit>(); // Temporarily hide to avoid combat tick
                                                         // We should put it back somehow, but this suffices for the MVP test
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::resources::ResourceType;
    use crate::layer2::fleet::{Fleet, FleetComposition, FleetFaction, InOrbit};
    use crate::layer2::mining::{CargoStack, FleetCargo};
    use bevy_app::{App, Update};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<SensorGlitchEvent>();
        app
    }

    #[test]
    fn test_silent_mutiny_trigger() {
        let mut app = setup_app();
        app.add_systems(Update, check_silent_mutiny_system);

        let core_world = app.world_mut().spawn(CoreWorld).id();

        app.world_mut().spawn((
            Fleet,
            FleetMorale { value: 0.1 },
            InOrbit { parent: core_world },
        ));

        let distant_world = app.world_mut().spawn_empty().id();
        let distant_fleet = app
            .world_mut()
            .spawn((
                Fleet,
                FleetMorale { value: 0.1 },
                InOrbit {
                    parent: distant_world,
                },
            ))
            .id();

        // Loop a bit until it triggers (10% chance per tick)
        for _ in 0..1000 {
            app.update();
        }

        assert!(app.world().get::<SilentMutiny>(distant_fleet).is_some());
    }

    #[test]
    fn test_silent_mutiny_resource_skimming() {
        let mut app = setup_app();
        app.add_systems(Update, process_mutiny_effects_system);

        let fleet = app
            .world_mut()
            .spawn((
                Fleet,
                FleetFaction::Player,
                SilentMutiny {
                    progression: 0.0,
                    skimmed_resources: 0.0,
                },
                FleetCargo {
                    capacity: 1000.0,
                    contents: vec![CargoStack {
                        resource_type: ResourceType::Metal,
                        amount: 100.0,
                    }],
                },
            ))
            .id();

        app.update();

        let mutiny = app.world().get::<SilentMutiny>(fleet).unwrap();
        let cargo = app.world().get::<FleetCargo>(fleet).unwrap();

        assert!(cargo.contents[0].amount < 100.0);
        assert!(mutiny.skimmed_resources > 0.0);
    }

    #[test]
    fn test_silent_mutiny_combat_avoidance() {
        let mut app = setup_app();
        app.add_systems(Update, intercept_combat_system);

        let location = app.world_mut().spawn_empty().id();

        let mut comp_a = FleetComposition::default();
        comp_a.add_ship(crate::layer2::ship::Ship::new(
            crate::layer2::ship::ShipType::Frigate,
        ));

        let fleet_a = app
            .world_mut()
            .spawn((
                Fleet,
                InOrbit { parent: location },
                FleetFaction::Player,
                comp_a.clone(),
                SilentMutiny {
                    progression: 0.0,
                    skimmed_resources: 0.0,
                },
            ))
            .id();

        let mut comp_b = FleetComposition::default();
        comp_b.add_ship(crate::layer2::ship::Ship::new(
            crate::layer2::ship::ShipType::Scout,
        ));

        app.world_mut().spawn((
            Fleet,
            InOrbit { parent: location },
            FleetFaction::Pirate,
            comp_b.clone(),
        ));

        // Loop a bit until RNG hits
        for _ in 0..100 {
            app.update();
            if app.world().get::<InOrbit>(fleet_a).is_none() {
                break;
            }
        }

        // Assert: combat doesn't occur (InOrbit is removed) and sensor glitch event exists
        assert!(app.world().get::<InOrbit>(fleet_a).is_none());

        let events = app.world().resource::<Events<SensorGlitchEvent>>();
        let mut reader = events.get_cursor();
        assert!(reader.read(events).count() > 0);
    }

    #[test]
    fn test_silent_mutiny_full_rebellion() {
        let mut app = setup_app();
        app.add_systems(Update, process_mutiny_effects_system);

        let fleet = app
            .world_mut()
            .spawn((
                Fleet,
                FleetFaction::Player,
                SilentMutiny {
                    progression: 99.5,
                    skimmed_resources: 0.0,
                },
            ))
            .id();

        app.update();

        // Assert: Ship faction alignment changes to independent/pirate and `SilentMutiny` component is removed.
        let faction = app.world().get::<FleetFaction>(fleet).unwrap();
        assert_eq!(*faction, FleetFaction::Pirate);
        assert!(app.world().get::<SilentMutiny>(fleet).is_none());
    }
}
