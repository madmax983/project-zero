# 1076 Fading Homeworld

## 1. Overview
The "Fading Homeworld" feature introduces a dynamic relationship between the Layer 1 colony and its Layer 3 "Core World" (the motherland). Over time, the Core World slowly decays in stability and resources. As it decays, it issues increasingly draconian demands (e.g., massive resource tithes) on the player's Layer 1 colony. Refusing these demands significantly damages diplomatic relations and risks punitive invasion fleets, but fulfilling them strains the colony's own survival. This creates a powerful tension between loyalty to the decaying center and ruthless independence.

## 2. Dependencies
- Layer 1 Resource System (Stockpiles, Production)
- Layer 3 Faction System (Core World representation, Diplomatic Standing)
- Event System (Demands, Tithing)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::resource::{ResourcePool, ItemType};
    use crate::layer3::diplomacy::{DiplomaticRelation, FactionId};
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_core_world_decay_increases_demand_severity() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<SimulationTime>()
            .init_resource::<Events<CoreWorldDemandEvent>>();

        app.add_systems(Update, update_core_world_decay_system);

        let core_faction_id = app.world_mut().spawn(CoreWorld {
            stability: 100.0,
            decay_rate: 1.0,
        }).id();

        // Simulate some time passing to decrease stability
        for _ in 0..10 {
            app.world_mut().resource_mut::<SimulationTime>().tick += 1;
            app.update();
        }

        let core_world = app.world().get::<CoreWorld>(core_faction_id).unwrap();
        assert!(core_world.stability < 100.0);

        // Severity should be inversely proportional to stability
        let demand_severity = calculate_demand_severity(core_world.stability);
        assert!(demand_severity > 1.0); // Baseline severity
    }

    #[test]
    fn test_fulfilling_demand_depletes_resources_and_maintains_standing() {
        let mut app = App::new();
        // Setup initial resources and diplomatic standing
        app.insert_resource(ResourcePool {
            food: 500,
            alloys: 100,
        });

        let core_faction = app.world_mut().spawn((
            FactionId("Motherland".to_string()),
            DiplomaticRelation { standing: 50.0 },
        )).id();

        app.add_systems(Update, handle_core_world_demands_system);

        // Fulfill the demand
        let demand = CoreWorldDemand {
            resource_type: ItemType::Food,
            amount: 200,
            penalty: 20.0,
            faction: core_faction,
        };

        fulfill_demand(&mut app.world_mut(), demand);

        let pool = app.world().resource::<ResourcePool>();
        assert_eq!(pool.food, 300); // 500 - 200

        let relation = app.world().get::<DiplomaticRelation>(core_faction).unwrap();
        assert_eq!(relation.standing, 50.0); // Standing maintained
    }

    #[test]
    fn test_refusing_demand_decreases_standing() {
        let mut app = App::new();
        let core_faction = app.world_mut().spawn((
            FactionId("Motherland".to_string()),
            DiplomaticRelation { standing: 50.0 },
        )).id();

        let demand = CoreWorldDemand {
            resource_type: ItemType::Food,
            amount: 200,
            penalty: 20.0,
            faction: core_faction,
        };

        refuse_demand(&mut app.world_mut(), demand);

        let relation = app.world().get::<DiplomaticRelation>(core_faction).unwrap();
        assert_eq!(relation.standing, 30.0); // 50.0 - 20.0
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::resource::{ResourcePool, ItemType};
use crate::layer3::diplomacy::{DiplomaticRelation, FactionId};

#[derive(Component)]
pub struct CoreWorld {
    pub stability: f32,
    pub decay_rate: f32,
}

#[derive(Event, Clone)]
pub struct CoreWorldDemandEvent {
    pub demand: CoreWorldDemand,
}

#[derive(Clone)]
pub struct CoreWorldDemand {
    pub resource_type: ItemType,
    pub amount: u32,
    pub penalty: f32,
    pub faction: Entity,
}

pub fn update_core_world_decay_system(mut query: Query<&mut CoreWorld>) {
    for mut core_world in query.iter_mut() {
        core_world.stability -= core_world.decay_rate;
        if core_world.stability < 0.0 {
            core_world.stability = 0.0;
        }
    }
}

pub fn calculate_demand_severity(stability: f32) -> f32 {
    if stability >= 100.0 {
        1.0
    } else {
        1.0 + ((100.0 - stability) / 100.0) * 2.0
    }
}

pub fn fulfill_demand(world: &mut World, demand: CoreWorldDemand) {
    let mut pool = world.resource_mut::<ResourcePool>();
    match demand.resource_type {
        ItemType::Food => {
            if pool.food >= demand.amount {
                pool.food -= demand.amount;
            }
        },
        _ => {}
    }
}

pub fn refuse_demand(world: &mut World, demand: CoreWorldDemand) {
    if let Some(mut relation) = world.get_mut::<DiplomaticRelation>(demand.faction) {
        relation.standing -= demand.penalty;
    }
}

pub fn handle_core_world_demands_system() {
    // Boilerplate for event reading/dispatching in real implementation
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor Event Handling:** `fulfill_demand` and `refuse_demand` shouldn't take `&mut World` directly in the real systems; they should be proper Bevy systems reacting to player choices via events (e.g., `DemandResponseEvent`).
- **Dynamic Demand Generation:** The logic that creates `CoreWorldDemandEvent` should run periodically based on the `CoreWorld`'s decay state. When stability reaches critical thresholds, demands should shift from luxuries to basic survival goods (e.g., Food).
- **Invasion Trigger:** When `DiplomaticRelation` standing drops below a critical negative threshold, a punitive fleet (Loyalist Invasion) should be spawned on Layer 2 to attack the colony.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Core world stability predictably decreases over time.
- [ ] Decreased stability results in larger resource demands.
- [ ] Fulfilling demands correctly deducts from Layer 1 resource pools.
- [ ] Refusing demands correctly penalizes diplomatic standing with the Core World faction.

## 7. Technical Guidance
- **System Placement:** The decay system should ideally run in a simulation tick schedule rather than every frame.
- **UI Integration:** The player must be explicitly prompted with the demand via a UI modal that clearly shows the consequence of refusal (e.g., "-20 Standing").
- **Resource Constraints:** Ensure that `ResourcePool` deduction logic handles edge cases, such as the colony having fewer resources than demanded. If the player clicks "Fulfill" but lacks resources, it should be treated as a refusal or cause a partial penalty (or disable the "Fulfill" button entirely).

## 8. Questions
*Builder: add questions here if spec is unclear.*
