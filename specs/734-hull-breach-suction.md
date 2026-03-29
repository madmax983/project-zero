# 734 - Hull Breach Suction

## 1. Overview
The terrifying physics of a vacuum. When a pressurized room is breached to a vacuum, air rushes out. Pops and loose items (resources) are pulled towards the breach. Small items are lost to space. Pops take impact damage or are ejected. This tension relies on strict safety protocols vs. fast construction.

## 2. Dependencies
- Grid map with air pressure/vacuum states.
- Pathfinding/Positioning for Pops.
- Spatial queries for items.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_hull_breach_pulls_pops() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add map with breach at (10, 10)
        let breach_pos = Vec2::new(10.0, 10.0);

        let pop_pos = Vec2::new(8.0, 10.0);
        let pop_entity = app.world_mut().spawn((
            Pop,
            Transform::from_translation(pop_pos.extend(0.0)),
        )).id();

        // Act
        // Process hull breach logic
        // ... (system that applies suction force towards breach_pos)
        app.update();

        // Assert
        let transform = app.world().get::<Transform>(pop_entity).unwrap();
        // The pop should have moved closer to the breach
        assert!(transform.translation.x > 8.0);
    }

    #[test]
    fn test_hull_breach_ejects_items() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add item right next to breach
        let item_entity = app.world_mut().spawn((
            Item { weight: 1.0 },
            Transform::from_translation(Vec3::new(9.9, 10.0, 0.0)),
        )).id();

        // Act
        // Process ejection logic
        app.update();

        // Assert
        // Item should be despawned (ejected to space)
        assert!(app.world().get_entity(item_entity).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Item {
    pub weight: f32,
}

#[derive(Component)]
pub struct HullBreach {
    pub position: Vec2,
    pub suction_power: f32,
}

pub fn apply_suction(
    breaches: Query<&HullBreach>,
    mut pops: Query<&mut Transform, With<Pop>>,
    mut items: Query<(Entity, &mut Transform, &Item), Without<Pop>>,
    mut commands: Commands,
) {
    for breach in breaches.iter() {
        // Pull pops
        for mut transform in pops.iter_mut() {
            let pos = transform.translation.truncate();
            let dist = pos.distance(breach.position);
            if dist < breach.suction_power && dist > 0.1 {
                let dir = (breach.position - pos).normalize();
                transform.translation += (dir * 1.0).extend(0.0); // Basic pull
            }
        }

        // Pull items
        for (entity, mut transform, item) in items.iter_mut() {
            let pos = transform.translation.truncate();
            let dist = pos.distance(breach.position);

            // If item is very close to breach, eject it
            if dist < 0.5 && item.weight < 10.0 {
                commands.entity(entity).despawn();
            } else if dist < breach.suction_power {
                let dir = (breach.position - pos).normalize();
                transform.translation += (dir * 2.0 / item.weight).extend(0.0);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Implement proper physics integration (like `bevy_rapier` or internal rigidbodies) for the suction instead of modifying `Transform` directly.
- Ensure suction respects walls/doors so pops in sealed rooms next to a breach don't get pulled.
- Introduce an `EjectedEvent` instead of immediately despawning, so the game log can report what was lost.

## 6. Acceptance Criteria (Testable!)
- [ ] Hull breach pulls pops and items toward it based on distance.
- [ ] Items near the breach are ejected (despawned or event fired).
- [ ] `cargo test` returns 0 failures and coverage is 85%+.
- [ ] `cargo clippy -- -D warnings` passes.

## 7. Technical Guidance
- When modifying walls to build or expand, spawn a temporary `HullBreach` if the room isn't depressurized first.
- Provide a `Depressurize` command for rooms to allow safe construction.

## 8. Questions
*Builder: add questions here if spec is unclear.*
