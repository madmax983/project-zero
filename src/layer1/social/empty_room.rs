use crate::layer1::building::Building;
use crate::layer1::clutter::ClutterGrid;
use crate::layer1::items::Item;
use crate::layer1::map::GridPosition;
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;
use rand::Rng;
use std::collections::HashSet;

#[derive(Component)]
pub struct SanctuaryZone {
    pub active: bool,
}

pub fn evaluate_sanctuary_emptiness(
    mut q_zones: Query<(&mut SanctuaryZone, &GridPosition)>,
    q_buildings: Query<&GridPosition, With<Building>>,
    q_items: Query<&GridPosition, With<Item>>,
    clutter_grid: Option<Res<ClutterGrid>>,
) {
    let mut occupied = HashSet::new();
    for pos in q_buildings.iter() {
        occupied.insert((pos.x, pos.y));
    }
    for pos in q_items.iter() {
        occupied.insert((pos.x, pos.y));
    }
    if let Some(clutter) = &clutter_grid {
        for y in 0..clutter.height {
            for x in 0..clutter.width {
                if clutter.get(x, y) > 0.0 {
                    occupied.insert((x as i32, y as i32));
                }
            }
        }
    }

    for (mut sanctuary, pos) in q_zones.iter_mut() {
        sanctuary.active = !occupied.contains(&(pos.x, pos.y));
    }
}

pub fn apply_sanctuary_stress_relief(
    q_zones: Query<(&SanctuaryZone, &GridPosition)>,
    mut q_pops: Query<(&mut StressTracker, &GridPosition)>,
    mut clutter_grid: Option<ResMut<ClutterGrid>>,
) {
    let mut rng = rand::thread_rng();

    for (mut stress, pop_pos) in q_pops.iter_mut() {
        let in_zone = q_zones.iter().any(|(s, p)| s.active && p.x == pop_pos.x && p.y == pop_pos.y);
        if in_zone {
            stress.accumulated_stress = (stress.accumulated_stress - 1.0).max(0.0);

            if rng.gen_bool(0.01) {
                if let Some(ref mut clutter) = clutter_grid {
                    if pop_pos.x >= 0 && pop_pos.y >= 0 {
                        clutter.add_clutter(pop_pos.x as usize, pop_pos.y as usize, 1.0);
                    }
                }
            }
        }
    }
}
