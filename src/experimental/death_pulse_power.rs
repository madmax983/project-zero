//! The Death Pulse Power Phenomenon (Nova Feature).
//!
//! # The Spark
//! We have a `PopDied` event, `GridPosition`, and `Battery` for power storage.
//!
//! # The Feature
//! A specialized generator component `DeathCapacitor` that absorbs the bio-electric discharge
//! of a dying Pop if they perish within a short radius of it, instantly charging its connected `Battery`.
//!
//! # The Potential
//! This turns a pure negative (losing a Pop) into a momentary, potentially life-saving
//! massive energy spike, incentivizing morbid strategies like executing Pops to power the colony during blackouts.
//! Especially relevant for "death cult" playthroughs or handling massive disaster casualties.

use crate::layer1::energy::Battery;
use crate::layer1::entities::pop::PopDied;
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

/// A building component that harvests energy from dying pops nearby.
#[derive(Component)]
pub struct DeathCapacitor {
    /// Radius in tiles within which it can absorb death pulses.
    pub radius: i32,
    /// Amount of energy generated per death.
    pub pulse_yield: f32,
}

/// A system that captures the "death pulse" and feeds it into local batteries.
pub fn death_pulse_power_system(
    mut died_events: EventReader<PopDied>,
    positions: Query<&GridPosition>, // To lookup where the entity died (if it hasn't been despawned completely)
    mut capacitors: Query<(&GridPosition, &DeathCapacitor, &mut Battery)>,
) {
    for event in died_events.read() {
        // Since the pop entity might still have its GridPosition for this tick before full despawn,
        // or we just query it. Alternatively, we don't have position directly in the event,
        // but we can look it up from the entity!
        if let Ok(death_pos) = positions.get(event.entity) {
            for (cap_pos, capacitor, mut battery) in capacitors.iter_mut() {
                let dx = (cap_pos.x - death_pos.x).abs();
                let dy = (cap_pos.y - death_pos.y).abs();
                let distance = dx.max(dy);

                if distance <= capacitor.radius {
                    // Harvest the pulse!
                    battery.charge += capacitor.pulse_yield;
                    if battery.charge > battery.capacity {
                        battery.charge = battery.capacity;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<PopDied>>();
        world
    }

    #[test]
    fn test_death_pulse_charges_battery() {
        let mut world = setup_world();

        let capacitor_entity = world
            .spawn((
                GridPosition { x: 5, y: 5 },
                DeathCapacitor {
                    radius: 3,
                    pulse_yield: 50.0,
                },
                Battery {
                    charge: 10.0,
                    capacity: 100.0,
                    max_throughput: 100.0,
                },
            ))
            .id();

        let dying_pop = world.spawn(GridPosition { x: 6, y: 6 }).id(); // Within radius

        world.send_event(PopDied {
            entity: dying_pop,
            name: "Timmy".to_string(),
            tick: 0,
            reason: "Tragic accident".to_string(),
        });

        world.run_system_once(death_pulse_power_system).unwrap();

        let battery = world.get::<Battery>(capacitor_entity).unwrap();
        assert_eq!(battery.charge, 60.0);
    }

    #[test]
    fn test_death_pulse_ignores_out_of_range() {
        let mut world = setup_world();

        let capacitor_entity = world
            .spawn((
                GridPosition { x: 5, y: 5 },
                DeathCapacitor {
                    radius: 3,
                    pulse_yield: 50.0,
                },
                Battery {
                    charge: 10.0,
                    capacity: 100.0,
                    max_throughput: 100.0,
                },
            ))
            .id();

        let dying_pop = world.spawn(GridPosition { x: 10, y: 10 }).id(); // Out of radius

        world.send_event(PopDied {
            entity: dying_pop,
            name: "Timmy".to_string(),
            tick: 0,
            reason: "Tragic accident".to_string(),
        });

        world.run_system_once(death_pulse_power_system).unwrap();

        let battery = world.get::<Battery>(capacitor_entity).unwrap();
        assert_eq!(battery.charge, 10.0);
    }
}
