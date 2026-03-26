# Spec 629: Atmospheric Ignition

## 1. Overview
Atmospheric Ignition introduces a dangerous environmental hazard linked to advanced industry. Highly productive but "dirty" late-game buildings emit "Volatile Vapors." These vapors pool in low-altitude terrain tiles. They do not naturally harm Pops, but if any "spark" event (e.g., weapon fire, electrical short, lightning) occurs within a tile containing these vapors, the entire contiguous vapor cloud detonates in a massive "Air-Burst" explosion, destroying nearby structures and incinerating Pops. This mechanic forces players to consider ventilation, zoning, and strict safety measures around high-yield industrial sectors.

## 2. Dependencies
- **Layer 1 Grid/Fluids** (Atmosphere grid, vapor pooling)
- **Layer 1 Environmental Events** (Sparks, explosions, structural damage)
- **Layer 1 Buildings** (Industrial emission mechanics)

## 3. RED Phase: Tests First

```rust
// specs/629-atmospheric-ignition.md

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use std::collections::HashSet;

    // Mock components and resources
    #[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
    struct GridPosition {
        x: i32,
        y: i32,
    }

    #[derive(Component)]
    struct VolatileVapor {
        concentration: f32,
    }

    #[derive(Event)]
    struct SparkEvent {
        position: GridPosition,
    }

    #[derive(Event)]
    struct ExplosionEvent {
        position: GridPosition,
        damage: f32,
    }

    #[test]
    fn test_spark_ignites_vapor_and_creates_explosion() {
        // Arrange
        let mut app = App::new();
        app.add_event::<SparkEvent>();
        app.add_event::<ExplosionEvent>();
        app.add_systems(Update, process_ignition);

        let pos = GridPosition { x: 5, y: 5 };
        app.world.spawn((pos, VolatileVapor { concentration: 10.0 }));

        // Act
        app.world.send_event(SparkEvent { position: pos });
        app.update();

        // Assert
        let explosion_events = app.world.resource::<Events<ExplosionEvent>>();
        let mut reader = explosion_events.get_reader();
        let explosions: Vec<_> = reader.read(explosion_events).collect();

        assert_eq!(explosions.len(), 1, "An explosion should have been triggered");
        assert_eq!(explosions[0].position, pos, "Explosion should occur at the spark position");
        assert!(explosions[0].damage > 0.0, "Explosion should deal damage");

        // Vapor should be consumed
        let vapor_count = app.world.query::<&VolatileVapor>().iter(&app.world).count();
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

        app.world.spawn((pos1, VolatileVapor { concentration: 10.0 }));
        app.world.spawn((pos2, VolatileVapor { concentration: 10.0 }));
        app.world.spawn((pos3, VolatileVapor { concentration: 10.0 }));

        // Act
        app.world.send_event(SparkEvent { position: pos1 });
        app.update();

        // Assert
        let explosion_events = app.world.resource::<Events<ExplosionEvent>>();
        let mut reader = explosion_events.get_reader();
        let explosions: Vec<_> = reader.read(explosion_events).collect();

        assert_eq!(explosions.len(), 2, "Explosions should occur at both contiguous tiles");

        let explosion_positions: HashSet<_> = explosions.iter().map(|e| e.position).collect();
        assert!(explosion_positions.contains(&pos1));
        assert!(explosion_positions.contains(&pos2));
        assert!(!explosion_positions.contains(&pos3), "Isolated vapor should not ignite");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct VolatileVapor {
    pub concentration: f32,
}

#[derive(Event)]
pub struct SparkEvent {
    pub position: GridPosition,
}

#[derive(Event)]
pub struct ExplosionEvent {
    pub position: GridPosition,
    pub damage: f32,
}

pub fn process_ignition(
    mut commands: Commands,
    mut sparks: EventReader<SparkEvent>,
    mut explosions: EventWriter<ExplosionEvent>,
    vapor_query: Query<(Entity, &GridPosition, &VolatileVapor)>,
) {
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
```

## 5. REFACTOR Phase: Quality & Design
- **Grid Integration**: This minimal implementation uses isolated entities for grid tiles. Integration should map this to the actual `AtmosphereGrid` or `FluidGrid` data structures for efficiency.
- **Spark Sources**: Ensure that combat systems (weapons), damaged power grids, and lightning weather events properly emit `SparkEvent` instances.
- **Visuals and Audio**: Add systems responding to `ExplosionEvent` to spawn fire particles and play explosion sound effects.
- **Chronicle Integration**: If an explosion destroys buildings or kills Pops, log an event to the Chronicle indicating an industrial disaster.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Spark events within volatile vapor trigger an explosion.
- [ ] Explosions consume the vapor.
- [ ] Explosions accurately chain to adjacent/contiguous vapor tiles but not isolated ones.

## 7. Technical Guidance
- Flood-filling across the Bevy ECS can be slow if querying every entity. If `AtmosphereGrid` is a single large array/resource, perform the flood fill within that resource directly.
- Ensure that vapor only pools in low-altitude tiles, which will require interaction with `TerrainGrid` elevation data during the vapor emission/diffusion step (not shown here, but part of the broader feature).

## 8. Questions
*Builder: add questions here if spec is unclear.*
