use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

/// Severity level of a notification, used for UI styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationSeverity {
    /// Informational message (blue/neutral).
    Info,
    /// Success message (green).
    Success,
    /// Warning message (yellow/orange).
    Warning,
    /// Critical error or alert (red).
    Error,
}

/// A temporary alert displayed to the player.
#[derive(Debug, Clone)]
pub struct Notification {
    /// Unique identifier.
    pub id: u64,
    /// The message text.
    pub text: String,
    /// Severity level.
    pub severity: NotificationSeverity,
    /// Tick when created.
    pub created_at: u64,
    /// Tick when it expires (if any).
    pub expires_at: Option<u64>,
    /// Optional entity to jump to when clicked.
    pub action_link: Option<Entity>,
}

/// Resource managing the queue of active notifications.
#[derive(Resource, Default)]
pub struct NotificationQueue {
    /// List of active notifications.
    pub active: Vec<Notification>,
    next_id: u64,
}

impl NotificationQueue {
    const MAX_NOTIFICATIONS: usize = 10;

    /// Adds a notification with default expiration (300 ticks).
    pub fn add(
        &mut self,
        text: impl Into<String>,
        severity: NotificationSeverity,
        current_tick: u64,
    ) -> u64 {
        // Default expiration: 300 ticks (30s at 10TPS)
        self.add_with_expiration(text, severity, current_tick, current_tick + 300)
    }

    /// Adds an informational notification.
    pub fn add_info(&mut self, text: impl Into<String>, current_tick: u64) -> u64 {
        self.add(text, NotificationSeverity::Info, current_tick)
    }

    /// Adds a success notification.
    pub fn add_success(&mut self, text: impl Into<String>, current_tick: u64) -> u64 {
        self.add(text, NotificationSeverity::Success, current_tick)
    }

    /// Adds a warning notification.
    pub fn add_warning(&mut self, text: impl Into<String>, current_tick: u64) -> u64 {
        self.add(text, NotificationSeverity::Warning, current_tick)
    }

    /// Adds an error notification.
    pub fn add_error(&mut self, text: impl Into<String>, current_tick: u64) -> u64 {
        self.add(text, NotificationSeverity::Error, current_tick)
    }

    /// Adds a notification with explicit expiration.
    pub fn add_with_expiration(
        &mut self,
        text: impl Into<String>,
        severity: NotificationSeverity,
        current_tick: u64,
        expires_at: u64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let notification = Notification {
            id,
            text: text.into(),
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

    /// Dismisses a notification by ID.
    pub fn dismiss(&mut self, id: u64) {
        if let Some(pos) = self.active.iter().position(|n| n.id == id) {
            self.active.remove(pos);
        }
    }
}

/// System that removes expired notifications.
pub fn notification_expiration_system(
    mut queue: ResMut<NotificationQueue>,
    time: Res<SimulationTime>,
) {
    let current_tick = time.tick;
    queue
        .active
        .retain(|n| n.expires_at.is_none_or(|expiry| current_tick < expiry));
}

#[cfg(test)]
mod tests {
    use crate::layer1::notifications::{
        notification_expiration_system, NotificationQueue, NotificationSeverity,
    };
    use crate::shared::time::SimulationTime;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

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
        world.insert_resource(SimulationTime {
            tick: 100,
            ..Default::default()
        });

        // Add notification that expires at tick 150
        let mut queue = world.resource_mut::<NotificationQueue>();
        queue.add_with_expiration(
            "Expires soon".to_string(),
            NotificationSeverity::Info,
            100,
            150,
        );

        // Run system at tick 100 (should stay)
        world
            .run_system_once(notification_expiration_system)
            .unwrap();
        assert_eq!(world.resource::<NotificationQueue>().active.len(), 1);

        // Advance time to 151
        world.resource_mut::<SimulationTime>().tick = 151;
        world
            .run_system_once(notification_expiration_system)
            .unwrap();
        assert!(world.resource::<NotificationQueue>().active.is_empty());
    }

    #[test]
    fn test_notification_limit() {
        let mut queue = NotificationQueue::default();
        // Add more than MAX (e.g., 10)
        for i in 0..15 {
            queue.add(format!("Alert {}", i), NotificationSeverity::Info, 0);
        }

        // Should clamp to 10 most recent
        assert!(queue.active.len() <= 10);
        assert_eq!(queue.active.last().unwrap().text, "Alert 14");
        // Oldest (Alert 0, 1, 2, 3, 4) should be gone.
        // Alert 5..14 should be present (10 items).
        assert_eq!(queue.active[0].text, "Alert 5");
    }

    #[test]
    fn test_dismiss_notification() {
        let mut queue = NotificationQueue::default();
        let id = queue.add("To Dismiss".to_string(), NotificationSeverity::Info, 0);

        assert_eq!(queue.active.len(), 1);
        queue.dismiss(id);
        assert!(queue.active.is_empty());
    }

    #[test]
    fn test_convenience_methods() {
        let mut queue = NotificationQueue::default();
        let tick = 100;

        let id_info = queue.add_info("Info", tick);
        let id_success = queue.add_success("Success", tick);
        let id_warning = queue.add_warning("Warning", tick);
        let id_error = queue.add_error("Error", tick);

        assert_eq!(queue.active.len(), 4);

        let n_info = queue.active.iter().find(|n| n.id == id_info).unwrap();
        assert_eq!(n_info.text, "Info");
        assert_eq!(n_info.severity, NotificationSeverity::Info);

        let n_success = queue.active.iter().find(|n| n.id == id_success).unwrap();
        assert_eq!(n_success.text, "Success");
        assert_eq!(n_success.severity, NotificationSeverity::Success);

        let n_warning = queue.active.iter().find(|n| n.id == id_warning).unwrap();
        assert_eq!(n_warning.text, "Warning");
        assert_eq!(n_warning.severity, NotificationSeverity::Warning);

        let n_error = queue.active.iter().find(|n| n.id == id_error).unwrap();
        assert_eq!(n_error.text, "Error");
        assert_eq!(n_error.severity, NotificationSeverity::Error);
    }
}
