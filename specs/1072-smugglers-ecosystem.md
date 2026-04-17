# 1072: The Smuggler's Ecosystem

## 1. Overview

"The Smuggler's Ecosystem" introduces a shadow economy that thrives when strict trade embargoes or high tariffs create unmet needs. When a colony suffers from unfulfilled demands due to policy restrictions, "Smuggler Hubs" will spontaneously form in unmonitored layer 2 nodes (e.g., asteroid belts or abandoned stations). These hubs bypass standard logistics networks, fulfilling unmet Layer 1 needs at steep markups, while simultaneously increasing local crime rates and corruption. This creates a tension between tolerating the black market to keep citizens happy and spending military resources to hunt down illicit suppliers.

## 2. Dependencies

- Layer 2 nodes / Asteroid belts infrastructure
- Existing Needs and Policy/Embargo systems

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::market::{TradePolicy, Embargo};
    use crate::layer2::nodes::{Node, NodeType};
    use crate::layer1::needs::{Needs, ItemType};
    use crate::layer1::colony::CrimeRate;

    #[test]
    fn test_smuggler_hub_spawns_on_unmet_embargoed_need() {
        let mut app = App::new();
        app.add_plugins(SmugglerEcosystemPlugin);

        // Arrange: Create a colony with an unmet need and an embargo on the item
        let colony_entity = app.world_mut().spawn((
            Colony,
            Needs::new(vec![(ItemType::LuxuryGoods, 0.0)]), // unmet need
            TradePolicy { embargoes: vec![ItemType::LuxuryGoods] },
            CrimeRate(0.0),
        )).id();

        // Create an unmonitored asteroid belt
        let asteroid_node = app.world_mut().spawn((
            Node { node_type: NodeType::AsteroidBelt },
            Unmonitored,
        )).id();

        // Act: Run the simulation to trigger smuggler hub formation
        app.update();

        // Assert: A Smuggler Hub should spawn on the unmonitored node
        let has_hub = app.world().get::<SmugglerHub>(asteroid_node).is_some();
        assert!(has_hub, "Smuggler Hub should spawn when embargoed goods are demanded");
    }

    #[test]
    fn test_smugglers_fulfill_needs_and_increase_crime() {
        let mut app = App::new();
        app.add_plugins(SmugglerEcosystemPlugin);

        let colony_entity = app.world_mut().spawn((
            Colony,
            Needs::new(vec![(ItemType::LuxuryGoods, 0.0)]),
            TradePolicy { embargoes: vec![ItemType::LuxuryGoods] },
            CrimeRate(0.0),
        )).id();

        let hub_entity = app.world_mut().spawn((
            Node { node_type: NodeType::AsteroidBelt },
            SmugglerHub { active: true, target_colony: colony_entity, supplied_item: ItemType::LuxuryGoods },
        )).id();

        // Act: Smugglers supply goods
        app.update();

        // Assert: Needs are met, but crime goes up
        let colony_needs = app.world().get::<Needs>(colony_entity).unwrap();
        assert!(colony_needs.get_fulfillment(ItemType::LuxuryGoods) > 0.0, "Smugglers should fulfill the embargoed need");

        let crime_rate = app.world().get::<CrimeRate>(colony_entity).unwrap();
        assert!(crime_rate.0 > 0.0, "Smuggler activity should increase colony crime rate");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

pub struct SmugglerEcosystemPlugin;

impl Plugin for SmugglerEcosystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_smuggler_hubs, process_smuggler_trade));
    }
}

#[derive(Component)]
pub struct SmugglerHub {
    pub active: bool,
    pub target_colony: Entity,
    pub supplied_item: ItemType,
}

#[derive(Component)]
pub struct Unmonitored;

fn spawn_smuggler_hubs(
    mut commands: Commands,
    colonies: Query<(Entity, &Needs, &TradePolicy)>,
    unmonitored_nodes: Query<Entity, (With<Node>, With<Unmonitored>, Without<SmugglerHub>)>,
) {
    for (colony_ent, needs, policy) in colonies.iter() {
        for embargoed_item in &policy.embargoes {
            if needs.get_fulfillment(*embargoed_item) <= 0.0 {
                if let Some(node_ent) = unmonitored_nodes.iter().next() {
                    commands.entity(node_ent).insert(SmugglerHub {
                        active: true,
                        target_colony: colony_ent,
                        supplied_item: *embargoed_item,
                    });
                }
            }
        }
    }
}

fn process_smuggler_trade(
    hubs: Query<&SmugglerHub>,
    mut colonies: Query<(&mut Needs, &mut CrimeRate)>,
) {
    for hub in hubs.iter() {
        if hub.active {
            if let Ok((mut needs, mut crime)) = colonies.get_mut(hub.target_colony) {
                needs.set_fulfillment(hub.supplied_item, 1.0); // Fulfill need
                crime.0 += 0.1; // Increase crime
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Refactoring Opportunities**:
  - Abstract the economic calculation so smugglers charge actual "credits" from the colony, reducing colony wealth.
  - Introduce `SmugglerFleet` entities moving from the Hub to the Colony instead of instantaneous fulfillment.
- **Code Smells**: Instant need fulfillment is a bit simplistic and ignores logistics/travel time.
- **Performance Considerations**: Iterating over all unmonitored nodes for every unmet need could be expensive on a large map. Spatial hashing or indexing `Unmonitored` nodes could optimize hub placement.
- **API Improvements**: Use events (e.g., `SmugglingAttemptEvent`) so security forces have a chance to intercept the goods before they fulfill needs and increase crime.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code in `layer2::smuggling`.
- [ ] Smuggler hubs spontaneously form on unmonitored nodes when embargoed goods are demanded.
- [ ] Active smuggling operations fulfill colony needs but increase the `CrimeRate` component.

## 7. Technical Guidance

- Place this code in a new module, e.g., `src/layer2/smuggling.rs`.
- Integrate with `Chronicle` so major smuggler hub formations create a lore event.
- Ensure the `CrimeRate` increases linearly or uses existing diminishing returns mechanics, rather than growing infinitely.
- Consider what happens when an `Unmonitored` node becomes monitored (e.g., a player stations a patrol fleet there). The `SmugglerHub` should either be suppressed, destroyed, or moved.

## 8. Questions

*Builder: add questions here if spec is unclear.*
