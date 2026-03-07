# 398: Planetary Migrations

## 1. Overview
Space isn't empty; it's an ecosystem. "Planetary Migrations" introduces biological space-faring entities (like Space Whales or Energy Clouds) that migrate between system nodes based on specific, slow seasonal cycles on Layer 2.

These entities are generally non-hostile but massive. When they cross established player shipping lanes or orbital paths, they can cause logistical disruption or collisions. Dealing with them presents a choice: reroute traffic (inefficient) or cull the herd (causing diplomatic issues with Xeno-Conservationists).

## 2. Dependencies
- `099-fleet-movement.md` (for collision and pathfinding)
- `094-system-view.md` (for Layer 2 mapping)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::movement::{Position, Velocity};
    use crate::layer2::system::SystemNode;

    #[test]
    fn test_space_whale_spawns_and_moves() {
        let mut app = App::new();
        app.add_systems(Update, (spawn_migration_herd_system, update_migration_movement_system));

        app.update(); // Trigger spawn

        let mut query = app.world_mut().query::<(&SpaceWhale, &Velocity)>();
        assert!(query.iter(app.world()).count() > 0, "At least one whale should spawn");

        let (_, velocity) = query.iter(app.world()).next().unwrap();
        assert!(velocity.x != 0.0 || velocity.y != 0.0, "Whale should be moving");
    }

    #[test]
    fn test_whale_collision_with_fleet() {
        let mut app = App::new();
        app.add_systems(Update, check_whale_collisions_system);

        let whale_ent = app.world_mut().spawn((
            SpaceWhale { mass: 1000.0, hostility: 0.0 },
            Position { x: 10.0, y: 10.0 },
        )).id();

        let fleet_ent = app.world_mut().spawn((
            crate::layer2::fleet::Fleet,
            crate::layer2::fleet::FleetHealth { current: 100.0, max: 100.0 },
            Position { x: 10.0, y: 10.0 }, // Exact same position
        )).id();

        app.update();

        let health = app.world().get::<crate::layer2::fleet::FleetHealth>(fleet_ent).unwrap();
        assert!(health.current < 100.0, "Fleet should take damage from collision");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer2::movement::{Position, Velocity};
use crate::layer2::fleet::{Fleet, FleetHealth};

#[derive(Component, Debug)]
pub struct SpaceWhale {
    pub mass: f32,
    pub hostility: f32, // Increases if attacked
}

pub fn spawn_migration_herd_system(
    mut commands: Commands,
    // Add time or event trigger here
) {
    // Mock spawning for MVP
    commands.spawn((
        SpaceWhale { mass: 5000.0, hostility: 0.0 },
        Position { x: 0.0, y: 0.0 },
        Velocity { x: 1.0, y: 1.0 },
    ));
}

pub fn update_migration_movement_system(
    mut query: Query<(&mut Position, &Velocity), With<SpaceWhale>>,
) {
    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.x;
        pos.y += vel.y;
    }
}

pub fn check_whale_collisions_system(
    whale_query: Query<(&SpaceWhale, &Position)>,
    mut fleet_query: Query<(&mut FleetHealth, &Position), With<Fleet>>,
) {
    for (whale, w_pos) in whale_query.iter() {
        for (mut health, f_pos) in fleet_query.iter_mut() {
            // Simple distance check
            let dx = w_pos.x - f_pos.x;
            let dy = w_pos.y - f_pos.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < 2.0 { // Collision radius
                health.current = (health.current - (whale.mass * 0.01)).max(0.0);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathing:** Space Whales should target specific system nodes (like gas giants or asteroid belts) to "graze", not just move in straight lines.
- **Avoidance AI:** Add logic for automated trade fleets to actively attempt to steer around whales, losing time but saving hull integrity.
- **Lore Integration:** Emitting a unique Chronicle event the first time a migration enters the system.

## 6. Acceptance Criteria (Testable!)
- [ ] `SpaceWhale` component exists and functions.
- [ ] Whales move across the Layer 2 map.
- [ ] Fleets take damage when overlapping with a whale's position.
- [ ] Tests pass (including `cargo test`).

## 7. Technical Guidance
- Distance checking in `check_whale_collisions_system` is O(N*M) which is fine for small numbers but might need spatial hashing if fleets/whales scale up significantly.
- Add `SpaceWhale` rendering in the `src/ui/system_map.rs` so the player can actually see them coming.

## 8. Questions
*Builder: add questions here if spec is unclear.*
