use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

use crate::layer1::map::GridPosition;

/// Represents volatile vapor pooling in a tile.
#[derive(Component)]
pub struct VolatileVapor {
    pub concentration: f32,
}

/// Event fired when a spark occurs, potentially igniting vapor.
#[derive(Event)]
pub struct SparkEvent {
    pub position: GridPosition,
}

/// Event fired when an atmospheric explosion happens.
#[derive(Event)]
pub struct ExplosionEvent {
    pub position: GridPosition,
    pub damage: f32,
}

/// System that processes sparks and creates explosion events via flood-fill.
pub fn process_ignition(
    mut commands: Commands,
    mut sparks: EventReader<SparkEvent>,
    mut explosions: EventWriter<ExplosionEvent>,
    vapor_query: Query<(Entity, &GridPosition, &VolatileVapor)>,
) {
    if sparks.is_empty() {
        return;
    }

    let mut to_ignite: HashSet<GridPosition> = HashSet::new();

    // Map grid positions to entities for easy lookup
    let mut vapor_map = std::collections::HashMap::new();
    for (entity, pos, vapor) in vapor_query.iter() {
        vapor_map.insert(*pos, (entity, vapor.concentration));
    }

    for spark in sparks.read() {
        if to_ignite.contains(&spark.position) { continue; }

        // Simple Flood Fill for contiguous vapor
        let mut queue = VecDeque::new();
        if vapor_map.contains_key(&spark.position) {
            queue.push_back(spark.position);
            to_ignite.insert(spark.position);
        }

        while let Some(current_pos) = queue.pop_front() {
            // Trigger explosion
            let (entity, concentration) = vapor_map.get(&current_pos).unwrap();
            explosions.send(ExplosionEvent {
                position: current_pos,
                damage: *concentration * 5.0, // Arbitrary scaling
            });
            commands.entity(*entity).despawn(); // Consume vapor

            // Check adjacent cells
            let adjacent = [
                GridPosition { x: current_pos.x + 1, y: current_pos.y },
                GridPosition { x: current_pos.x - 1, y: current_pos.y },
                GridPosition { x: current_pos.x, y: current_pos.y + 1 },
                GridPosition { x: current_pos.x, y: current_pos.y - 1 },
            ];

            for adj in adjacent {
                if vapor_map.contains_key(&adj) && !to_ignite.contains(&adj) {
                    to_ignite.insert(adj);
                    queue.push_back(adj);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spark_ignites_vapor_and_creates_explosion() {
        // Arrange
        let mut app = App::new();
        app.add_event::<SparkEvent>();
        app.add_event::<ExplosionEvent>();
        app.add_systems(Update, process_ignition);

        let pos = GridPosition { x: 5, y: 5 };
        app.world_mut().spawn((pos, VolatileVapor { concentration: 10.0 }));

        // Act
        app.world_mut().send_event(SparkEvent { position: pos });
        app.update();

        // Assert
        let explosion_events = app.world().resource::<Events<ExplosionEvent>>();
        let mut reader = explosion_events.get_cursor();
        let explosions: Vec<_> = reader.read(explosion_events).collect();

        assert_eq!(explosions.len(), 1, "An explosion should have been triggered");
        assert_eq!(explosions[0].position, pos, "Explosion should occur at the spark position");
        assert!(explosions[0].damage > 0.0, "Explosion should deal damage");

        // Vapor should be consumed
        let vapor_count = app.world_mut().query::<&VolatileVapor>().iter(app.world()).count();
        assert_eq!(vapor_count, 0, "Ignited vapor should be consumed/despawned");
    }

    #[test]
    fn test_spark_causes_chain_reaction_in_contiguous_vapor() {
        // Arrange
        let mut app = App::new();
        app.add_event::<SparkEvent>();
        app.add_event::<ExplosionEvent>();
        app.add_systems(Update, process_ignition);

        let pos1 = GridPosition { x: 5, y: 5 };
        let pos2 = GridPosition { x: 6, y: 5 }; // Contiguous to pos1
        let pos3 = GridPosition { x: 10, y: 10 }; // Isolated

        app.world_mut().spawn((pos1, VolatileVapor { concentration: 10.0 }));
        app.world_mut().spawn((pos2, VolatileVapor { concentration: 10.0 }));
        app.world_mut().spawn((pos3, VolatileVapor { concentration: 10.0 }));

        // Act
        app.world_mut().send_event(SparkEvent { position: pos1 });
        app.update();

        // Assert
        let explosion_events = app.world().resource::<Events<ExplosionEvent>>();
        let mut reader = explosion_events.get_cursor();
        let explosions: Vec<_> = reader.read(explosion_events).collect();

        assert_eq!(explosions.len(), 2, "Explosions should occur at both contiguous tiles");

        let explosion_positions: HashSet<_> = explosions.iter().map(|e| e.position).collect();
        assert!(explosion_positions.contains(&pos1));
        assert!(explosion_positions.contains(&pos2));
        assert!(!explosion_positions.contains(&pos3), "Isolated vapor should not ignite");
    }
}
