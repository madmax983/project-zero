//! Orbital debris mechanics.
//!
//! Handles the accumulation of debris from launches and combat,
//! and the risks it poses to ships and future launches.

use crate::layer2::events::{LaunchEvent, ShipDestroyedEvent};
use crate::layer2::fleet::{Fleet, FleetComposition, FleetHealth, InOrbit};
use bevy_ecs::prelude::*;

/// Component tracking the amount of debris in orbit around a body.
#[derive(Component, Default, Debug)]
pub struct OrbitalDebris {
    /// The density of debris, where 0.0 is clear and 1.0 is extremely hazardous.
    /// Can exceed 1.0.
    pub amount: f32,
}

/// System that increases orbital debris based on events.
pub fn debris_accumulation_system(
    mut events_launch: EventReader<LaunchEvent>,
    mut events_destroy: EventReader<ShipDestroyedEvent>,
    mut query: Query<&mut OrbitalDebris>,
) {
    for event in events_launch.read() {
        if let Ok(mut debris) = query.get_mut(event.planet) {
            if event.success {
                debris.amount += 0.05;
            } else {
                debris.amount += 0.10;
            }
        }
    }

    for event in events_destroy.read() {
        if let Ok(mut debris) = query.get_mut(event.planet) {
            debris.amount += 0.20;
        }
    }
}

/// System that applies damage to fleets in orbit if debris is high.
pub fn debris_attrition_system(
    mut commands: Commands,
    planet_query: Query<&OrbitalDebris>,
    mut fleet_query: Query<
        (
            Entity,
            &InOrbit,
            &mut FleetHealth,
            Option<&mut FleetComposition>,
        ),
        With<Fleet>,
    >,
    mut event_writer: EventWriter<ShipDestroyedEvent>,
) {
    for (entity, orbit, mut health, mut maybe_comp) in &mut fleet_query {
        let Ok(debris) = planet_query.get(orbit.parent) else {
            continue;
        };

        if debris.amount > 0.1 {
            // Damage based on debris amount
            let damage = debris.amount * 5.0;

            if let Some(ref mut comp) = maybe_comp {
                // Concrete fleet with ships
                let destroyed = comp.take_damage(damage);
                for ship_type in destroyed {
                    event_writer.send(ShipDestroyedEvent {
                        planet: orbit.parent,
                        ship_class: format!("{ship_type:?}"),
                    });
                }

                // Recalculate health from comp
                let total_health: f32 = comp.ships.iter().map(|s| s.health).sum();
                let max_health: f32 = comp.ships.iter().map(|s| s.max_health).sum();
                health.current = total_health;
                health.max = max_health;

                if comp.ships.is_empty() {
                    commands.entity(entity).despawn();
                }
            } else {
                // Abstract fleet
                health.current = (health.current - damage).max(0.0);
                if health.current <= 0.001 {
                    commands.entity(entity).despawn();
                    event_writer.send(ShipDestroyedEvent {
                        planet: orbit.parent,
                        ship_class: "Unknown Fleet".to_string(),
                    });
                }
            }
        }
    }
}

/// System that slowly decays orbital debris over time due to atmospheric drag.
pub fn debris_decay_system(mut query: Query<&mut OrbitalDebris>) {
    for mut debris in &mut query {
        if debris.amount > 0.0 {
            // Decay by 0.5% per tick
            debris.amount *= 0.995;
            // Floor at very low values to avoid floating point drift
            if debris.amount < 0.001 {
                debris.amount = 0.0;
            }
        }
    }
}

/// Calculates the risk of launch failure given the debris amount.
///
/// Returns a probability between 0.0 and 1.0.
/// TODO: Integrate this with Launch Logistics (Spec 105) when implemented.
#[must_use]
pub fn calculate_launch_risk(debris_amount: f32) -> f32 {
    // Sigmoid or linear scaling
    // Spec test: calculate_launch_risk(0.8) > 0.5
    // Spec test: calculate_launch_risk(0.0) == 0.0

    // Linear capping at 0.9
    (debris_amount * 0.8).min(0.9)
}

/// Helper function to perform debris cleanup on a planet.
pub fn perform_cleanup(world: &mut World, planet: Entity, amount: f32) {
    if let Some(mut debris) = world.get_mut::<OrbitalDebris>(planet) {
        debris.amount = (debris.amount - amount).max(0.0);
    }
}
