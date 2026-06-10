//! Biological Stock Market
//
//! Simulates a grim futures market where the stock is the genetic material and labor potential
//! of populations. Investors bet on the lifespan and productivity of entire planetary workforces.

use bevy::prelude::*;
use std::collections::BTreeMap; // Changed HashMap to BTreeMap per REFACTOR phase

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
    pub demands: BTreeMap<GoodType, f32>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, PartialOrd, Ord)]
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
                market
                    .demands
                    .insert(virus.target_good.clone(), virus.potency);
            }
        }
    }
}

/// INT-1062: MarketVirus Demand Generation
pub fn biological_stock_market_bridge(
    mut events: EventReader<crate::layer2::trade::routes::TradeRouteExecutedEvent>,
    mut market: ResMut<crate::layer3::market::GalacticMarket>,
) {
    for event in events.read() {
        if event.item_type == "Food" || event.item_type == "Textiles" {
            let resource = if event.item_type == "Food" {
                crate::layer1::resources::ResourceType::Food
            } else {
                crate::layer1::resources::ResourceType::Waste // Mock mapping for Textiles
            };
            let pool = market.supply_pool.entry(resource).or_insert(0.0);
            let demand_effect = (event.amount as f32) * 0.05;
            *pool = (*pool - demand_effect).max(0.0);
        }
    }
}

// Biological Stock Market Tests
//
// Executable tests for verifying the grim futures market mechanics, including
// virus engineering and demand manipulation.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::*;

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
        let mut query = app.world_mut().query::<&MarketVirus>();
        let virus_count = query.iter(app.world()).count();
        assert_eq!(virus_count, 1, "A market virus should be created");

        let virus = query.single(app.world());
        assert_eq!(virus.target_good, GoodType::FoulRoot);
        assert_eq!(virus.potency, 5.0);
    }

    #[test]
    fn test_inoculate_export_shipment() {
        let mut app = App::new();
        app.add_systems(Update, inoculate_shipment_system);

        let virus = app
            .world_mut()
            .spawn(MarketVirus {
                target_good: GoodType::FoulRoot,
                potency: 5.0,
            })
            .id();

        let shipment = app
            .world_mut()
            .spawn(TradeShipment {
                goods: vec![GoodType::LuxuryTextiles],
                destination: FactionId(1),
                infected_with: None,
            })
            .id();

        app.world_mut().spawn(InoculateCommand { virus, shipment });

        app.update();

        // Assert: shipment is now infected
        let updated_shipment = app.world().get::<TradeShipment>(shipment).unwrap();
        assert!(updated_shipment.infected_with.is_some());
        assert_eq!(
            updated_shipment.infected_with.as_ref().unwrap().target_good,
            GoodType::FoulRoot
        );
        // Virus entity should be consumed/moved
        assert!(app.world().get::<MarketVirus>(virus).is_none());
    }

    #[test]
    fn test_market_demand_manipulation() {
        let mut app = App::new();
        app.add_event::<ShipmentArrivalEvent>();
        app.add_systems(Update, process_infected_imports_system);

        app.world_mut().insert_resource(GalacticMarket {
            demands: vec![(GoodType::FoulRoot, 10.0)].into_iter().collect(),
        });

        let _faction = app.world_mut().spawn(Faction { id: FactionId(1) }).id();

        // Simulate a shipment arriving
        app.world_mut().send_event(ShipmentArrivalEvent {
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
        assert!(
            *new_demand > 10.0,
            "Market demand should increase due to the virus"
        );
        assert_eq!(*new_demand, 25.0, "Demand increases by virus potency");
    }
}
