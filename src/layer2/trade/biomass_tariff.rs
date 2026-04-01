use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;

#[derive(Event, Debug)]
pub struct TradeDeal {
    pub offered_crops: f32,
    pub offered_pops: usize,
    pub requested_resin: f32,
}

pub fn process_biomass_tariff_system(
    mut events: EventReader<TradeDeal>,
    mut resources: ResMut<ColonyResources>,
    mut commands: Commands,
    mut pops: Query<(Entity, &mut StressTracker), With<Pop>>,
) {
    for deal in events.read() {
        if deal.offered_crops > 0.0 && deal.offered_pops == 0 {
            if resources.food >= deal.offered_crops {
                resources.food -= deal.offered_crops;
                resources.wood += deal.requested_resin; // Using wood as bio-resin
            }
        } else if deal.offered_pops > 0 && deal.offered_crops == 0.0 {
            let mut traded = 0;
            for (entity, mut stress) in pops.iter_mut() {
                if traded < deal.offered_pops {
                    commands.entity(entity).despawn();
                    traded += 1;
                } else {
                    stress.accumulated_stress = (stress.accumulated_stress + 40.0).min(100.0);
                }
            }

            if traded > 0 {
                resources.wood += deal.requested_resin; // Using wood as bio-resin
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{process_biomass_tariff_system, TradeDeal};
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::stress::StressTracker;
    use bevy_ecs::event::Events;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_pay_biomass_tariff_with_crops() {
        let mut world = World::new();
        // Setup stash with crops
        let mut stash = ColonyResources::default();
        stash.food = 500.0;
        stash.wood = 0.0; // The alien currency (using wood for now)
        world.insert_resource(stash);
        world.init_resource::<Events<TradeDeal>>();

        // Send a trade deal proposing crops for resin
        world.send_event(TradeDeal {
            offered_crops: 500.0,
            offered_pops: 0,
            requested_resin: 100.0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_biomass_tariff_system);
        schedule.run(&mut world);

        let final_stash = world.resource::<ColonyResources>();
        assert_eq!(final_stash.food, 0.0, "Crops consumed for tariff");
        assert_eq!(final_stash.wood, 100.0, "Bio-resin received");
    }

    #[test]
    fn test_pay_biomass_tariff_with_pops_damages_sanity() {
        let mut world = World::new();
        // A single desperate pop to trade
        let pop_to_trade = world
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 10.0,
                },
            ))
            .id();

        // Stash to receive the reward
        let mut stash = ColonyResources::default();
        stash.wood = 0.0; // The alien currency
        world.insert_resource(stash);
        world.init_resource::<Events<TradeDeal>>();

        world.send_event(TradeDeal {
            offered_crops: 0.0,
            offered_pops: 1,         // Desperate times
            requested_resin: 1000.0, // Worth a lot more
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_biomass_tariff_system);
        schedule.run(&mut world);

        let bystander = world
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 10.0,
                },
            ))
            .id();
        world.send_event(TradeDeal {
            offered_crops: 0.0,
            offered_pops: 0,
            requested_resin: 0.0,
        });
        schedule.run(&mut world);

        // Pop should be despawned (traded)
        assert!(
            world.get_entity(pop_to_trade).is_err(),
            "Traded pop is gone"
        );

        let bystander_stress = world.get::<StressTracker>(bystander).unwrap();
        assert!(
            bystander_stress.accumulated_stress <= 10.0,
            "Stress should be maintained in second run"
        );

        // Final sanity check
        let final_stash = world.resource::<ColonyResources>();
        assert_eq!(final_stash.wood, 1000.0, "Bio-resin received");
    }
}
