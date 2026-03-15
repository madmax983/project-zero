# 463: Biological Transit

## 1. Overview

**Fantasy:** Growing a living, pulsating biological transit system beneath your colony that is highly efficient but completely alien.

**Mechanic:** You discover a vast fungal network underground. By feeding it specific nutrients, you can "train" it to act as a hyper-fast, frictionless conveyor belt for raw resources, completely replacing mechanical haulers and pneumatic tubes.

However, this comes with a massive risk. If a disease breaks out in your agricultural sector, the mycelium can inadvertently transport the pathogen directly into every connected stockpile and residential zone simultaneously, turning a localized outbreak into a colony-wide pandemic in minutes.

**Tension:** Unparalleled, free logistical throughput vs. creating a terrifyingly efficient vector for disease and invasive species.

## 2. Dependencies

- `025` — Hauling Logistics (Verified Implemented)
- `111` — Conveyor Logistics (Verified Implemented)
- `457` — Subterranean Mycelial Network (Specced)

## 3. RED Phase: Tests First

Write these tests in `src/layer1/logistics/biological_transit_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::logistics::biological_transit::{FungalNetwork, train_network_system, disease_transmission_system, Pathogen};
    use crate::layer1::item::ResourceType;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_network_training() {
        let mut world = World::new();

        // Spawn a Fungal Network
        let network_entity = world.spawn(FungalNetwork {
            throughput: 0.0,
            connected_nodes: vec![GridPosition { x: 0, y: 0 }],
            is_diseased: false,
        }).id();

        // Add nutrients to train the network
        // ... (Mock resource consumption) ...

        let mut schedule = Schedule::default();
        schedule.add_systems(train_network_system);
        schedule.run(&mut world);

        // Network throughput should increase after training
        let network = world.get::<FungalNetwork>(network_entity).unwrap();
        assert!(network.throughput > 0.0);
    }

    #[test]
    fn test_disease_transmission() {
        let mut world = World::new();

        // Spawn a connected network
        let network_entity = world.spawn(FungalNetwork {
            throughput: 10.0,
            connected_nodes: vec![
                GridPosition { x: 0, y: 0 }, // Node A (e.g., Farm)
                GridPosition { x: 10, y: 10 }, // Node B (e.g., Stockpile)
            ],
            is_diseased: false,
        }).id();

        // Introduce pathogen at Node A
        world.spawn((Pathogen, GridPosition { x: 0, y: 0 }));

        let mut schedule = Schedule::default();
        schedule.add_systems(disease_transmission_system);
        schedule.run(&mut world);

        // Network should become diseased
        let network = world.get::<FungalNetwork>(network_entity).unwrap();
        assert!(network.is_diseased);

        // Pathogen should be transmitted to Node B
        let mut pathogen_at_b = false;
        let mut query = world.query::<(&Pathogen, &GridPosition)>();
        for (_, pos) in query.iter(&world) {
            if pos.x == 10 && pos.y == 10 {
                pathogen_at_b = true;
                break;
            }
        }
        assert!(pathogen_at_b);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/logistics/biological_transit.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;

#[derive(Component, Debug, Clone)]
pub struct FungalNetwork {
    pub throughput: f32,
    pub connected_nodes: Vec<GridPosition>,
    pub is_diseased: bool,
}

#[derive(Component, Debug)]
pub struct Pathogen;

pub fn train_network_system(mut query: Query<&mut FungalNetwork>) {
    for mut network in query.iter_mut() {
        // Simplified training logic: naturally increase throughput over time
        // In a real implementation, this would consume resources (nutrients)
        network.throughput += 1.0;
    }
}

pub fn disease_transmission_system(
    mut commands: Commands,
    mut network_query: Query<&mut FungalNetwork>,
    pathogen_query: Query<&GridPosition, With<Pathogen>>,
) {
    for mut network in network_query.iter_mut() {
        let mut infected = false;

        // Check if any pathogen is at a connected node
        for pathogen_pos in pathogen_query.iter() {
            if network.connected_nodes.contains(pathogen_pos) {
                infected = true;
                break;
            }
        }

        if infected {
            network.is_diseased = true;
            // Spread pathogen to all connected nodes
            for node in &network.connected_nodes {
                // Spawn pathogen at every node
                commands.spawn((Pathogen, *node));
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Pathogen Varieties**: Different pathogens could have varying transmission rates and effects (e.g., targeting specific resources or pops).
- **Network Pruning**: Players should have a mechanism to sever connections or "burn" sections of the network to halt disease spread, sacrificing throughput for safety.
- **Visuals**: Diseased networks should visibly pulsate with sickly colors or emit spores on the map.
- **Nutrient Mechanics**: Training should strictly require specific resources (e.g., Organic Waste) rather than just passive growth.

## 6. Acceptance Criteria

- [ ] `FungalNetwork` and `Pathogen` components are implemented.
- [ ] `train_network_system` increases network throughput.
- [ ] `disease_transmission_system` correctly infects the network and spreads pathogens to all connected nodes if one is exposed.
- [ ] Tests pass.

## 7. Technical Guidance

- Use spatial queries or grid lookups for efficient pathogen detection at nodes instead of iterating over all pathogens.
- Ensure the disease transmission logic doesn't result in infinite pathogen spawning loops (e.g., add a cooldown or check if a pathogen already exists at a node before spawning).
- Integrate the network's throughput directly into the colony's logistics/hauling calculations.

## 8. Questions

*Builder: add questions here if spec is unclear.*
