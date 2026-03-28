# Specification: The Kinetic Sleds

## 1. Overview
**Layer:** 1
**Title:** The Kinetic Sleds
**Description:** Reinventing logistics by ignoring friction. "Hover Sleds" move items almost instantly across straight flat terrain but cannot stop or turn quickly. If a sled path is blocked by a sudden obstacle (like a pop walking across the route), the sled crashes, dealing massive kinetic damage to whatever it hits and scattering the cargo.

## 2. Dependencies
- Tile/Grid Spatial system (`Position`, Grid Resources)
- Logistics/Hauling system (Item dropoff/pickup)
- Movement system (Straight-line trajectory calculation)
- Collision/Damage system

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_sled_moves_in_straight_line() {
        let mut app = App::new();
        app.add_systems(Update, process_sled_movement);

        // Arrange
        let sled = app.world_mut().spawn((
            Position { x: 0, y: 0 },
            HoverSled { velocity: Vec2::new(5.0, 0.0), cargo_weight: 100 },
        )).id();

        // Act
        app.world_mut().insert_resource(Time::new(std::time::Duration::from_secs(1)));
        app.update();

        // Assert
        let pos = app.world().get::<Position>(sled).unwrap();
        assert_eq!(pos.x, 5, "Sled should move 5 tiles right in 1 second");
        assert_eq!(pos.y, 0, "Sled should not deviate from straight line");
    }

    #[test]
    fn test_sled_collides_with_obstacle_causing_damage_and_scatter() {
        let mut app = App::new();
        app.add_event::<SledCrashEvent>();
        app.add_systems(Update, (process_sled_movement, handle_sled_crashes));

        // Arrange
        let obstacle = app.world_mut().spawn((
            Position { x: 5, y: 0 },
            Pop { health: 100 }, // Using Pop as a physical obstacle
        )).id();

        let sled = app.world_mut().spawn((
            Position { x: 4, y: 0 },
            HoverSled { velocity: Vec2::new(2.0, 0.0), cargo_weight: 500 },
        )).id();

        // Act: Sled moves from 4 to 6, crossing the obstacle at 5
        app.world_mut().insert_resource(Time::new(std::time::Duration::from_secs(1)));
        app.update();

        // Assert
        let events = app.world().resource::<Events<SledCrashEvent>>();
        let mut reader = events.get_cursor();
        let crashes: Vec<_> = reader.read(events).collect();

        assert_eq!(crashes.len(), 1, "Sled should trigger a crash event upon collision");
        assert_eq!(crashes[0].sled, sled);
        assert_eq!(crashes[0].obstacle, obstacle);
        assert!(crashes[0].kinetic_damage > 100, "Damage should be massive based on velocity and weight");

        let pop = app.world().get::<Pop>(obstacle).unwrap();
        assert!(pop.health <= 0, "Obstacle should take massive kinetic damage");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct HoverSled {
    pub velocity: Vec2,
    pub cargo_weight: i32,
}

#[derive(Component)]
pub struct Pop {
    pub health: i32,
}

#[derive(Event)]
pub struct SledCrashEvent {
    pub sled: Entity,
    pub obstacle: Entity,
    pub kinetic_damage: i32,
}

pub fn process_sled_movement(
    time: Option<Res<Time>>,
    mut sled_query: Query<(Entity, &mut Position, &HoverSled)>,
    obstacle_query: Query<(Entity, &Position), (With<Pop>, Without<HoverSled>)>,
    mut crash_events: EventWriter<SledCrashEvent>,
) {
    let dt = time.map(|t| t.delta_seconds()).unwrap_or(1.0);

    // Map obstacles
    let mut grid: HashMap<(i32, i32), Entity> = HashMap::new();
    for (entity, pos) in obstacle_query.iter() {
        grid.insert((pos.x, pos.y), entity);
    }

    for (sled_entity, mut pos, sled) in sled_query.iter_mut() {
        let dx = (sled.velocity.x * dt).round() as i32;
        let dy = (sled.velocity.y * dt).round() as i32;

        // Raycast check along the movement path
        // (Simplified for minimal implementation: just checks destination)
        let target_x = pos.x + dx;
        let target_y = pos.y + dy;

        if let Some(&obstacle_entity) = grid.get(&(target_x, target_y)) {
            let damage = (sled.velocity.length() * sled.cargo_weight as f32) as i32;
            crash_events.send(SledCrashEvent {
                sled: sled_entity,
                obstacle: obstacle_entity,
                kinetic_damage: damage,
            });
            // Sled stops on crash
            pos.x = target_x;
            pos.y = target_y;
        } else {
            pos.x = target_x;
            pos.y = target_y;
        }
    }
}

pub fn handle_sled_crashes(
    mut events: EventReader<SledCrashEvent>,
    mut pops: Query<&mut Pop>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok(mut pop) = pops.get_mut(event.obstacle) {
            pop.health -= event.kinetic_damage;
            // Despawn sled or scatter items
            commands.entity(event.sled).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Movement Path:** `process_sled_movement` currently only checks the destination tile. It needs proper raycasting or stepping to check every tile along the movement path in a single frame to prevent "tunneling" through obstacles at high speeds.
- **Scattering Cargo:** Upon a crash, the `cargo` carried by the sled should be instantiated as dropped items in a radius around the crash site.
- **Routing:** Pops should avoid walking on "Sled Paths" if alternative routes exist to minimize accidental deaths. Implement a pathfinding cost penalty for designated sled routes.

## 6. Acceptance Criteria (Testable!)
- [ ] `test_sled_moves_in_straight_line` passes.
- [ ] `test_sled_collides_with_obstacle_causing_damage_and_scatter` passes.
- [ ] High-speed sleds don't tunnel through obstacles.
- [ ] Sled cargo is properly scattered on collision.
- [ ] Test coverage ≥85%.

## 7. Technical Guidance
- Implement in `src/layer1/logistics/hover_sled.rs`.
- Ensure `HoverSled` respects the global terrain (cannot pass through walls).
- Connect the `SledCrashEvent` to the notification/chronicle system to log "Industrial Accident: Sled Crash".

## 8. Questions
*Builder: add questions here if spec is unclear.*
