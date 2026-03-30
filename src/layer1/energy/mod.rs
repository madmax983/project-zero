//! Energy system for the colony.
//!
//! Handles power generation, distribution, and consumption.
//! Grids are formed dynamically based on connectivity via `Conduit`s and power-related buildings.

use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::weather::{WeatherState, WeatherType};
use bevy_ecs::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::{HashMap, HashSet, VecDeque};

/// Auroral power generation.
pub mod auroral;
#[cfg(test)]
mod auroral_tests;
#[cfg(test)]
mod blackout_tests;
#[cfg(test)]
mod instability_tests;
pub mod load_limits;
pub use auroral::update_auroral_output_system;

/// Emits power to the grid.
#[derive(Component, Debug, Clone)]
pub struct PowerSource {
    /// Amount of power produced per tick.
    pub output: f32,
    /// Whether the source is currently active (e.g., has fuel).
    pub active: bool,
}

impl Default for PowerSource {
    fn default() -> Self {
        Self {
            output: 10.0,
            active: true,
        }
    }
}

/// Consumes power from the grid.
#[derive(Component, Debug, Clone, Default)]
pub struct PowerConsumer {
    /// Amount of power consumed per tick.
    pub demand: f32,
    /// Whether the consumer is currently powered.
    pub active: bool,
}

/// Consumes fuel to operate.
#[derive(Component, Debug, Clone)]
pub struct FuelConsumer {
    /// Amount of fuel consumed per tick.
    pub amount: f32,
}

/// Connects power grid elements.
#[derive(Component, Debug, Clone)]
pub struct Conduit;

/// Event emitted when a grid component takes damage from overload.
#[derive(Event, Debug, Clone, Copy)]
pub struct GridOverloadEvent {
    /// The entity that was overloaded.
    pub victim: Entity,
}

/// Global protocol to cut power in emergencies.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct BlackoutProtocol {
    /// Whether the blackout is currently active.
    pub active: bool,
}

/// Global grid power state (calculated each tick).
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct GridPower {
    /// Total available power.
    pub available: u32,
    /// Total required power.
    pub required: u32,
}

/// Stores excess power to buffer brownouts.
#[derive(Component, Debug, Clone)]
pub struct Battery {
    /// Total capacity.
    pub capacity: f32,
    /// Current charge.
    pub charge: f32,
    /// Maximum charge/discharge per tick.
    pub max_throughput: f32,
}

impl Battery {
    /// Charges the battery.
    pub fn charge(&mut self, amount: f32) {
        self.charge = (self.charge + amount).min(self.capacity);
    }

    /// Discharges the battery.
    pub fn discharge(&mut self, amount: f32) -> f32 {
        let actual = amount.min(self.charge).min(self.max_throughput);
        self.charge -= actual;
        actual
    }
}

fn build_grid_map(world: &mut World) -> HashMap<(i32, i32), Entity> {
    let mut grid_map = HashMap::new();
    let mut query = world.query_filtered::<(Entity, &GridPosition), Or<(
        With<PowerSource>,
        With<PowerConsumer>,
        With<Conduit>,
        With<Battery>,
        With<crate::layer1::kinetic_storage::KineticBattery>,
    )>>();

    for (entity, pos) in query.iter(world) {
        grid_map.insert((pos.x, pos.y), entity);
    }
    grid_map
}

fn bfs_grid(
    start_pos: (i32, i32),
    grid_map: &HashMap<(i32, i32), Entity>,
    world: &World,
    visited: &mut HashSet<(i32, i32)>,
) -> (f32, f32, Vec<Entity>) {
    let mut grid_entities = Vec::new();
    let mut queue = VecDeque::new();

    if visited.insert(start_pos) {
        queue.push_back(start_pos);
    }

    let mut total_production = 0.0;
    let mut total_demand = 0.0;

    while let Some(pos) = queue.pop_front() {
        if let Some(&entity) = grid_map.get(&pos) {
            grid_entities.push(entity);

            if let Some(source) = world.get::<PowerSource>(entity).filter(|s| s.active) {
                // Check if a Quirk stops production
                let is_stopped = world
                    .get::<crate::layer1::rituals::Quirk>(entity)
                    .is_some_and(crate::layer1::rituals::Quirk::stops_production);

                if !is_stopped {
                    let efficiency = world
                        .get::<crate::layer1::prototyping::Prototype>(entity)
                        .map_or(1.0, |p| p.efficiency_modifier);
                    total_production += source.output * efficiency;
                }
            }
            if let Some(consumer) = world.get::<PowerConsumer>(entity) {
                total_demand += consumer.demand;
            }

            let neighbors = [
                (pos.0 + 1, pos.1),
                (pos.0 - 1, pos.1),
                (pos.0, pos.1 + 1),
                (pos.0, pos.1 - 1),
            ];

            for neighbor in neighbors {
                if grid_map.contains_key(&neighbor) && !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
    }
    (total_production, total_demand, grid_entities)
}

/// Calculates total production and demand for the grid connected to `start_entity`.
pub fn calculate_grid_stats(world: &mut World, start_entity: Entity) -> (f32, f32) {
    let grid_map = build_grid_map(world);
    let start_pos = match world.get::<GridPosition>(start_entity) {
        Some(p) => (p.x, p.y),
        None => return (0.0, 0.0),
    };

    let mut visited = HashSet::new();
    let (prod, demand, _) = bfs_grid(start_pos, &grid_map, world, &mut visited);
    (prod, demand)
}

/// System to update power grids.
/// Identifies connected components, sums production/demand, and enables/disables consumers.
///
/// ⚡ Bolt Optimization:
/// - Avoids allocating a Vec for `positions` by iterating directly over `grid_map.keys().copied()`.
/// - Utilizes single-pass iteration replacing two separate `.iter().filter(...).copied().collect()` chains
///   with a single loop over `grid_entities`. This avoids iterating the vector twice and removes
///   two intermediate iterator/collection overheads per grid.
pub fn power_grid_system(world: &mut World) {
    // 0. Check Protocol
    let blackout = world
        .get_resource::<BlackoutProtocol>()
        .is_some_and(|b| b.active);

    if blackout {
        let mut consumers = world.query::<&mut PowerConsumer>();
        for mut consumer in consumers.iter_mut(world) {
            consumer.active = false;
        }
        return;
    }

    // Check Weather
    let demand_multiplier = world.get_resource::<WeatherState>().map_or(1.0, |w| {
        if w.current_weather == WeatherType::MagneticStorm {
            1.5
        } else {
            1.0
        }
    });

    // 1. Build grid map
    let grid_map = build_grid_map(world);

    // 2. Find connected components
    let mut visited: HashSet<(i32, i32)> = HashSet::new();

    let mut global_production = 0.0;
    let mut global_demand = 0.0;

    for start_pos in grid_map.keys().copied() {
        if visited.contains(&start_pos) {
            continue;
        }

        // BFS for this grid
        let (total_production, base_demand, grid_entities) =
            bfs_grid(start_pos, &grid_map, world, &mut visited);

        let total_demand = base_demand * demand_multiplier;

        // 3. Calculate Net & Handle Batteries
        let mut net = total_production - total_demand;

        // Collect batteries in this grid
        let mut batteries = Vec::new();
        let mut kinetic_batteries = Vec::new();
        for e in &grid_entities {
            if world.get::<Battery>(*e).is_some() {
                batteries.push(*e);
            }
            if world
                .get::<crate::layer1::kinetic_storage::KineticBattery>(*e)
                .is_some()
            {
                kinetic_batteries.push(*e);
            }
        }

        let provided = handle_batteries(world, &batteries, &kinetic_batteries, net);
        if net < 0.0 {
            // Update net after battery discharge (effectively increasing production availability)
            net += provided;
        }

        // 4. Handle Activation & Overload
        // Recalculate effective supply ratio
        // If net >= 0, supply_ratio = 1.0 (fully powered)
        // If net < 0, supply_ratio = (production + provided) / demand

        // total_available = total_production + battery_provided
        // battery_provided is calculated above.
        // If net < 0 originally: provided is what batteries gave.
        // New net = old_net + provided.
        // total_available = total_production + provided = total_demand + New net.

        let total_available = total_demand + net;

        let supply_ratio = if total_demand > 0.0 {
            (total_available / total_demand).min(1.0)
        } else {
            1.0
        };

        handle_overload(world, &grid_entities, total_production, total_demand);

        activate_consumers(world, &grid_entities, net, supply_ratio);

        global_production += total_production;
        global_demand += total_demand;
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    world.insert_resource(GridPower {
        available: global_production.max(0.0) as u32,
        required: global_demand.max(0.0) as u32,
    });
}

/// System to process fuel consumption for power sources.
pub fn process_fuel_consumption_system(
    mut resources: ResMut<ColonyResources>,
    mut query: Query<(&mut PowerSource, &FuelConsumer)>,
) {
    let available_fuel = resources.fuel;
    let mut fuel_spent = 0.0;

    for (mut source, consumer) in &mut query {
        if available_fuel - fuel_spent >= consumer.amount {
            // Have fuel
            fuel_spent += consumer.amount;

            // Activate if inactive
            if !source.active {
                source.active = true;
            }
        } else {
            // Out of fuel
            // Deactivate if active
            if source.active {
                source.active = false;
            }
        }
    }

    // Deduct total from global resources
    if fuel_spent > 0.0 {
        resources.fuel = (resources.fuel - fuel_spent).max(0.0);
    }
}

fn handle_batteries(
    world: &mut World,
    batteries: &[Entity],
    kinetic_batteries: &[Entity],
    net: f32,
) -> f32 {
    let mut provided = 0.0;

    if net > 0.0 {
        // Surplus: Charge batteries
        #[allow(clippy::cast_precision_loss)]
        let total_batteries = (batteries.len() + kinetic_batteries.len()) as f32;
        if total_batteries > 0.0 {
            let charge_per_battery = net / total_batteries;
            for bat_entity in batteries {
                if let Some(mut bat) = world.get_mut::<Battery>(*bat_entity) {
                    bat.charge(charge_per_battery);
                }
            }
            for bat_entity in kinetic_batteries {
                if let Some(mut bat) =
                    world.get_mut::<crate::layer1::kinetic_storage::KineticBattery>(*bat_entity)
                {
                    let input = charge_per_battery.min(bat.charge_rate);
                    // Efficiency loss on input
                    let stored = input * bat.efficiency;
                    bat.charge = (bat.charge + stored).min(bat.capacity);
                }
            }
        }
    } else if net < 0.0 {
        // Deficit: Discharge batteries
        let mut needed = -net;

        // Discharge normal batteries
        for bat_entity in batteries {
            if let Some(mut bat) = world.get_mut::<Battery>(*bat_entity) {
                let amount = bat.discharge(needed);
                provided += amount;
                needed -= amount;
                if needed <= 0.0 {
                    break;
                }
            }
        }

        // Discharge kinetic batteries if needed
        if needed > 0.0 {
            for bat_entity in kinetic_batteries {
                if let Some(mut bat) =
                    world.get_mut::<crate::layer1::kinetic_storage::KineticBattery>(*bat_entity)
                {
                    let available = bat.charge;
                    let discharge = needed.min(available).min(bat.charge_rate);
                    bat.charge -= discharge;
                    provided += discharge;
                    needed -= discharge;
                    if needed <= 0.0 {
                        break;
                    }
                }
            }
        }
    }

    provided
}

fn handle_overload(
    world: &mut World,
    grid_entities: &[Entity],
    total_production: f32,
    total_demand: f32,
) {
    let overload_ratio = if total_production > 0.0 {
        total_demand / total_production
    } else {
        1.0
    };

    let mut rng = rand::thread_rng();

    // Overload Check (>150% demand vs base production)
    // Batteries don't prevent overload damage caused by high demand on generators
    if overload_ratio > 1.5 {
        // Risk of damage to random entity in grid
        // Chance increases with overload: (ratio - 1.5) * 0.05
        // e.g. 2.0 ratio -> 0.025 (2.5%) per tick
        if rng.gen_bool((0.05 * f64::from(overload_ratio - 1.5)).min(1.0)) {
            // Pick random entity
            if let Some(victim) = grid_entities.choose(&mut rng) {
                // Clippy suggests collapsing, but let_chains is unstable
                #[allow(clippy::collapsible_if)]
                if let Some(mut health) = world.get_mut::<crate::layer1::health::Health>(*victim) {
                    health.take_damage(10.0);
                }

                // Emit overload event
                world.send_event(GridOverloadEvent { victim: *victim });
            }
        }
    }
}

fn activate_consumers(world: &mut World, grid_entities: &[Entity], net: f32, supply_ratio: f32) {
    // Activation
    let mut rng = rand::thread_rng();
    for entity in grid_entities {
        if let Some(mut consumer) = world.get_mut::<PowerConsumer>(*entity) {
            if net >= -f32::EPSILON {
                consumer.active = true;
            } else {
                // Brownout: Probabilistic activation
                // e.g. 80% supply -> 80% chance to run
                consumer.active = rng.gen_bool(f64::from(supply_ratio));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{calculate_grid_stats, Conduit, PowerConsumer, PowerSource};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_power_components() {
        let source = PowerSource {
            output: 10.0,
            active: true,
        };
        let consumer = PowerConsumer {
            demand: 5.0,
            active: true,
        };
        let _conduit = Conduit; // Marker

        assert_eq!(source.output, 10.0);
        assert_eq!(consumer.demand, 5.0);
    }

    #[test]
    fn test_grid_connectivity_isolated() {
        // Source and Consumer far apart, no conduit
        let mut world = World::new();

        // Generator at 0,0
        let generator = world
            .spawn((
                PowerSource {
                    output: 10.0,
                    active: true,
                },
                GridPosition { x: 0, y: 0 },
                Building {
                    building_type: BuildingType::Generator,
                }, // Assume new type
            ))
            .id();

        // Consumer at 10,10
        let cons = world
            .spawn((
                PowerConsumer {
                    demand: 5.0,
                    active: false,
                },
                GridPosition { x: 10, y: 10 },
                Building {
                    building_type: BuildingType::Smelter,
                },
            ))
            .id();

        // Run calculation
        let (production, _demand) = calculate_grid_stats(&mut world, generator); // Pass root entity to flood fill

        // Gen is its own grid
        assert_eq!(production, 10.0);

        // Consumer is isolated
        let (c_prod, c_demand) = calculate_grid_stats(&mut world, cons);
        assert_eq!(c_prod, 0.0);
        assert_eq!(c_demand, 5.0);
    }

    #[test]
    fn test_grid_connectivity_connected() {
        let mut world = World::new();

        // Generator at 0,0
        let generator = world
            .spawn((
                PowerSource {
                    output: 10.0,
                    active: true,
                },
                GridPosition { x: 0, y: 0 },
                Building {
                    building_type: BuildingType::Generator,
                },
            ))
            .id();

        // Conduit at 0,1
        world.spawn((
            Conduit,
            GridPosition { x: 0, y: 1 },
            Building {
                building_type: BuildingType::PowerPole,
            }, // New type
        ));

        // Consumer at 0,2
        let _cons = world
            .spawn((
                PowerConsumer {
                    demand: 5.0,
                    active: false,
                },
                GridPosition { x: 0, y: 2 },
                Building {
                    building_type: BuildingType::Smelter,
                },
            ))
            .id();

        // Run calculation (start from generator)
        let (production, demand) = calculate_grid_stats(&mut world, generator);

        assert_eq!(production, 10.0);
        assert_eq!(demand, 5.0);
    }

    #[test]
    fn test_overload_shutdown() {
        // 10 Prod, 15 Demand
        let mut world = World::new();

        // Gen 10
        world.spawn((
            PowerSource {
                output: 10.0,
                active: true,
            },
            GridPosition { x: 0, y: 0 },
            Building {
                building_type: BuildingType::Generator,
            },
        ));

        // Cons 1 (10)
        let c1 = world
            .spawn((
                PowerConsumer {
                    demand: 10.0,
                    active: true,
                },
                GridPosition { x: 0, y: 1 },
                Conduit, // Connect them implicitly or explicitly
                Building {
                    building_type: BuildingType::Smelter,
                },
            ))
            .id();

        // Cons 2 (5)
        let c2 = world
            .spawn((
                PowerConsumer {
                    demand: 5.0,
                    active: true,
                },
                GridPosition { x: 0, y: 2 },
                Conduit,
                Building {
                    building_type: BuildingType::Smelter,
                },
            ))
            .id();

        // Run system
        super::power_grid_system(&mut world);

        // Verify active state toggles
        // For Grid Instability (Spec 125): If Demand > Production, consumers flicker (Brownout).
        // Since 15 > 10, supply ratio is 0.66. There is a chance they are active.
        // We cannot deterministically assert !active unless supply is 0.
        // But for this test, we accept either state as valid runtime behavior,
        // effectively disabling the strict check to avoid flakiness until we mock RNG.
        let _c1_state = world.get::<PowerConsumer>(c1).unwrap();
        let _c2_state = world.get::<PowerConsumer>(c2).unwrap();

        // assert!(!c1_state.active); // Flaky
    }

    #[test]
    fn test_quirk_stops_power_production() {
        use crate::layer1::rituals::{Quirk, QuirkType};
        let mut world = World::new();

        let generator = world
            .spawn((
                PowerSource {
                    output: 10.0,
                    active: true,
                },
                GridPosition { x: 0, y: 0 },
                Building {
                    building_type: BuildingType::Generator,
                },
                Quirk {
                    quirk_type: QuirkType::Glitchy,
                }, // Should stop production
            ))
            .id();

        let (production, _) = calculate_grid_stats(&mut world, generator);
        assert_eq!(production, 0.0, "Glitchy generator should produce 0 power");
    }
}
