# Spec 1062: The Biological Stock Market

## 1. Overview
The Biological Stock Market introduces a cross-layer economic sabotage mechanic where Layer 1 bio-labs engineer non-lethal "Market Viruses." These viruses alter Layer 3 empire Pops' preferences when secretly attached to Layer 2 trade exports, shifting galactic demand to artificially inflate the value of specific goods (like local roots or textiles) the player colony produces.

## 2. Dependencies
- Layer 1 Bio-lab infrastructure or biological research mechanics.
- Layer 2 trade and export systems (`TradeRoute` or equivalent export mechanism).
- Layer 3 market demand and price fluctuation mechanics (`MarketDemand` or equivalent).

## 3. RED Phase: Tests First

```rust
// src/layer3/economy/biological_stock_market/tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_market_virus_creation() {
        let mut app = App::new();
        // Setup initial state
        app.add_systems(Update, engineer_market_virus_system);

        let biolab = app.world_mut().spawn(BioLab { efficiency: 1.0 }).id();

        // Act: trigger engineering of a virus targeting "Foul Root"
        app.world_mut().spawn(EngineerVirusCommand {
            source_biolab: biolab,
            target_good: GoodType::FoulRoot,
            potency: 5.0,
        });

        app.update();

        // Assert: virus is created in inventory/storage
        let virus_count = app.world().query::<&MarketVirus>().iter(&app.world()).count();
        assert_eq!(virus_count, 1, "A market virus should be created");

        let virus = app.world().query::<&MarketVirus>().single(&app.world());
        assert_eq!(virus.target_good, GoodType::FoulRoot);
        assert_eq!(virus.potency, 5.0);
    }

    #[test]
    fn test_inoculate_export_shipment() {
        let mut app = App::new();
        app.add_systems(Update, inoculate_shipment_system);

        let virus = app.world_mut().spawn(MarketVirus {
            target_good: GoodType::FoulRoot,
            potency: 5.0,
        }).id();

        let shipment = app.world_mut().spawn(TradeShipment {
            goods: vec![GoodType::LuxuryTextiles],
            destination: FactionId(1),
            infected_with: None,
        }).id();

        app.world_mut().spawn(InoculateCommand {
            virus,
            shipment,
        });

        app.update();

        // Assert: shipment is now infected
        let updated_shipment = app.world().get::<TradeShipment>(shipment).unwrap();
        assert!(updated_shipment.infected_with.is_some());
        assert_eq!(updated_shipment.infected_with.unwrap().target_good, GoodType::FoulRoot);
        // Virus entity should be consumed/moved
        assert!(app.world().get::<MarketVirus>(virus).is_none());
    }

    #[test]
    fn test_market_demand_manipulation() {
        let mut app = App::new();
        app.add_systems(Update, process_infected_imports_system);

        app.world_mut().insert_resource(GalacticMarket {
            demands: vec![(GoodType::FoulRoot, 10.0)].into_iter().collect(),
        });

        let faction = app.world_mut().spawn(Faction { id: FactionId(1) }).id();

        // Simulate a shipment arriving
        app.world_mut().spawn(ShipmentArrivalEvent {
            shipment: TradeShipment {
                goods: vec![GoodType::LuxuryTextiles],
                destination: FactionId(1),
                infected_with: Some(MarketVirus {
                    target_good: GoodType::FoulRoot,
                    potency: 15.0,
                }),
            },
        });

        app.update();

        // Assert: Market demand for target good has increased
        let market = app.world().resource::<GalacticMarket>();
        let new_demand = market.demands.get(&GoodType::FoulRoot).unwrap();
        assert!(*new_demand > 10.0, "Market demand should increase due to the virus");
        assert_eq!(*new_demand, 25.0, "Demand increases by virus potency");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer3/economy/biological_stock_market/mod.rs

use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq, Clone)]
pub struct MarketVirus {
    pub target_good: GoodType,
    pub potency: f32,
}

#[derive(Component)]
pub struct BioLab {
    pub efficiency: f32,
}

#[derive(Component)]
pub struct EngineerVirusCommand {
    pub source_biolab: Entity,
    pub target_good: GoodType,
    pub potency: f32,
}

#[derive(Component)]
pub struct InoculateCommand {
    pub virus: Entity,
    pub shipment: Entity,
}

#[derive(Component, Clone)]
pub struct TradeShipment {
    pub goods: Vec<GoodType>,
    pub destination: FactionId,
    pub infected_with: Option<MarketVirus>,
}

#[derive(Event)]
pub struct ShipmentArrivalEvent {
    pub shipment: TradeShipment,
}

#[derive(Resource)]
pub struct GalacticMarket {
    pub demands: HashMap<GoodType, f32>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum GoodType {
    FoulRoot,
    LuxuryTextiles,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct FactionId(pub u32);

#[derive(Component)]
pub struct Faction {
    pub id: FactionId,
}

pub fn engineer_market_virus_system(
    mut commands: Commands,
    query: Query<(Entity, &EngineerVirusCommand)>,
) {
    for (entity, cmd) in query.iter() {
        commands.spawn(MarketVirus {
            target_good: cmd.target_good.clone(),
            potency: cmd.potency,
        });
        commands.entity(entity).despawn();
    }
}

pub fn inoculate_shipment_system(
    mut commands: Commands,
    mut shipments: Query<&mut TradeShipment>,
    viruses: Query<&MarketVirus>,
    cmds: Query<(Entity, &InoculateCommand)>,
) {
    for (cmd_entity, cmd) in cmds.iter() {
        if let Ok(virus) = viruses.get(cmd.virus) {
            if let Ok(mut shipment) = shipments.get_mut(cmd.shipment) {
                shipment.infected_with = Some(virus.clone());
                commands.entity(cmd.virus).despawn();
                commands.entity(cmd_entity).despawn();
            }
        }
    }
}

pub fn process_infected_imports_system(
    mut events: EventReader<ShipmentArrivalEvent>,
    mut market: ResMut<GalacticMarket>,
) {
    for event in events.read() {
        if let Some(virus) = &event.shipment.infected_with {
            if let Some(demand) = market.demands.get_mut(&virus.target_good) {
                *demand += virus.potency;
            } else {
                market.demands.insert(virus.target_good.clone(), virus.potency);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Use `BTreeMap` or `IndexMap` instead of `HashMap` in `GalacticMarket` to ensure deterministic simulation state when iterating over market demands.
- The `TradeShipment` component should probably link to a global `MarketVirus` registry rather than cloning the component to save memory if many shipments are infected.
- Add a decay mechanic for the virus-induced demand, so the economic spike isn't permanent, reflecting the empire eventually curing the virus.
- Extract the specific virus crafting logic into its own system, taking resource costs (like biological samples) into account.
- Add detection risk mechanics: if the virus is discovered, massive diplomatic penalties with the destination Faction should occur.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] BioLabs can successfully spawn `MarketVirus` entities via command.
- [ ] `MarketVirus` can be attached to `TradeShipment`s.
- [ ] Infected `TradeShipment`s arriving at destinations correctly increment market demand for the target good.

## 7. Technical Guidance
- Ensure `process_infected_imports_system` acts securely. If multiple shipments arrive, the demands should increase cumulatively.
- `MarketVirus` should be treated as a specialized resource. In an integrated scenario, a `Cargo` inventory system might hold the virus instead of keeping it as a free-floating entity before inoculation.
- Make sure to clear the `ShipmentArrivalEvent` buffer in `src/layer1/systems/cleanup.rs` to prevent memory leaks if this is handled in Layer 1 systems.

## 8. Questions
*Builder: add questions here if spec is unclear.*
