# Geodetic Sentience

## 1. Overview
**Layer:** 1
**Fantasy:** I swear that rock was on the left side of the room yesterday.
**Mechanic:** Rare "Living Stone" items found in deep mines. They are Resources, but they slowly move towards heat sources or other stones when unobserved. If enough gather, they fuse into a Golem.
**Emergence:** You stockpile the stones in the warehouse. Overnight, they migrate to the reactor room and wall in the door. The night shift engineer is trapped.

## 2. Dependencies
- Resource and Inventory system
- Spatial querying / Heat maps
- Observation system (if items are being watched)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_living_stone_moves_when_unobserved() {
        let mut app = App::new();
        app.add_systems(Update, living_stone_movement_system);

        let entity = app.world_mut().spawn((
            LivingStone,
            Position { x: 0.0, y: 0.0 },
            TargetHeatSource { x: 10.0, y: 0.0 },
            Unobserved, // Simulating lack of observation
        )).id();

        app.update();

        let pos = app.world().get::<Position>(entity).unwrap();
        assert!(pos.x > 0.0, "Living stone should move towards heat source when unobserved");
    }

    #[test]
    fn test_living_stone_stays_still_when_observed() {
        let mut app = App::new();
        app.add_systems(Update, living_stone_movement_system);

        let entity = app.world_mut().spawn((
            LivingStone,
            Position { x: 0.0, y: 0.0 },
            TargetHeatSource { x: 10.0, y: 0.0 },
            // Not Unobserved!
        )).id();

        app.update();

        let pos = app.world().get::<Position>(entity).unwrap();
        assert_eq!(pos.x, 0.0, "Living stone should NOT move when observed");
    }

    #[test]
    fn test_living_stones_fuse_into_golem() {
        let mut app = App::new();
        app.add_event::<GolemFusionEvent>();
        app.add_systems(Update, living_stone_fusion_system);

        // Spawn enough stones at the same position
        for _ in 0..5 {
            app.world_mut().spawn((LivingStone, Position { x: 5.0, y: 5.0 }));
        }

        app.update();

        let events = app.world().resource::<Events<GolemFusionEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.len(&events), 1, "Should fuse into a Golem when clustered");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct LivingStone;

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct TargetHeatSource {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Unobserved;

#[derive(Event)]
pub struct GolemFusionEvent {
    pub position: Position,
}

pub fn living_stone_movement_system(
    mut query: Query<(&mut Position, &TargetHeatSource), (With<LivingStone>, With<Unobserved>)>,
) {
    for (mut pos, target) in query.iter_mut() {
        // Simple step towards target
        if pos.x < target.x { pos.x += 1.0; }
        if pos.x > target.x { pos.x -= 1.0; }
        if pos.y < target.y { pos.y += 1.0; }
        if pos.y > target.y { pos.y -= 1.0; }
    }
}

pub fn living_stone_fusion_system(
    query: Query<(Entity, &Position), With<LivingStone>>,
    mut events: EventWriter<GolemFusionEvent>,
    mut commands: Commands,
) {
    // Highly naive O(n^2) or grouped approach for minimal pass
    // For GREEN phase, let's just count them roughly
    let mut grid: std::collections::HashMap<(i32, i32), Vec<Entity>> = std::collections::HashMap::new();

    for (entity, pos) in query.iter() {
        let key = (pos.x as i32, pos.y as i32);
        grid.entry(key).or_default().push(entity);
    }

    for (key, entities) in grid.iter() {
        if entities.len() >= 5 {
            events.send(GolemFusionEvent {
                position: Position { x: key.0 as f32, y: key.1 as f32 },
            });
            // Despawn the stones to complete fusion
            for e in entities {
                commands.entity(*e).despawn();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Optimize the fusion check using spatial hashing.
- Only calculate target heat source if one isn't currently assigned or if a closer one appears.
- Ensure observation mechanics integrate properly with colony line-of-sight systems.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: unobserved stones move, clustering triggers Golem fusion.

## 7. Technical Guidance
- Make sure `Unobserved` is added/removed accurately by the vision/line-of-sight system.

## 8. Questions
*Builder: add questions here if spec is unclear.*
