use crate::layer1::energy::Battery;
use crate::layer1::map::GridPosition;
use crate::layer1::mind::utility_types::{ActionType, PopAction};
use bevy_ecs::prelude::*;

pub fn sleepwalking_sabotage_system(
    pops: Query<(&PopAction, &GridPosition)>,
    mut batteries: Query<(&mut Battery, &GridPosition)>,
) {
    let drain_amount = 10.0;
    for (action, pop_pos) in &pops {
        if action.current == ActionType::Sleepwalking {
            for (mut battery, bat_pos) in &mut batteries {
                if pop_pos.x == bat_pos.x && pop_pos.y == bat_pos.y {
                    battery.charge = (battery.charge - drain_amount).max(0.0);
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(sleepwalking_sabotage_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_sleepwalking_sabotage_discharges_battery() {
        let mut world = World::new();

        world.spawn((
            PopAction {
                current: ActionType::Sleepwalking,
                current_utility: 100.0,
                ticks_committed: 10,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let battery = world
            .spawn((
                Battery {
                    capacity: 100.0,
                    charge: 50.0,
                    max_throughput: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world.run_system_once(sleepwalking_sabotage_system).unwrap();

        let b = world.get::<Battery>(battery).unwrap();
        assert!(
            b.charge < 50.0,
            "Battery charge should have been drained by the sleepwalking pop"
        );
    }
}
