use crate::layer1::entities::drone::Drone;
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum IntelligenceLevel {
    Low,
    High,
}

#[derive(Component)]
pub struct DroneBehavior {
    pub intelligence_level: IntelligenceLevel,
}

impl Default for DroneBehavior {
    fn default() -> Self {
        Self {
            intelligence_level: IntelligenceLevel::Low,
        }
    }
}

pub fn update_drone_clusters(
    mut query: Query<(Entity, &GridPosition, &mut DroneBehavior), With<Drone>>,
) {
    let mut drone_positions = Vec::new();
    for (entity, pos, _) in query.iter() {
        drone_positions.push((entity, *pos));
    }

    let clustering_radius = 5.0;

    for (entity, pos, mut behavior) in query.iter_mut() {
        let nearby_count = drone_positions
            .iter()
            .filter(|(e, p)| {
                if *e == entity {
                    return false;
                }
                let dx = p.x as f32 - pos.x as f32;
                let dy = p.y as f32 - pos.y as f32;
                let distance = (dx * dx + dy * dy).sqrt();
                distance < clustering_radius
            })
            .count();

        if nearby_count >= 2 {
            behavior.intelligence_level = IntelligenceLevel::High;
        } else {
            behavior.intelligence_level = IntelligenceLevel::Low;
        }
    }
}

pub fn update_drone_behavior(query: Query<&DroneBehavior, With<Drone>>) {
    // Placeholder for actual behavior execution based on intelligence level
    for behavior in query.iter() {
        match behavior.intelligence_level {
            IntelligenceLevel::High => {
                // Swarm logic
            }
            IntelligenceLevel::Low => {
                // Dumb logic
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_swarm_intelligence_basic_behavior() {
        let mut app = App::new();
        app.add_systems(Update, (update_drone_clusters, update_drone_behavior));

        let drone1 = app
            .world_mut()
            .spawn((
                Drone::default(),
                GridPosition { x: 0, y: 0 },
                DroneBehavior::default(),
            ))
            .id();
        let _drone2 = app
            .world_mut()
            .spawn((
                Drone::default(),
                GridPosition { x: 1, y: 0 },
                DroneBehavior::default(),
            ))
            .id();
        let _drone3 = app
            .world_mut()
            .spawn((
                Drone::default(),
                GridPosition { x: 0, y: 1 },
                DroneBehavior::default(),
            ))
            .id();

        app.update();

        assert_eq!(
            app.world()
                .get::<DroneBehavior>(drone1)
                .unwrap()
                .intelligence_level,
            IntelligenceLevel::High
        );
    }

    #[test]
    fn test_swarm_intelligence_edge_cases() {
        let mut app = App::new();
        app.add_systems(Update, (update_drone_clusters, update_drone_behavior));

        let drone = app
            .world_mut()
            .spawn((
                Drone::default(),
                GridPosition { x: 100, y: 100 },
                DroneBehavior::default(),
            ))
            .id();

        app.update();

        assert_eq!(
            app.world()
                .get::<DroneBehavior>(drone)
                .unwrap()
                .intelligence_level,
            IntelligenceLevel::Low
        );
    }
}
