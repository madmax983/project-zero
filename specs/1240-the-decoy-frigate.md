# The Decoy Frigate

## 1. Overview
**Layer:** 2
**Fantasy:** Winning a war through deception rather than firepower, making the enemy chase ghosts.
**Mechanic:** An unarmed, fragile ship equipped with advanced electronic warfare suites. It can perfectly spoof the sensor signature of any other ship class (e.g., a dreadnought or a colony ship). It draws enemy fire and diverts their fleets, but if scanned at close range, the illusion shatters.

## 2. Dependencies
- Layer 2 Combat Targeting System
- Fleet Sensor System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_decoy_spoofs_target_priority() {
        // Arrange
        let mut app = App::new();
        app.world_mut().spawn((Ship, TargetPriority(10))); // Normal ship
        let decoy = app.world_mut().spawn((DecoyFrigate { spoofed_priority: 100 }, TargetPriority(1))).id();

        // Act
        app.add_systems(Update, apply_decoy_spoofing_system);
        app.update();

        // Assert
        let priority = app.world().get::<TargetPriority>(decoy).unwrap();
        assert_eq!(priority.0, 100, "Decoy should assume the spoofed priority level");
    }

    #[test]
    fn test_decoy_illusion_shatters_on_close_scan() {
        // Arrange
        let mut app = App::new();
        app.add_event::<CloseRangeScanEvent>();

        let decoy = app.world_mut().spawn((
            DecoyFrigate { spoofed_priority: 100 },
            TargetPriority(100),
        )).id();

        app.world_mut().send_event(CloseRangeScanEvent { target: decoy });

        // Act
        app.add_systems(Update, resolve_close_scans_system);
        app.update();

        // Assert
        assert!(app.world().get::<DecoyFrigate>(decoy).is_none(), "Decoy component should be removed on discovery");
        let priority = app.world().get::<TargetPriority>(decoy).unwrap();
        assert_eq!(priority.0, 1, "Target priority should drop to minimum after discovery");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct TargetPriority(pub u32);

#[derive(Component)]
pub struct DecoyFrigate {
    pub spoofed_priority: u32,
}

#[derive(Event)]
pub struct CloseRangeScanEvent {
    pub target: Entity,
}

pub fn apply_decoy_spoofing_system(
    mut query: Query<(&DecoyFrigate, &mut TargetPriority)>,
) {
    for (decoy, mut priority) in query.iter_mut() {
        priority.0 = decoy.spoofed_priority;
    }
}

pub fn resolve_close_scans_system(
    mut events: EventReader<CloseRangeScanEvent>,
    mut commands: Commands,
    mut query: Query<&mut TargetPriority, With<DecoyFrigate>>,
) {
    for ev in events.read() {
        if let Ok(mut priority) = query.get_mut(ev.target) {
            // Illusion shattered
            commands.entity(ev.target).remove::<DecoyFrigate>();
            priority.0 = 1; // Drop to lowest priority since it's just an unarmed decoy
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Visual spoofing: The ship should adopt the visual mesh/icon of the spoofed class until discovered.
- Distance calculations: `CloseRangeScanEvent` should be triggered automatically when enemy ships enter a certain radius.
- Friendly fire/Neutral scans: Neutrals might accidentally scan it and broadcast the truth to enemies.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Decoy assumes target priority of larger ships to draw fire.
- [ ] Close range scans shatter the illusion, dropping priority.

## 7. Technical Guidance
- **Targeting AI:** Ensure the enemy AI actually respects the `TargetPriority` component when choosing whom to shoot at.
- **Visuals:** Add a system that synchronizes the Decoy's render state with its `spoofed_priority` or class.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
