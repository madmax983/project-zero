# Specification: 457 - Subterranean Mycelial Network

## 1. Overview
The **Subterranean Mycelial Network** introduces a biological transit system beneath the colony in Layer 1. Players can discover and "train" a vast fungal network by feeding it specific nutrients. This network acts as a hyper-fast, frictionless conveyor belt for raw resources, replacing mechanical haulers and pneumatic tubes. However, this unmatched logistical throughput comes with a severe risk: the network can inadvertently become a vector for disease, instantly spreading pathogens from one sector (e.g., agricultural) directly into all connected stockpiles and residential zones. This creates a tension between incredible, free logistical power and the terrifying vulnerability of a biological single-point-of-failure.

## 2. Dependencies
- `018` Mining Resources (for raw materials)
- `022` Resource Stockpiles (for storage connection)
- `025` Hauling Logistics (for the existing hauling system it replaces)
- `111` Conveyor Logistics (for comparison/integration with existing transit)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use scale::layer1::map::GridPosition;
    use scale::layer1::item::{ResourceType, Inventory};
    use scale::layer1::contagion::{DiseaseCarrier, ContagionEvent};

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup base systems
        world
    }

    #[test]
    fn test_mycelial_network_transports_resources_instantly() {
        let mut world = setup_world();

        let source_node = world.spawn((
            MycelialNode { connected: true },
            GridPosition { x: 10, y: 10 },
            Inventory { items: vec![(ResourceType::Stone, 100.0)] },
        )).id();

        let dest_node = world.spawn((
            MycelialNode { connected: true },
            GridPosition { x: 50, y: 50 },
            Inventory { items: vec![] },
        )).id();

        // Trigger transport event
        world.send_event(MycelialTransportEvent {
            source: source_node,
            destination: dest_node,
            resource: ResourceType::Stone,
            amount: 50.0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(mycelial_transport_system);
        schedule.run(&mut world);

        let source_inv = world.get::<Inventory>(source_node).unwrap();
        let dest_inv = world.get::<Inventory>(dest_node).unwrap();

        assert_eq!(source_inv.get_amount(ResourceType::Stone), 50.0);
        assert_eq!(dest_inv.get_amount(ResourceType::Stone), 50.0);
    }

    #[test]
    fn test_mycelial_network_spreads_disease() {
        let mut world = setup_world();
        world.init_resource::<Events<ContagionEvent>>();

        let infected_node = world.spawn((
            MycelialNode { connected: true },
            GridPosition { x: 10, y: 10 },
            DiseaseCarrier { pathogen_id: "blight_strain_a".to_string() },
        )).id();

        let clean_node_1 = world.spawn((
            MycelialNode { connected: true },
            GridPosition { x: 20, y: 20 },
        )).id();

        let clean_node_2 = world.spawn((
            MycelialNode { connected: true },
            GridPosition { x: 30, y: 30 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(mycelial_contagion_system);
        schedule.run(&mut world);

        // Check if contagion events were fired for connected nodes
        let events = world.resource::<Events<ContagionEvent>>();
        let mut reader = events.get_cursor();
        let contagion_events: Vec<&ContagionEvent> = reader.read(events).collect();

        assert_eq!(contagion_events.len(), 2, "Disease should spread to all connected clean nodes");
        assert!(contagion_events.iter().any(|e| e.target == clean_node_1));
        assert!(contagion_events.iter().any(|e| e.target == clean_node_2));
    }

    #[test]
    fn test_unconnected_nodes_do_not_transport_or_infect() {
        let mut world = setup_world();
        world.init_resource::<Events<ContagionEvent>>();

        let source_node = world.spawn((
            MycelialNode { connected: false },
            GridPosition { x: 10, y: 10 },
            Inventory { items: vec![(ResourceType::Stone, 100.0)] },
            DiseaseCarrier { pathogen_id: "blight_strain_a".to_string() },
        )).id();

        let dest_node = world.spawn((
            MycelialNode { connected: true },
            GridPosition { x: 50, y: 50 },
            Inventory { items: vec![] },
        )).id();

        world.send_event(MycelialTransportEvent {
            source: source_node,
            destination: dest_node,
            resource: ResourceType::Stone,
            amount: 50.0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems((mycelial_transport_system, mycelial_contagion_system));
        schedule.run(&mut world);

        let dest_inv = world.get::<Inventory>(dest_node).unwrap();
        assert_eq!(dest_inv.get_amount(ResourceType::Stone), 0.0, "Unconnected nodes should not transport");

        let events = world.resource::<Events<ContagionEvent>>();
        assert!(events.is_empty(), "Unconnected nodes should not spread disease");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::item::{ResourceType, Inventory};
use crate::layer1::contagion::{DiseaseCarrier, ContagionEvent};

#[derive(Component)]
pub struct MycelialNode {
    pub connected: bool,
}

#[derive(Event)]
pub struct MycelialTransportEvent {
    pub source: Entity,
    pub destination: Entity,
    pub resource: ResourceType,
    pub amount: f32,
}

pub fn mycelial_transport_system(
    mut events: EventReader<MycelialTransportEvent>,
    mut nodes_query: Query<(&MycelialNode, &mut Inventory)>,
) {
    for event in events.read() {
        // Check if both source and dest exist and are connected
        if let Ok([(source_node, mut source_inv), (dest_node, mut dest_inv)]) =
            nodes_query.get_many_mut([event.source, event.destination])
        {
            if source_node.connected && dest_node.connected {
                let available = source_inv.get_amount(event.resource);
                let transfer_amount = available.min(event.amount);

                if transfer_amount > 0.0 {
                    source_inv.remove(event.resource, transfer_amount);
                    dest_inv.add(event.resource, transfer_amount);
                }
            }
        }
    }
}

pub fn mycelial_contagion_system(
    mut contagion_events: EventWriter<ContagionEvent>,
    nodes_query: Query<(Entity, &MycelialNode, Option<&DiseaseCarrier>)>,
) {
    let mut infected_pathogens = Vec::new();

    // Find all active pathogens in connected nodes
    for (_, node, carrier) in nodes_query.iter() {
        if node.connected {
            if let Some(carrier) = carrier {
                if !infected_pathogens.contains(&carrier.pathogen_id) {
                    infected_pathogens.push(carrier.pathogen_id.clone());
                }
            }
        }
    }

    // If there are pathogens in the network, spread to all connected nodes
    if !infected_pathogens.is_empty() {
        for (entity, node, carrier) in nodes_query.iter() {
            if node.connected && carrier.is_none() {
                // For MVP, just spread the first found pathogen
                contagion_events.send(ContagionEvent {
                    target: entity,
                    pathogen_id: infected_pathogens[0].clone(),
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Nutrient Training**: Implement a system where establishing a `MycelialNode` connection requires a continuous upkeep of specific nutrients (e.g., Water, Bio-Waste). If upkeep fails, `connected` becomes false.
- **Disease Resistance**: Add a localized immunity or filter component that players can build on specific nodes to prevent them from receiving pathogens from the network, at a high cost.
- **Visual Feedback**: Add a visual representation (e.g., pulsing fungal tendrils) connecting active nodes on the grid. Change the color if a disease is actively spreading through the network.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes without warnings.
- [ ] Test coverage is ≥85% for `src/layer1/logistics/mycelial.rs` (or equivalent).
- [ ] Connected nodes instantly transfer specified resources via `MycelialTransportEvent`.
- [ ] A disease present on one connected node instantly attempts to spread to all other connected nodes.

## 7. Technical Guidance
- Place the core logic in `src/layer1/logistics/mycelial.rs`.
- The `mycelial_transport_system` requires `nodes_query.get_many_mut` because it transfers resources between two entities simultaneously. Handle errors gracefully if entities are missing.
- Ensure the `ContagionEvent` struct aligns with the existing implementation in `src/layer1/contagion.rs`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
