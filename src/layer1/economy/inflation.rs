use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct MarketCrashEvent {
    pub new_multiplier: f32,
}

#[derive(Resource)]
pub struct MarketState {
    pub credit_value_multiplier: f32,
}

#[derive(Component)]
pub struct EmpireResources {
    pub credits: f32,
    pub alloys: u32,
}

#[derive(Event)]
pub struct BarterRequest {
    pub initiator: Entity,
    pub target: Entity,
    pub offer_alloys: u32,
    pub request_alloys: u32,
}

pub fn trigger_market_crash(
    mut market: ResMut<MarketState>,
    mut events: EventReader<MarketCrashEvent>,
) {
    for event in events.read() {
        market.credit_value_multiplier = event.new_multiplier;
    }
}

pub fn process_barter_trade(
    mut query: Query<&mut EmpireResources>,
    mut barter_events: EventReader<BarterRequest>,
) {
    for trade in barter_events.read() {
        if let Ok([mut init_res, mut target_res]) = query.get_many_mut([trade.initiator, trade.target]) {
            // Check if both parties have enough alloys
            if init_res.alloys >= trade.offer_alloys && target_res.alloys >= trade.request_alloys {
                // Execute the barter
                init_res.alloys -= trade.offer_alloys;
                init_res.alloys += trade.request_alloys;

                target_res.alloys -= trade.request_alloys;
                target_res.alloys += trade.offer_alloys;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_market_crash_reduces_credit_value() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(MarketState {
            credit_value_multiplier: 1.0, // 1 Credit = 1 Unit of purchasing power
        });

        world.insert_resource(Events::<MarketCrashEvent>::default());
        world.send_event(MarketCrashEvent {
            new_multiplier: 0.1,
        });

        // Act: Trigger a market crash
        let _ = world.run_system_once(trigger_market_crash);

        // Assert: Credit value multiplier should drop significantly
        let market = world.resource::<MarketState>();
        assert_eq!(market.credit_value_multiplier, 0.1); // Value plummeted
    }

    #[test]
    fn test_barter_trade_bypasses_credit_value() {
        // Arrange
        let mut world = World::new();
        // A crashed market where credits are worthless
        world.insert_resource(MarketState {
            credit_value_multiplier: 0.1,
        });

        world.insert_resource(Events::<BarterRequest>::default());

        // Setup two entities (empires/traders) to perform a barter
        let empire_a = world.spawn(EmpireResources {
            credits: 1_000_000.0,
            alloys: 100,
        }).id();

        let empire_b = world.spawn(EmpireResources {
            credits: 10.0,
            alloys: 500,
        }).id();

        // Let's say Empire A wants 100 Alloys from Empire B, using barter (100 Alloys for 100 Alloys)
        world.send_event(BarterRequest {
            initiator: empire_a,
            target: empire_b,
            offer_alloys: 100,
            request_alloys: 100,
        });

        // Act: Process the barter
        let _ = world.run_system_once(process_barter_trade);

        // Assert: Goods are exchanged regardless of credit value
        let a_res = world.get::<EmpireResources>(empire_a).unwrap();
        let b_res = world.get::<EmpireResources>(empire_b).unwrap();

        assert_eq!(a_res.alloys, 100); // 100 - 100 + 100 = 100
        assert_eq!(b_res.alloys, 500); // 500 - 100 + 100 = 500
    }
}
