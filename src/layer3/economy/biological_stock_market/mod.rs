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

#[cfg(test)]
pub mod tests;
