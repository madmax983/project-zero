# 821: The Sub-Space Whispers

## 1. Overview
Layer 2 ships utilizing high-speed sub-space drives accumulate "Whisper Counters." If a fleet accumulates too many whispers without docking at a stabilized core world, the fleet initiates an emergency, uncontrollable jump into deep space, becoming "Lost." This mechanic creates tension between the strategic advantage of instantaneous, rapid fleet response and the creeping risk of your armada descending into madness and abandoning you.

## 2. Dependencies
- `src/layer2/fleet.rs` or similar Layer 2 implementation.
- `src/layer2/map.rs` or system mapping logic to determine if a world is "stabilized".

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_fleet_accumulates_whispers_on_sub_space_jump() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, process_sub_space_jumps);
        app.add_event::<SubSpaceJumpEvent>();

        let fleet = app.world_mut().spawn((
            Fleet { ..default() },
            WhisperCounter { count: 0 },
        )).id();

        // Act
        app.world_mut().send_event(SubSpaceJumpEvent { fleet_entity: fleet });
        app.update();

        // Assert
        let whispers = app.world().get::<WhisperCounter>(fleet).unwrap();
        assert_eq!(whispers.count, 1);
    }

    #[test]
    fn test_fleet_becomes_lost_at_max_whispers() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, check_whisper_madness);

        let fleet = app.world_mut().spawn((
            Fleet { ..default() },
            WhisperCounter { count: 10 }, // Assuming 10 is max
            Position { x: 5, y: 5 },
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<LostInDeepSpace>(fleet).is_some());
    }

    #[test]
    fn test_docking_at_stabilized_world_clears_whispers() {
        // Test that a fleet with >0 whispers returning to a stabilized core world resets to 0.
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct WhisperCounter {
    pub count: u32,
}

#[derive(Component)]
pub struct LostInDeepSpace;

pub struct SubSpaceJumpEvent {
    pub fleet_entity: Entity,
}

pub fn process_sub_space_jumps(
    mut events: EventReader<SubSpaceJumpEvent>,
    mut query: Query<&mut WhisperCounter>,
) {
    for event in events.read() {
        if let Ok(mut whispers) = query.get_mut(event.fleet_entity) {
            whispers.count += 1;
        }
    }
}

pub fn check_whisper_madness(
    mut commands: Commands,
    query: Query<(Entity, &WhisperCounter), Without<LostInDeepSpace>>,
) {
    for (entity, whispers) in query.iter() {
        if whispers.count >= 10 { // Threshold
            commands.entity(entity).insert(LostInDeepSpace);
            // Optionally: despawn or move to a random deep space coordinate
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create a `WhisperMadnessEvent` when a fleet crosses the threshold, so the UI and Chronicle systems can log the event.
- Expose the maximum whisper threshold as a configuration constant or resource to allow tech upgrades to increase it.
- Determine exactly what `LostInDeepSpace` entails (e.g., entity despawn, moved to unreachable coords, or turned hostile).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Jumping increases whisper count, and exceeding the threshold correctly applies the Lost condition.
- [ ] Docking correctly clears whispers.

## 7. Technical Guidance
- Consider the interaction with existing movement systems. Ensure `WhisperCounter` is only incremented on actual FTL / sub-space jumps, not normal sublight travel.
- Clearing whispers upon docking might require a system checking `ArrivalEvent` or a localized system state.

## 8. Questions
*Builder: add questions here if spec is unclear.*
