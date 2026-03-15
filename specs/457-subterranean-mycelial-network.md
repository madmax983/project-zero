# Specification 457: Subterranean Mycelial Network

## 1. Overview
This feature introduces a vast, living biological transit system beneath the colony in Layer 1. By feeding it specific nutrients, players can "train" it to act as a frictionless conveyor belt for raw resources, replacing mechanical haulers and pneumatic tubes. While it provides incredible logistical throughput, it also acts as a terrifyingly efficient vector for disease and invasive species.

## 2. Dependencies
- `025` Hauling Logistics (or equivalent inventory routing system)
- `120` Crop Diversity / Soil Fertility (for nutrient feeding)
- `034` Pop Health and Damage (for disease outbreaks)
- `111` Conveyor Logistics (for resource moving comparisons)

## 3. RED Phase: Tests First

```rust
// tests/mycelial_network_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use scale::layer1::map::GridPosition;
    use scale::layer1::resources::InventoryItem;
    use scale::layer1::health::ContagionVector;

    fn setup_world() -> World {
        let mut world = World::new();
        // Insert grid, time, and basic resources
        world
    }

    #[test]
    fn test_mycelial_node_transports_item() {
        let mut world = setup_world();

        let source_node = world.spawn((
            MycelialNode { network_id: 1, is_active: true },
            GridPosition { x: 5, y: 5 },
        )).id();

        let target_node = world.spawn((
            MycelialNode { network_id: 1, is_active: true },
            GridPosition { x: 50, y: 50 },
        )).id();

        let item = world.spawn((
            InventoryItem { amount: 100, item_type: "RawOre".to_string() },
            InMycelialTransit { source: source_node, target: target_node, progress: 0.0 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_mycelial_transit_system);

        // Fast forward
        for _ in 0..10 {
            schedule.run(&mut world);
        }

        // Item should have arrived
        let transit = world.get::<InMycelialTransit>(item);
        assert!(transit.is_none());

        let position = world.get::<GridPosition>(item).unwrap();
        assert_eq!(position.x, 50);
        assert_eq!(position.y, 50);
    }

    #[test]
    fn test_mycelial_network_spreads_disease() {
        let mut world = setup_world();

        let source_node = world.spawn((
            MycelialNode { network_id: 1, is_active: true },
            GridPosition { x: 5, y: 5 },
            ContagionVector { disease_id: "SporeRot".to_string(), severity: 1.0 },
        )).id();

        let target_node = world.spawn((
            MycelialNode { network_id: 1, is_active: true },
            GridPosition { x: 50, y: 50 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(mycelial_disease_propagation_system);
        schedule.run(&mut world);

        // The target node should now also be a contagion vector
        let target_contagion = world.get::<ContagionVector>(target_node);
        assert!(target_contagion.is_some());
        assert_eq!(target_contagion.unwrap().disease_id, "SporeRot");
    }

    #[test]
    fn test_mycelial_network_requires_nutrients() {
        let mut world = setup_world();

        let node = world.spawn((
            MycelialNode { network_id: 1, is_active: true },
            GridPosition { x: 10, y: 10 },
            MycelialHunger { current: 100.0, max: 100.0, decay_rate: 10.0 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(mycelial_upkeep_system);

        // Fast forward to starve the node
        for _ in 0..15 {
            schedule.run(&mut world);
        }

        let starved_node = world.get::<MycelialNode>(node).unwrap();
        assert!(!starved_node.is_active); // Node should deactivate
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/logistics/mycelial.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::health::ContagionVector;

#[derive(Component, Debug, Clone)]
pub struct MycelialNode {
    pub network_id: u32,
    pub is_active: bool,
}

#[derive(Component, Debug, Clone)]
pub struct InMycelialTransit {
    pub source: Entity,
    pub target: Entity,
    pub progress: f32, // 0.0 to 1.0
}

#[derive(Component, Debug, Clone)]
pub struct MycelialHunger {
    pub current: f32,
    pub max: f32,
    pub decay_rate: f32,
}

pub fn process_mycelial_transit_system(
    mut commands: Commands,
    mut transits: Query<(Entity, &mut InMycelialTransit)>,
    nodes: Query<(&MycelialNode, &GridPosition)>,
) {
    for (entity, mut transit) in transits.iter_mut() {
        // Fast transit speed (e.g., 20% per tick)
        transit.progress += 0.2;

        if transit.progress >= 1.0 {
            if let Ok((target_node, target_pos)) = nodes.get(transit.target) {
                if target_node.is_active {
                    commands.entity(entity).remove::<InMycelialTransit>();
                    commands.entity(entity).insert(*target_pos);
                }
            } else {
                // If target lost, dump at source or despawn
                commands.entity(entity).remove::<InMycelialTransit>();
            }
        }
    }
}

pub fn mycelial_disease_propagation_system(
    mut commands: Commands,
    infected_nodes: Query<(&MycelialNode, &ContagionVector)>,
    mut all_nodes: Query<(Entity, &MycelialNode, Option<&ContagionVector>)>,
) {
    // Collect active diseases by network ID
    let mut network_diseases = std::collections::HashMap::new();
    for (node, contagion) in infected_nodes.iter() {
        if node.is_active {
            network_diseases.insert(node.network_id, contagion.clone());
        }
    }

    // Spread to all connected active nodes
    for (entity, node, current_contagion) in all_nodes.iter_mut() {
        if node.is_active && current_contagion.is_none() {
            if let Some(disease) = network_diseases.get(&node.network_id) {
                commands.entity(entity).insert(disease.clone());
            }
        }
    }
}

pub fn mycelial_upkeep_system(
    mut nodes: Query<(&mut MycelialNode, &mut MycelialHunger)>,
) {
    for (mut node, mut hunger) in nodes.iter_mut() {
        if hunger.current > 0.0 {
            hunger.current -= hunger.decay_rate;
        }

        if hunger.current <= 0.0 {
            hunger.current = 0.0;
            node.is_active = false;
        } else {
            node.is_active = true;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding & Capacity**: The MVP assumes direct A-to-B logic. We should implement limits on how much bandwidth a `MycelialNode` can handle per tick.
- **Disease Immunity**: Add specific biological countermeasures (fungicides) that can clear `ContagionVector` from nodes.
- **Visuals**: Add pulsing visual effects on grid tiles connecting the nodes to represent the flow of resources.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/logistics/mycelial.rs`.
- [ ] Items are transported between active nodes.
- [ ] Diseases introduced at one node rapidly infect all other active nodes in the same network.
- [ ] Nodes deactivate when starved of nutrients.

## 7. Technical Guidance
- `InMycelialTransit` is best used as a temporary component on the inventory items themselves while they are in the network.
- `ContagionVector` needs to be defined if it isn't already, likely in `src/layer1/health/mod.rs` or similar.

## 8. Questions
*Builder: add questions here if spec is unclear.*
