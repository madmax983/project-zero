use bevy_ecs::prelude::*;

/// Represents the physical health of an entity (Pop).
///
/// Decouples death from specific causes (starvation, damage).
/// Current range: 0.0 to max (default 100.0).
#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    /// Current health points. <= 0 means death.
    pub current: f32,
    /// Maximum health points.
    pub max: f32,
}

/// Marker component for dead entities pending processing/despawn.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Dead;

/// Tracks physical trauma that affects biometric identification.
///
/// Scars accumulate over time due to damage or surgery and contribute to biometric drift.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Scars {
    pub count: u32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

impl Health {
    /// Returns true if health is greater than 0.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    /// Reduces health by amount, clamped at 0.
    /// Ignores negative damage (healing) and NaN.
    pub fn take_damage(&mut self, amount: f32) {
        if amount.is_nan() || amount < 0.0 {
            return;
        }
        self.current = (self.current - amount).max(0.0);
    }
}

/// System that checks for entities with zero health and marks them as Dead.
#[allow(clippy::type_complexity)]
pub fn check_health_status_system(
    mut commands: Commands,
    query: Query<(Entity, &Health), (Without<Dead>, Changed<Health>)>,
) {
    for (entity, health) in query.iter() {
        if !health.is_alive() {
            commands.entity(entity).insert(Dead);
        }
    }
}

/// Generic system to despawn dead entities.
/// Should run AFTER all specific death handlers have processed the Dead component.
pub fn despawn_dead_entities_system(mut commands: Commands, query: Query<Entity, With<Dead>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_defaults() {
        let health = Health::default();
        assert!((health.current - 100.0).abs() < f32::EPSILON);
        assert!((health.max - 100.0).abs() < f32::EPSILON);
        assert!(health.is_alive());
    }

    #[test]
    fn test_take_damage() {
        let mut health = Health::default();
        health.take_damage(10.0);
        assert!((health.current - 90.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_damage_clamped_at_zero() {
        let mut health = Health::default();
        health.take_damage(200.0);
        assert!((health.current - 0.0).abs() < f32::EPSILON);
        assert!(!health.is_alive());
    }

    #[test]
    fn test_death_check() {
        let mut health = Health::default();
        health.take_damage(100.0);
        assert!(!health.is_alive());
        assert!((health.current - 0.0).abs() < f32::EPSILON);
    }
}
