use crate::layer1::energy::PowerConsumer;
use crate::layer1::map::GridPosition;
use crate::layer1::stress::StressTracker;
use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct PhantomGridCapable {
    pub connection_chance: f32,
}

#[derive(Component)]
pub struct PhantomGridConnected;

pub fn phantom_grid_connection_system(
    mut commands: Commands,
    mut query: Query<
        (Entity, &mut PowerConsumer, &PhantomGridCapable),
        Without<PhantomGridConnected>,
    >,
) {
    let mut rng = rand::thread_rng();
    for (entity, mut consumer, capable) in query.iter_mut() {
        if !consumer.active && consumer.demand > 0.0 && rng.gen::<f32>() < capable.connection_chance
        {
            consumer.active = true;
            commands.entity(entity).insert(PhantomGridConnected);
        }
    }
}

pub fn phantom_grid_hum_system(
    time: Res<Time>,
    buildings: Query<&GridPosition, With<PhantomGridConnected>>,
    mut pops: Query<(&GridPosition, &mut StressTracker)>,
) {
    let hum_radius = 5;
    let stress_increase_rate = 1.0;

    for (pop_pos, mut tracker) in pops.iter_mut() {
        for b_pos in buildings.iter() {
            let dx = pop_pos.x - b_pos.x;
            let dy = pop_pos.y - b_pos.y;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq <= hum_radius * hum_radius {
                tracker.accumulated_stress += stress_increase_rate * time.delta_secs();
                // Only take stress from one building per tick for simplicity
                break;
            }
        }
    }
}

pub fn phantom_grid_disconnection_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut PowerConsumer), With<PhantomGridConnected>>,
) {
    for (entity, mut consumer) in query.iter_mut() {
        if consumer.active {
            // Normal power is active, so disconnect from phantom grid
            commands.entity(entity).remove::<PhantomGridConnected>();
        } else {
            // Still no normal power, but we are connected. We should provide power this tick.
            consumer.active = true;
        }
    }
}
