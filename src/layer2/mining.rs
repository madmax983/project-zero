use crate::layer1::resources::ResourceType;
use crate::layer2::fleet::{Fleet, FleetOrder, InOrbit};
use bevy_ecs::prelude::*;

/// Component for orbital bodies that can be mined.
#[derive(Component, Debug, Clone)]
pub struct MiningTarget {
    /// The type of resource available.
    pub resource_type: ResourceType,
    /// The amount of resource remaining.
    pub amount: f32,
    /// Multiplier for time taken to mine.
    pub mining_difficulty: f32,
}

/// Represents a single stack of cargo in a fleet.
#[derive(Debug, Clone, Copy)]
pub struct CargoStack {
    /// The type of resource in this stack.
    pub resource_type: ResourceType,
    /// The amount of the resource.
    pub amount: f32,
}

/// Component for fleets to store mined resources.
#[derive(Component, Debug, Clone, Default)]
pub struct FleetCargo {
    /// The list of resource stacks.
    pub contents: Vec<CargoStack>,
    /// The total capacity of the cargo hold.
    pub capacity: f32,
}

impl FleetCargo {
    /// Calculates the current total load of the cargo.
    #[must_use]
    pub fn current_load(&self) -> f32 {
        self.contents.iter().map(|s| s.amount).sum()
    }

    /// Adds a resource to the cargo, respecting capacity.
    /// Returns the amount actually added.
    pub fn add(&mut self, resource_type: ResourceType, amount: f32) -> f32 {
        let current = self.current_load();
        let available = self.capacity - current;
        let to_add = amount.min(available);

        if to_add <= 0.0 {
            return 0.0;
        }

        // Check if stack exists
        if let Some(stack) = self.contents.iter_mut().find(|s| s.resource_type == resource_type) {
            stack.amount += to_add;
        } else {
            self.contents.push(CargoStack {
                resource_type,
                amount: to_add,
            });
        }
        to_add
    }
}

/// Component indicating a fleet is actively mining.
#[derive(Component, Debug, Clone)]
pub struct FleetMining {
    /// The entity being mined.
    pub target: Entity,
    /// The rate of mining per tick.
    pub rate: f32,
}

/// System to handle Mine orders.
#[allow(clippy::type_complexity)]
pub fn fleet_mine_order_system(
    mut commands: Commands,
    query: Query<(Entity, &FleetOrder, Option<&InOrbit>, Option<&FleetCargo>), With<Fleet>>,
) {
    for (entity, order, maybe_orbit, maybe_cargo) in &query {
        if let FleetOrder::Mine(target_entity) = order {
            // Must be in orbit of target, and parent must match target
            let in_orbit = maybe_orbit.is_some_and(|orbit| orbit.parent == *target_entity);

            // Must have cargo space
            let has_space = maybe_cargo.is_some_and(|cargo| cargo.current_load() < cargo.capacity);

            if in_orbit && has_space {
                // Start Mining
                commands
                    .entity(entity)
                    .remove::<FleetOrder>()
                    .insert(FleetMining {
                        target: *target_entity,
                        rate: 1.0, // Default base rate
                    });
                continue;
            }

            // Invalid order (wrong location or full) - consume it to prevent infinite loop
            commands.entity(entity).remove::<FleetOrder>();
        }
    }
}

/// System to process mining over time.
pub fn mining_system(
    mut commands: Commands,
    mut fleets: Query<(Entity, &mut FleetMining, &mut FleetCargo)>,
    mut targets: Query<&mut MiningTarget>,
) {
    for (fleet_entity, mining, mut cargo) in &mut fleets {
        if let Ok(mut target) = targets.get_mut(mining.target) {
            if target.amount <= 0.0 {
                // Depleted
                commands.entity(fleet_entity).remove::<FleetMining>();
                continue;
            }

            // Clamp amount to what's available in the target
            let amount_to_mine = mining.rate.min(target.amount);

            // Try to add to cargo (handles capacity)
            let actually_added = cargo.add(target.resource_type, amount_to_mine);

            // Deduct from target (only what was actually taken)
            target.amount -= actually_added;

            // Stop if full or depleted
            if cargo.current_load() >= cargo.capacity || target.amount <= 0.0 {
                commands.entity(fleet_entity).remove::<FleetMining>();
            }
        } else {
            // Target destroyed/despawned
            commands.entity(fleet_entity).remove::<FleetMining>();
        }
    }
}
