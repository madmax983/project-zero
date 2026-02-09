use crate::layer1::building::{Building, BuildingType};
use crate::layer1::chronicle::{Chronicle, EventImportance};
use crate::layer1::resources::{ColonyResources, ResourceType};
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Component marker for the Trade Depot building.
#[derive(Component, Default)]
pub struct TradeDepot;

/// Represents a single trade offer from a merchant.
#[derive(Debug, Clone)]
pub struct TradeDeal {
    /// The resource required to purchase.
    pub cost_resource: ResourceType,
    /// The amount required.
    pub cost_amount: f32,
    /// The resource given in return.
    pub give_resource: ResourceType,
    /// The amount given.
    pub give_amount: f32,
}

/// A visiting merchant with a set of deals.
#[derive(Debug, Clone)]
pub struct Merchant {
    /// The merchant's name.
    pub name: String,
    /// The simulation tick when they arrived.
    pub arrival_tick: u64,
    /// The simulation tick when they will depart.
    pub departure_tick: u64,
    /// The deals they offer.
    pub deals: Vec<TradeDeal>,
}

/// Tracks the state of merchant visits.
#[derive(Resource, Default)]
pub struct MerchantState {
    /// The currently present merchant, if any.
    pub active_merchant: Option<Merchant>,
    /// The tick until which no new merchant can arrive.
    pub cooldown: u64,
}

/// System to handle merchant arrivals and departures.
///
/// Checks if a Trade Depot exists and if the cooldown has expired.
/// If so, spawns a new merchant with random deals.
/// Also handles departure of active merchants.
#[allow(clippy::collapsible_if)]
pub fn merchant_arrival_system(world: &mut World) {
    let current_tick = world.resource::<SimulationTime>().tick;

    // Check if depot exists
    let depot_exists = world
        .query::<(&Building, &TradeDepot)>()
        .iter(world)
        .any(|(b, _)| b.building_type == BuildingType::TradeDepot);

    // 1. Departure Check
    // We scope this to release the borrow on MerchantState before logging
    let mut event_to_log: Option<(String, EventImportance)> = None;

    {
        let mut state = world.resource_mut::<MerchantState>();

        let mut should_depart = false;
        if let Some(merchant) = &state.active_merchant {
            if current_tick >= merchant.departure_tick {
                should_depart = true;
            }
        }

        if should_depart {
            if let Some(merchant) = state.active_merchant.take() {
                let name = merchant.name;
                event_to_log = Some((
                    format!("Merchant {name} has departed."),
                    EventImportance::Standard,
                ));
            }
            state.cooldown = current_tick + 2000;
        } else if state.active_merchant.is_none() && current_tick >= state.cooldown && depot_exists {
            // Arrival Check
            let mut rng = rand::thread_rng();

            // Randomize merchant name
            let names = ["Wanderer", "Caravan", "Trader", "Merchant", "Peddler"];
            let name = format!(
                "{} {}",
                names[rng.gen_range(0..names.len())],
                rng.gen_range(100..999)
            );

            // Generate deals
            let mut deals = Vec::new();
            for _ in 0..rng.gen_range(1..4) {
                // Determine cost/give types (simplified)
                // In a real system, we'd check what the colony lacks.
                let cost_type = match rng.gen_range(0..3) {
                    0 => ResourceType::Wood,
                    1 => ResourceType::Stone,
                    _ => ResourceType::Food,
                };

                let give_type = match rng.gen_range(0..3) {
                    0 => ResourceType::Metal,
                    1 => ResourceType::Planks,
                    _ => ResourceType::Ore,
                };

                deals.push(TradeDeal {
                    cost_resource: cost_type,
                    cost_amount: rng.gen_range(5.0f32..20.0f32).round(),
                    give_resource: give_type,
                    give_amount: rng.gen_range(2.0f32..10.0f32).round(),
                });
            }

            let merchant = Merchant {
                name: name.clone(),
                arrival_tick: current_tick,
                departure_tick: current_tick + 500,
                deals,
            };

            event_to_log = Some((
                format!("Merchant {name} has arrived."),
                EventImportance::Major,
            ));
            state.active_merchant = Some(merchant);
        }
    }

    // Log events if any occurred
    if let Some((msg, importance)) = event_to_log {
        if let Some(mut chronicle) = world.get_resource_mut::<Chronicle>() {
            chronicle.add_event(current_tick, msg, importance);
        }
    }
}

/// Executes a trade deal, exchanging resources if affordable.
///
/// Returns `true` if the trade was successful.
pub fn execute_trade(world: &mut World, deal: &TradeDeal) -> bool {
    let mut resources = world.resource_mut::<ColonyResources>();

    // Check affordability
    let affordable = match deal.cost_resource {
        ResourceType::Wood => resources.wood >= deal.cost_amount,
        ResourceType::Stone => resources.stone >= deal.cost_amount,
        ResourceType::Food => resources.food >= deal.cost_amount,
        ResourceType::Metal => resources.metal >= deal.cost_amount,
        ResourceType::Ore => resources.ore >= deal.cost_amount,
        ResourceType::Planks => resources.planks >= deal.cost_amount,
        ResourceType::Blocks => resources.blocks >= deal.cost_amount,
        ResourceType::Waste => resources.waste >= deal.cost_amount,
    };

    if !affordable {
        return false;
    }

    // Deduct
    match deal.cost_resource {
        ResourceType::Wood => resources.wood -= deal.cost_amount,
        ResourceType::Stone => resources.stone -= deal.cost_amount,
        ResourceType::Food => resources.food -= deal.cost_amount,
        ResourceType::Metal => resources.metal -= deal.cost_amount,
        ResourceType::Ore => resources.ore -= deal.cost_amount,
        ResourceType::Planks => resources.planks -= deal.cost_amount,
        ResourceType::Blocks => resources.blocks -= deal.cost_amount,
        ResourceType::Waste => resources.waste -= deal.cost_amount,
    }

    // Add
    match deal.give_resource {
        ResourceType::Wood => resources.add_wood(deal.give_amount),
        ResourceType::Stone => resources.add_stone(deal.give_amount),
        ResourceType::Food => resources.add_food(deal.give_amount),
        ResourceType::Metal => resources.add_metal(deal.give_amount),
        ResourceType::Ore => resources.add_ore(deal.give_amount),
        ResourceType::Planks => resources.add_planks(deal.give_amount),
        ResourceType::Blocks => resources.add_blocks(deal.give_amount),
        ResourceType::Waste => resources.add_waste(deal.give_amount),
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::chronicle::Chronicle;
    use crate::layer1::resources::{ColonyResources, ResourceType};
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_trade_depot_component_exists() {
        let depot = TradeDepot::default();
        let _ = depot;
    }

    #[test]
    fn test_merchant_state_resource_defaults() {
        let state = MerchantState::default();
        assert!(state.active_merchant.is_none());
        assert!(state.cooldown == 0); // Default is 0
    }

    #[test]
    fn test_merchant_arrival_requires_depot() {
        let mut world = World::new();
        world.insert_resource(MerchantState::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(Chronicle::default());
        // No TradeDepot spawned

        merchant_arrival_system(&mut world);

        let state = world.resource::<MerchantState>();
        assert!(state.active_merchant.is_none());
    }

    #[test]
    fn test_merchant_arrival_with_depot() {
        let mut world = World::new();
        // Set cooldown to 0 to force arrival check
        world.insert_resource(MerchantState {
            cooldown: 0,
            ..Default::default()
        });
        world.insert_resource(SimulationTime {
            tick: 100,
            ..Default::default()
        });
        world.insert_resource(Chronicle::default());

        // Spawn Trade Depot
        world.spawn((
            Building {
                building_type: BuildingType::TradeDepot,
            },
            TradeDepot::default(),
        ));

        merchant_arrival_system(&mut world);

        let state = world.resource::<MerchantState>();
        assert!(state.active_merchant.is_some());

        let merchant = state.active_merchant.as_ref().unwrap();
        assert!(merchant.departure_tick > 100);
        assert!(!merchant.deals.is_empty());
    }

    #[test]
    fn test_merchant_departs_after_time() {
        let mut world = World::new();
        // Setup state with active merchant
        let merchant = Merchant {
            name: "Leaver".to_string(),
            arrival_tick: 0,
            departure_tick: 100,
            deals: vec![],
        };
        world.insert_resource(MerchantState {
            active_merchant: Some(merchant),
            cooldown: 0,
        });

        // Advance time past departure
        world.insert_resource(SimulationTime { tick: 101, ..Default::default() });
        world.insert_resource(Chronicle::default());

        // Depot exists
        world.spawn((
            Building { building_type: BuildingType::TradeDepot },
            TradeDepot::default(),
        ));

        merchant_arrival_system(&mut world);

        let state = world.resource::<MerchantState>();
        assert!(state.active_merchant.is_none());
        assert!(state.cooldown > 101);

        // Check log
        let chronicle = world.resource::<Chronicle>();
        assert!(!chronicle.events.is_empty());
        assert!(chronicle.events[0].text.contains("has departed"));
    }

    #[test]
    fn test_execute_trade_success() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.wood = 50.0;
        resources.metal = 0.0;
        world.insert_resource(resources);

        let deal = TradeDeal {
            cost_resource: ResourceType::Wood,
            cost_amount: 10.0,
            give_resource: ResourceType::Metal,
            give_amount: 5.0,
        };

        let success = execute_trade(&mut world, &deal);

        assert!(success);
        let res = world.resource::<ColonyResources>();
        assert!((res.wood - 40.0).abs() < f32::EPSILON);
        assert!((res.metal - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_execute_trade_insufficient_funds() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.wood = 5.0; // Need 10
        world.insert_resource(resources);

        let deal = TradeDeal {
            cost_resource: ResourceType::Wood,
            cost_amount: 10.0,
            give_resource: ResourceType::Metal,
            give_amount: 5.0,
        };

        let success = execute_trade(&mut world, &deal);

        assert!(!success);
        let res = world.resource::<ColonyResources>();
        assert!((res.wood - 5.0).abs() < f32::EPSILON);
        assert!((res.metal - 0.0).abs() < f32::EPSILON);
    }
}
