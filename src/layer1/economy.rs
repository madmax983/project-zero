use bevy_ecs::prelude::*;
use crate::layer1::pop::Job;
use crate::layer1::actions::AssignmentType;
use crate::layer1::resources::ColonyResources;
use crate::layer1::needs::Needs;

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Wallet {
    pub credits: f32,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct ColonyPrices {
    pub food_price: f32,
    pub luxury_price: f32,
}

impl Default for ColonyPrices {
    fn default() -> Self {
        Self {
            food_price: 1.0,
            luxury_price: 5.0,
        }
    }
}

pub fn get_wage_for_job(job_type: AssignmentType) -> f32 {
    match job_type {
        AssignmentType::Miner => 2.0,
        AssignmentType::FarmWorker => 1.5,
        AssignmentType::Hauler => 1.0,
        AssignmentType::LibraryWorker => 3.0, // Research pays well
        AssignmentType::Engineer => 2.5,
        AssignmentType::Doctor => 3.0,
        AssignmentType::Governor => 5.0,
        _ => 1.0,
    }
}

pub fn pay_wage(world: &mut World, worker: Entity, amount: f32) {
    if let Some(mut wallet) = world.get_mut::<Wallet>(worker) {
        wallet.credits += amount;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;

    /// Helper system for economy tests.
    fn consume_food_with_payment_system(world: &mut World) {
        let price = world.resource::<ColonyPrices>().food_price;
        let food_avail = world.resource::<ColonyResources>().food;
        if food_avail < 1.0 {
            return;
        }

        let mut query = world.query::<(Entity, &mut Needs, &mut Wallet)>();
        let mut transactions = Vec::new();

        for (entity, needs, wallet) in query.iter(world) {
            if needs.hunger < 0.7 && wallet.credits >= price {
                transactions.push(entity);
            }
        }

        for entity in transactions {
            let mut food_res = world.resource_mut::<ColonyResources>();
            if food_res.food >= 1.0 {
                food_res.food -= 1.0;
            } else {
                break;
            }

            if let Some(mut wallet) = world.get_mut::<Wallet>(entity) {
                wallet.credits -= price;
            }
            if let Some(mut needs) = world.get_mut::<Needs>(entity) {
                needs.hunger = (needs.hunger + 0.3).min(1.0);
            }
        }
    }

    #[test]
    fn test_wallet_default() {
        let wallet = Wallet::default();
        assert_eq!(wallet.credits, 0.0);
    }

    #[test]
    fn test_pay_wages_system() {
        let mut world = World::new();
        // Spawn a pop with a wallet
        let pop = world.spawn((Pop, Wallet { credits: 10.0 })).id();

        // Simulate job completion event
        pay_wage(&mut world, pop, 5.0);

        let wallet = world.get::<Wallet>(pop).unwrap();
        assert_eq!(wallet.credits, 15.0);
    }

    #[test]
    fn test_purchase_food_success() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Wallet { credits: 10.0 },
            Needs { hunger: 0.5, rest: 0.5, ..Default::default() }
        )).id();

        world.insert_resource(ColonyPrices { food_price: 2.0, ..Default::default() });
        world.insert_resource(ColonyResources { food: 10.0, ..Default::default() });

        // Call modified consumption system
        consume_food_with_payment_system(&mut world);

        let wallet = world.get::<Wallet>(pop).unwrap();
        assert_eq!(wallet.credits, 8.0, "Should deduct 2.0 credits");

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.hunger > 0.5, "Should have eaten");
    }

    #[test]
    fn test_purchase_food_fail_poverty() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Wallet { credits: 1.0 }, // Not enough!
            Needs { hunger: 0.1, rest: 0.5, ..Default::default() } // Starving
        )).id();

        world.insert_resource(ColonyPrices { food_price: 2.0, ..Default::default() });
        world.insert_resource(ColonyResources { food: 10.0, ..Default::default() });

        consume_food_with_payment_system(&mut world);

        let wallet = world.get::<Wallet>(pop).unwrap();
        assert_eq!(wallet.credits, 1.0, "Should not deduct if purchase failed");

        let needs = world.get::<Needs>(pop).unwrap();
        assert_eq!(needs.hunger, 0.1, "Should NOT have eaten (too poor)");
    }

    #[test]
    fn test_job_wage_configuration() {
        // Ensure jobs have a wage associated
        let job = AssignmentType::Miner;
        assert!(get_wage_for_job(job) > 0.0);
    }
}
