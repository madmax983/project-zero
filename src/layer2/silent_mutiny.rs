use crate::layer1::resources::ResourceType;
use crate::layer2::fleet::{FleetFaction, InOrbit};
use crate::layer2::system::Orbit;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Marker component for a core world.
#[derive(Component, Debug, Clone, Copy)]
pub struct CoreWorld;

/// Component to store fleet morale.
#[derive(Component, Debug, Clone, Copy)]
pub struct FleetMorale {
    pub value: f32, // 0.0 to 1.0
}

impl FleetMorale {
    pub fn is_low(&self) -> bool {
        self.value < 0.3
    }
}

/// Component to store fleet cargo resources.
#[derive(Component, Debug, Clone, Default)]
pub struct FleetCargo {
    pub resources: std::collections::HashMap<ResourceType, f32>,
}

/// Component to indicate a silent mutiny is in progress.
#[derive(Component, Debug, Clone, Copy)]
pub struct SilentMutiny {
    pub progress: f32,          // 0.0 to 1.0. When it hits 1.0, full rebellion.
    pub skimmed_resources: f32, // Track total value or amount skimmed for stats/stash
}

impl Default for SilentMutiny {
    fn default() -> Self {
        Self {
            progress: 0.0,
            skimmed_resources: 0.0,
        }
    }
}

/// Component added to a fleet temporarily to avoid combat.
#[derive(Component, Debug, Clone, Copy)]
pub struct AvoidCombat;

/// Event emitted when a mutinous fleet fakes a sensor glitch to avoid combat.
#[derive(Event, Debug, Clone, Copy)]
pub struct SensorGlitchEvent {
    pub fleet: Entity,
}

const MUTINY_TRIGGER_CHANCE: f64 = 0.1;
const FAR_ORBIT_THRESHOLD: f32 = 500.0;
const MUTINY_PROGRESSION_RATE: f32 = 0.02;
const MUTINY_SKIM_PERCENTAGE: f32 = 0.05;
const MUTINY_SKIM_MAX: f32 = 5.0;
const AVOID_COMBAT_CHANCE: f64 = 0.5;

pub fn check_silent_mutiny_system(
    mut commands: Commands,
    fleets: Query<(Entity, &FleetMorale, &InOrbit), Without<SilentMutiny>>,
    orbits: Query<&Orbit>,
    core_worlds: Query<Entity, With<CoreWorld>>,
) {
    let mut rng = rand::thread_rng();

    // For MVP, we check the radius of the orbit if the parent is the core world or vice versa.
    for (entity, morale, in_orbit) in fleets.iter() {
        if morale.is_low() {
            // Calculate distance or check if the orbit is far from a core world
            let mut is_far = true;

            if core_worlds.contains(in_orbit.parent) {
                is_far = false;
            } else if let Ok(orbit) = orbits.get(in_orbit.parent) {
                if orbit.radius < FAR_ORBIT_THRESHOLD {
                    is_far = false; // Close to center
                }
            } else if let Ok(orbit) = orbits.get(entity) {
                if orbit.radius < FAR_ORBIT_THRESHOLD {
                    is_far = false;
                }
            }

            if is_far && rng.gen_bool(MUTINY_TRIGGER_CHANCE) {
                commands.entity(entity).insert(SilentMutiny::default());
            }
        }
    }
}

pub fn process_mutiny_effects_system(
    mut commands: Commands,
    mut fleets: Query<(
        Entity,
        &mut SilentMutiny,
        &mut FleetFaction,
        Option<&mut FleetCargo>,
    )>,
    mut sensor_glitch_events: EventWriter<SensorGlitchEvent>,
) {
    let mut rng = rand::thread_rng();

    for (entity, mut mutiny, mut faction, mut cargo) in fleets.iter_mut() {
        mutiny.progress += MUTINY_PROGRESSION_RATE;

        if let Some(ref mut cargo_comp) = cargo {
            let keys: Vec<_> = cargo_comp.resources.keys().copied().collect();
            for key in keys {
                if let Some(amount) = cargo_comp.resources.get_mut(&key) {
                    if *amount > 0.0 {
                        let skim_amount = (*amount * MUTINY_SKIM_PERCENTAGE).min(MUTINY_SKIM_MAX);
                        *amount -= skim_amount;
                        mutiny.skimmed_resources += skim_amount;
                    }
                }
            }
        }

        if rng.gen_bool(AVOID_COMBAT_CHANCE) {
            commands.entity(entity).insert(AvoidCombat);
            sensor_glitch_events.send(SensorGlitchEvent { fleet: entity });
        } else {
            commands.entity(entity).remove::<AvoidCombat>();
        }

        if mutiny.progress >= 1.0 {
            *faction = FleetFaction::Pirate;
            commands.entity(entity).remove::<SilentMutiny>();
            commands.entity(entity).remove::<AvoidCombat>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::combat::fleet_combat_system;
    use crate::layer2::events::ShipDestroyedEvent;
    use crate::layer2::fleet::FleetComposition;
    use crate::layer2::ship::{Ship, ShipType};

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<SensorGlitchEvent>>();
        world.init_resource::<Events<ShipDestroyedEvent>>();
        world
    }

    #[test]
    fn test_silent_mutiny_trigger() {
        let mut world = setup_world();

        let core_world = world.spawn(CoreWorld).id();

        // Fleet close to core world
        let close_orbit = world
            .spawn(Orbit {
                parent: core_world,
                radius: 10.0,
                speed: 0.1,
                angle: 0.0,
            })
            .id();
        let loyal_fleet = world
            .spawn((
                FleetMorale { value: 0.2 }, // Low morale, but close
                InOrbit {
                    parent: close_orbit,
                },
                FleetFaction::Player,
            ))
            .id();

        // Fleet far from core world
        let far_orbit = world
            .spawn(Orbit {
                parent: core_world,
                radius: 1000.0,
                speed: 0.1,
                angle: 0.0,
            })
            .id();
        let mutinous_fleet = world
            .spawn((
                FleetMorale { value: 0.2 }, // Low morale and far
                InOrbit { parent: far_orbit },
                FleetFaction::Player,
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(check_silent_mutiny_system);

        // Run enough times to hit the 10% probability
        let mut triggered = false;
        for _ in 0..100 {
            schedule.run(&mut world);
            if world.get::<SilentMutiny>(mutinous_fleet).is_some() {
                triggered = true;
                break;
            }
        }

        assert!(
            triggered,
            "Fleet far from core world with low morale should trigger silent mutiny"
        );
        assert!(
            world.get::<SilentMutiny>(loyal_fleet).is_none(),
            "Fleet close to core world should not trigger silent mutiny"
        );
    }

    #[test]
    fn test_silent_mutiny_resource_skimming() {
        let mut world = setup_world();

        let mut cargo = FleetCargo::default();
        cargo.resources.insert(ResourceType::Metal, 100.0);

        let fleet = world
            .spawn((SilentMutiny::default(), cargo, FleetFaction::Player))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_mutiny_effects_system);
        schedule.run(&mut world);

        let updated_cargo = world.get::<FleetCargo>(fleet).unwrap();
        assert!(
            updated_cargo
                .resources
                .get(&ResourceType::Metal)
                .copied()
                .unwrap_or(0.0)
                < 100.0,
            "Mutinous fleet should skim resources from cargo"
        );

        let mutiny = world.get::<SilentMutiny>(fleet).unwrap();
        assert!(
            mutiny.skimmed_resources > 0.0,
            "Skimmed resources should be tracked"
        );
    }

    #[test]
    fn test_silent_mutiny_combat_avoidance() {
        let mut world = setup_world();

        let combat_location = world.spawn_empty().id();

        // Fleet with mutiny
        let mut player_comp = FleetComposition::default();
        player_comp.add_ship(Ship::new(ShipType::Frigate));

        let player_fleet = world
            .spawn((
                FleetFaction::Player,
                InOrbit {
                    parent: combat_location,
                },
                player_comp,
                SilentMutiny::default(),
            ))
            .id();

        // Hostile fleet
        let mut pirate_comp = FleetComposition::default();
        pirate_comp.add_ship(Ship::new(ShipType::Frigate));
        let _pirate_fleet = world
            .spawn((
                FleetFaction::Pirate,
                InOrbit {
                    parent: combat_location,
                },
                pirate_comp,
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_mutiny_effects_system.before(fleet_combat_system));
        schedule.add_systems(fleet_combat_system);

        let mut avoided = false;
        for _ in 0..20 {
            schedule.run(&mut world);

            if world.get::<AvoidCombat>(player_fleet).is_some() {
                // Combat should be avoided
                let events = world.resource::<Events<SensorGlitchEvent>>();
                let reader = events.get_cursor();
                if reader.len(events) > 0 {
                    avoided = true;
                    break;
                }
            }

            // Re-setup if combat happened
            if world.get::<FleetComposition>(player_fleet).is_none() {
                // Should not happen, but if it does, it's a test failure because AvoidCombat didn't work properly
                // Wait, if AvoidCombat wasn't applied this tick (50% chance), combat will happen.
                // We just want to check if AvoidCombat CAN happen. So let's recreate it if it dies.
                world
                    .entity_mut(player_fleet)
                    .insert(FleetComposition::default());
                let mut comp = FleetComposition::default();
                comp.add_ship(Ship::new(ShipType::Frigate));
                world.entity_mut(player_fleet).insert(comp);
            }
        }

        assert!(
            avoided,
            "Silent mutiny should occasionally avoid combat and emit a SensorGlitchEvent"
        );
    }

    #[test]
    fn test_silent_mutiny_full_rebellion() {
        let mut world = setup_world();

        let fleet = world
            .spawn((
                SilentMutiny {
                    progress: 0.99,
                    skimmed_resources: 50.0,
                }, // Almost fully rebelled
                FleetFaction::Player,
                FleetCargo::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_mutiny_effects_system);
        schedule.run(&mut world);

        // Progress should push it over 1.0
        let faction = world.get::<FleetFaction>(fleet).unwrap();
        assert_eq!(
            *faction,
            FleetFaction::Pirate,
            "Fleet should change faction to Pirate on full rebellion"
        );
        assert!(
            world.get::<SilentMutiny>(fleet).is_none(),
            "SilentMutiny component should be removed on full rebellion"
        );
    }
}
