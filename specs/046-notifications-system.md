# 046: Notifications System

## Overview

The Chronicle system records history, but the player needs real-time, actionable feedback about immediate events. This spec introduces a **Notifications System** to queue and display temporary alerts (e.g., "Building Complete", "Low Food", "Raid Incoming").

Key features:
- **Notification**: A struct with `text`, `severity`, `expiration`, and `action_link`.
- **NotificationQueue**: A resource managing active notifications.
- **Expiration**: Auto-dismissal of low-priority alerts.
- **Severity**: Distinct levels (Info, Success, Warning, Error) for UI styling.

## Dependencies

- `003` — UI Layout (for future display)
- `010` — Chronicle System (optional integration)

## RED Phase: Tests First

Write these tests in `src/layer1/notifications_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::notifications::{
        Notification, NotificationQueue, NotificationSeverity, notification_expiration_system
    };
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_notification_queue_resource_exists() {
        let queue = NotificationQueue::default();
        assert!(queue.active.is_empty());
    }

    #[test]
    fn test_add_notification() {
        let mut queue = NotificationQueue::default();
        let id = queue.add("Test Alert".to_string(), NotificationSeverity::Info, 100);

        assert_eq!(queue.active.len(), 1);
        assert_eq!(queue.active[0].id, id);
        assert_eq!(queue.active[0].text, "Test Alert");
        assert_eq!(queue.active[0].severity, NotificationSeverity::Info);
    }

    #[test]
    fn test_notification_expiration() {
        let mut world = World::new();
        world.insert_resource(NotificationQueue::default());
        world.insert_resource(SimulationTime { tick: 100, ..Default::default() });

        // Add notification that expires at tick 150
        let mut queue = world.resource_mut::<NotificationQueue>();
        queue.add_with_expiration("Expires soon".to_string(), NotificationSeverity::Info, 100, 150);

        // Run system at tick 100 (should stay)
        notification_expiration_system(&mut world);
        assert_eq!(world.resource::<NotificationQueue>().active.len(), 1);

        // Advance time to 151
        world.resource_mut::<SimulationTime>().tick = 151;
        notification_expiration_system(&mut world);
        assert!(world.resource::<NotificationQueue>().active.is_empty());
    }

    #[test]
    fn test_notification_limit() {
        let mut queue = NotificationQueue::default();
        // Add more than MAX (e.g., 10)
        for i in 0..15 {
            queue.add(format!("Alert {}", i), NotificationSeverity::Info, 0);
        }

        // Should clamp to 10 most recent? Or reject?
        // Spec decision: Keep most recent.
        assert!(queue.active.len() <= 10);
        assert_eq!(queue.active.last().unwrap().text, "Alert 14");
    }

    #[test]
    fn test_dismiss_notification() {
        let mut queue = NotificationQueue::default();
        let id = queue.add("To Dismiss".to_string(), NotificationSeverity::Info, 0);

        assert_eq!(queue.active.len(), 1);
        queue.dismiss(id);
        assert!(queue.active.is_empty());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Create `src/layer1/notifications.rs`

```rust
use bevy_ecs::prelude::*;
use crate::shared::time::SimulationTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationSeverity {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u64,
    pub text: String,
    pub severity: NotificationSeverity,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub action_link: Option<Entity>, // Optional click target
}

#[derive(Resource, Default)]
pub struct NotificationQueue {
    pub active: Vec<Notification>,
    next_id: u64,
}

impl NotificationQueue {
    const MAX_NOTIFICATIONS: usize = 10;

    pub fn add(&mut self, text: String, severity: NotificationSeverity, current_tick: u64) -> u64 {
        // Default expiration: 300 ticks (30s at 10TPS)
        self.add_with_expiration(text, severity, current_tick, current_tick + 300)
    }

    pub fn add_with_expiration(
        &mut self,
        text: String,
        severity: NotificationSeverity,
        current_tick: u64,
        expires_at: u64
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let notification = Notification {
            id,
            text,
            severity,
            created_at: current_tick,
            expires_at: Some(expires_at),
            action_link: None,
        };

        self.active.push(notification);

        // Enforce limit (FIFO)
        if self.active.len() > Self::MAX_NOTIFICATIONS {
            self.active.remove(0);
        }

        id
    }

    pub fn dismiss(&mut self, id: u64) {
        if let Some(pos) = self.active.iter().position(|n| n.id == id) {
            self.active.remove(pos);
        }
    }
}

pub fn notification_expiration_system(
    mut queue: ResMut<NotificationQueue>,
    time: Res<SimulationTime>,
) {
    let current_tick = time.tick;
    queue.active.retain(|n| {
        match n.expires_at {
            Some(expiry) => current_tick < expiry,
            None => true,
        }
    });
}
```

### 2. Register System

In `src/layer1/mod.rs` and `main.rs`, register `NotificationQueue` and `notification_expiration_system`.

## REFACTOR Phase: Quality & Design

- **UI Integration**: Expose `NotificationQueue` to the UI layer (`src/ui/status.rs` or new `src/ui/notifications.rs`) to render them.
- **Chronicle Integration**: When adding a `Major` notification, automatically log to Chronicle? Or keep distinct?
- **Sound**: Add an audio event trigger on `add()`.
- **Action Link**: If `action_link` is present, UI should allow clicking to center camera on entity.

## Acceptance Criteria

- [ ] `NotificationQueue` resource exists and manages a list of alerts.
- [ ] Notifications have severity levels (Info, Success, Warning, Error).
- [ ] Notifications auto-expire based on `SimulationTime`.
- [ ] Queue size is limited (e.g., 10) to prevent UI clutter.
- [ ] Tests pass.

## Technical Guidance

- Use `ResMut<NotificationQueue>` in systems that generate alerts (e.g., `combat_system`, `construction_system`).
- UI rendering code belongs in `src/ui/`, not here. This spec covers the *logic* and *data*.
