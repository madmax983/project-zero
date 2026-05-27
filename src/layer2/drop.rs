use bevy_ecs::prelude::*;
use crate::layer1::economy::resources::{ResourceItem, ResourceType};
use crate::layer1::core::map::GridPosition;
use crate::layer2::fleet::{Fleet, FleetOrder, InOrbit};
use crate::layer2::mining::FleetCargo;
use rand::prelude::*;

#[derive(Component, Debug)]
pub struct DropPod {
    pub resource_type: ResourceType,
    pub amount: f32,
    pub impact_timer: f32,
    pub target_pos: (i32, i32),
    pub accuracy: f32,
    pub landing_delay: f32,
}

pub fn orbital_drop_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FleetOrder, &InOrbit, &mut FleetCargo), With<Fleet>>,
    colonies: Query<&GridPosition>,
) {
    let mut rng = thread_rng();

    for (entity, order, orbit, mut cargo) in query.iter_mut() {
        if let FleetOrder::Drop(res_type, amount) = *order {
            let mut taken = 0.0;
            if let Some(stack) = cargo.contents.iter_mut().find(|s| s.resource_type == res_type) {
                if stack.amount >= amount {
                    stack.amount -= amount;
                    taken = amount;
                } else {
                    taken = stack.amount;
                    stack.amount = 0.0;
                }
            }

            cargo.contents.retain(|s| s.amount > 0.0);

            if taken > 0.0 {
                // If the orbiting entity is a colony with a GridPosition on layer 1 (rare, usually it's a map),
                // use that, else default to map center.
                let mut center_x = 40;
                let mut center_y = 40;
                if let Ok(pos) = colonies.get(orbit.parent) {
                    center_x = pos.x;
                    center_y = pos.y;
                }

                let scatter = 10;
                let tx = center_x + rng.gen_range(-scatter..=scatter);
                let ty = center_y + rng.gen_range(-scatter..=scatter);

                commands.spawn(DropPod {
                    resource_type: res_type,
                    amount: taken,
                    impact_timer: 10.0,
                    target_pos: (tx, ty),
                    accuracy: 0.5,
                    landing_delay: 10.0,
                });
            }

            commands.entity(entity).remove::<FleetOrder>();
        }
    }
}

pub fn pod_impact_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DropPod)>,
) {
    for (entity, mut pod) in query.iter_mut() {
        // In a real frame-based system, use time.delta_seconds().
        // For this tick-based simulation schedule MVP, 1 tick = 1 decrement is acceptable,
        // but we should ideally use SimulationTime if available.
        pod.impact_timer -= 1.0;

        if pod.impact_timer <= 0.0 {
            commands.spawn((
                ResourceItem {
                    resource_type: pod.resource_type,
                    amount: pod.amount,
                },
                GridPosition {
                    x: pod.target_pos.0,
                    y: pod.target_pos.1,
                },
            ));

            commands.entity(entity).despawn();
        }
    }
}
