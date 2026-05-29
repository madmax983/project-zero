use crate::layer1::economy::resources::ColonyResources;
use bevy::prelude::*;

#[derive(Component)]
pub struct FossilizedShip {
    pub decay_rate: f32,
    pub maintenance_cost: u32,
    pub defense_bonus: u32,
    pub structural_integrity: f32,
}

pub fn fossilized_ship_decay_system(
    time: Res<Time>,
    mut query: Query<&mut FossilizedShip>,
    mut resources: ResMut<ColonyResources>,
) {
    let dt = time.delta_secs();

    for mut ship in query.iter_mut() {
        if ship.structural_integrity <= 0.0 {
            continue;
        }

        let maintenance_cost_f32 = ship.maintenance_cost as f32;
        let cost = ColonyResources {
            scrap: maintenance_cost_f32 * dt,
            ..Default::default()
        };

        if !resources.try_deduct(&cost) {
            ship.structural_integrity -= ship.decay_rate * dt;
        }
    }
}
