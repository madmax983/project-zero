# Specification: Fungal Networking

## 1. Overview
A rare underground biome feature called "Mycorrhizal Networks." Buildings constructed over these networks automatically share power and fluid resources without needing pipes or wires. However, the fungus occasionally demands a "tax" by siphoning random resources (food, water, power) to feed itself. This creates tension between cost-saving efficiency and unpredictable living infrastructure.

## 2. Dependencies
- Layer 1 Map & Biomes (Underground features)
- Layer 1 Resources (Power, Fluids, Food)
- Layer 1 Buildings (Consumption/Production)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[derive(Component)]
    struct MycorrhizalNetwork {
        hunger: f32,
    }

    #[derive(Component)]
    struct BuildingNeeds {
        power: f32,
    }

    #[derive(Component)]
    struct BuildingInventory {
        food: f32,
    }

    #[derive(Component)]
    struct OnNetwork(Entity);

    fn fungal_network_sharing_system(
        mut networks: Query<&mut MycorrhizalNetwork>,
        mut buildings: Query<(&mut BuildingNeeds, &OnNetwork)>,
    ) {
        // Implementation
    }

    fn fungal_tax_system(
        mut networks: Query<&mut MycorrhizalNetwork>,
        mut buildings: Query<(&mut BuildingInventory, &OnNetwork)>,
    ) {
        // Implementation
    }

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_fungal_tax_siphons_resources() {
        let mut world = setup_world();

        let network_entity = world.spawn_empty().id();
        world.entity_mut(network_entity).insert(MycorrhizalNetwork { hunger: 10.0 });

        let building_entity = world.spawn_empty().id();
        world.entity_mut(building_entity).insert((
            BuildingInventory { food: 50.0 },
            OnNetwork(network_entity),
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(fungal_tax_system);
        schedule.run(&mut world);

        let building = world.get::<BuildingInventory>(building_entity).unwrap();
        let network = world.get::<MycorrhizalNetwork>(network_entity).unwrap();

        assert!(building.food < 50.0, "Building should have lost food to tax");
        assert!(network.hunger < 10.0, "Network hunger should have decreased");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn fungal_tax_system(
    mut networks: Query<&mut MycorrhizalNetwork>,
    mut buildings: Query<(&mut BuildingInventory, &OnNetwork)>,
) {
    for mut network in networks.iter_mut() {
        if network.hunger > 0.0 {
            for (mut inventory, on_network) in buildings.iter_mut() {
                if inventory.food > 0.0 {
                    let amount = inventory.food.min(5.0);
                    inventory.food -= amount;
                    network.hunger -= amount;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** `fungal_tax_system` currently only steals food. It needs to dynamically select a random or most abundant resource across the entire networked structure.
- **Code Smells:** Using direct queries over all buildings every tick could be slow if there are many networks.
- **API Improvements:** Create an `enum FungalResource { Food, Power, Water }` to parameterize the tax.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] Buildings on the network correctly share resources and suffer sporadic taxing.

## 7. Technical Guidance
- Implement in `layer1/environment/fungal_network.rs`.
- Ensure a warning event is sent before a massive tax to give the player time to respond.

## 8. Questions
*Builder: add questions here if spec is unclear.*
