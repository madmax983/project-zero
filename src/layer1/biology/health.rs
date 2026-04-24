//! # Health and Mortality
//!
//! This module provides the foundational mechanics for tracking the physical
//! well-being of biological and synthetic entities (Pops) in the simulation.
//!
//! ## The Core Philosophy
//!
//! In SCALE, health is deliberately decoupled from the specific *causes* of
//! damage. Starvation, orbital crossfire, and workplace accidents all reduce
//! a unified [`Health`] pool. When health reaches zero, the entity is marked
//! with the [`Dead`] marker component rather than immediately despawned.
//!
//! This delayed despawn allows other systems (like Medical Triage, Organ
//! Harvesting, or Funerary Rites) to intercept and process the death event.

use bevy_ecs::prelude::*;

/// Represents the physical health of an entity (Pop).
///
/// Decouples death from specific causes (starvation, damage).
/// Current range: 0.0 to max (default 100.0).
///
/// # Examples
///
/// ```
/// use scale::layer1::health::Health;
///
/// let entity_health = Health { has_rust_lung: false,
///     current: 100.0,
///     max: 100.0,
/// };
/// assert!(entity_health.is_alive());
/// ```
#[derive(Component, Debug, Clone)]
pub struct Health {
    /// Current health points. <= 0 means death.
    pub current: f32,
    /// Maximum health points.
    pub max: f32,
    /// Whether the entity has RustLung.
    pub has_rust_lung: bool,
}

/// Marker component for dead entities pending processing/despawn.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Dead;

/// Records the proximate cause of death for downstream UI and chronicle systems.
#[derive(Component, Debug, Clone)]
pub struct DeathCause(pub String);

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
            has_rust_lung: false,
        }
    }
}

impl Health {
    /// Returns true if health is greater than 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::health::Health;
    ///
    /// let mut health = Health::default();
    /// assert!(health.is_alive());
    ///
    /// health.take_damage(100.0);
    /// assert!(!health.is_alive());
    /// ```
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    /// Reduces health by amount, clamped at 0.
    /// Ignores negative damage (healing) and NaN.
    ///
    /// # Panics
    ///
    /// This method will not panic, but passing `NaN` or negative numbers
    /// will result in a no-op to prevent accidental healing or state corruption.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::health::Health;
    ///
    /// let mut health = Health { current: 50.0, max: 100.0, has_rust_lung: false };
    /// health.take_damage(10.0);
    /// assert_eq!(health.current, 40.0);
    ///
    /// // Damage is clamped at zero
    /// health.take_damage(200.0);
    /// assert_eq!(health.current, 0.0);
    /// ```
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
