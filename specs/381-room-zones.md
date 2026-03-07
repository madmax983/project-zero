# 381: Room Zones

## 1. Overview
**Layer:** 1
**Fantasy:** A bedroom is a sanctuary, not just a bed in a field.
**Mechanic:** Players designate "Zones" (Bedroom, Dining, Hospital) over enclosed areas. Room Value = Size + Furniture Quality + Wall Material. Pops gain mood buffs from high-value rooms.
**Emergence:** Pops get jealous of the Governor's suite. A "Barracks" saves space but causes stress due to lack of privacy.
**Tension:** Build compact, efficient cells (sadness) or sprawling mansions (space/resource cost)?

## 2. Dependencies
- Room/Enclosure detection (from existing building placement and grid systems).
- Pop `Needs` (for mood buffs/debuffs based on room value and privacy).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::Needs;
    use bevy::prelude::*;

    #[test]
    fn test_room_zone_evaluation() {
        // Arrange
        let mut app = App::new();
        app.add_event::<RoomEvaluatedEvent>();
        app.add_systems(Update, evaluate_rooms_system);

        // Create a room entity
        let room_id = app.world_mut().spawn((
            RoomZone {
                zone_type: ZoneType::Bedroom,
                tiles: vec![(0, 0), (0, 1), (1, 0), (1, 1)], // Size 4
                base_value: 0.0,
            },
        )).id();

        // Add furniture
        app.world_mut().spawn((
            Furniture { quality: 10.0 },
            Position { x: 0, y: 0 },
            InRoom(room_id),
        ));

        // Act
        app.update();

        // Assert
        let room = app.world().get::<RoomZone>(room_id).unwrap();
        // Size (4) + Furniture (10) = 14
        assert_eq!(room.value(), 14.0);
    }

    #[test]
    fn test_pop_room_mood_buff() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_room_mood_system);

        let room_id = app.world_mut().spawn((
            RoomZone {
                zone_type: ZoneType::Bedroom,
                tiles: vec![(0, 0)],
                base_value: 50.0, // High value
            },
        )).id();

        // Pop assigned to this room
        let pop_id = app.world_mut().spawn((
            Needs {
                leisure: 50.0,
                rest: 50.0,
                hunger: 50.0,
                hygiene: 50.0,
            },
            AssignedRoom(room_id),
        )).id();

        // Act
        app.update();

        // Assert
        let needs = app.world().get::<Needs>(pop_id).unwrap();
        // Mood buff applied to leisure or rest needs
        assert!(needs.rest > 50.0);
    }

    #[test]
    fn test_barracks_privacy_penalty() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_room_mood_system);

        let room_id = app.world_mut().spawn((
            RoomZone {
                zone_type: ZoneType::Bedroom, // Can act as barracks if multiple pops
                tiles: vec![(0, 0), (0, 1)],
                base_value: 10.0,
            },
            OccupantCount(3), // Overcrowded
        )).id();

        let pop_id = app.world_mut().spawn((
            Needs {
                leisure: 50.0,
                rest: 50.0,
                hunger: 50.0,
                hygiene: 50.0,
            },
            AssignedRoom(room_id),
        )).id();

        // Act
        app.update();

        // Assert
        let needs = app.world().get::<Needs>(pop_id).unwrap();
        // Overcrowded barracks leads to stress/leisure penalty
        assert!(needs.leisure < 50.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::needs::Needs;

#[derive(Component)]
pub struct RoomZone {
    pub zone_type: ZoneType,
    pub tiles: Vec<(i32, i32)>,
    pub base_value: f32,
}

impl RoomZone {
    pub fn value(&self) -> f32 {
        self.base_value + self.tiles.len() as f32
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    Bedroom,
    Dining,
    Hospital,
}

#[derive(Component)]
pub struct Furniture {
    pub quality: f32,
}

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct InRoom(pub Entity);

#[derive(Component)]
pub struct AssignedRoom(pub Entity);

#[derive(Component)]
pub struct OccupantCount(pub usize);

#[derive(Event)]
pub struct RoomEvaluatedEvent(pub Entity);

pub fn evaluate_rooms_system(
    mut rooms: Query<(Entity, &mut RoomZone)>,
    furniture: Query<(&Furniture, &InRoom)>,
    mut events: EventWriter<RoomEvaluatedEvent>,
) {
    for (room_entity, mut room) in rooms.iter_mut() {
        let mut total_furniture_quality = 0.0;
        for (furn, in_room) in furniture.iter() {
            if in_room.0 == room_entity {
                total_furniture_quality += furn.quality;
            }
        }
        room.base_value = total_furniture_quality;
        events.send(RoomEvaluatedEvent(room_entity));
    }
}

pub fn apply_room_mood_system(
    mut pops: Query<(&mut Needs, &AssignedRoom)>,
    rooms: Query<(&RoomZone, Option<&OccupantCount>)>,
) {
    for (mut needs, assigned_room) in pops.iter_mut() {
        if let Ok((room, occupant_count)) = rooms.get(assigned_room.0) {
            let val = room.value();
            if val > 20.0 {
                needs.rest += 5.0; // High value buff
            }

            if let Some(count) = occupant_count {
                if count.0 > 2 {
                    needs.leisure -= 5.0; // Overcrowded debuff
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:** Cache room values instead of calculating them on the fly if there are many rooms. Use a spatial grid to quickly link furniture to rooms.
- **Code Smells:** The `OccupantCount` should be dynamically calculated from the `AssignedRoom` queries rather than explicitly stored as a component, unless cached.
- **Performance:** `evaluate_rooms_system` should only run when furniture or walls change, not every frame.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops gain mood (Needs) buffs from high-value rooms and debuffs from overcrowded rooms.

## 7. Technical Guidance
- Integrate with `Layer1SystemSet::Observation` or a specific slow-tick timer for mood application to prevent rapid need fluctuations.
- Ensure the wall materials are properly tracked in the room evaluation logic later.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
