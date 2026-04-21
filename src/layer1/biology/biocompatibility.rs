//! Biocompatibility System (Spec 107).
//!
//! This module simulates how well Pops adapt to planetary hazards, applying damage to
//! health based on environmental pollution and intrinsic biological resistance (`Biocompatibility`).

use crate::layer1::atmosphere::AtmosphereGrid;
use crate::layer1::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

/// Biocompatibility component representing a pop's resistance to atmospheric hazards.
///
/// Values range from 0.0 (Extremely Vulnerable) to 1.0 (Fully Immune).
/// Default is 0.5.
#[derive(Component, Debug, Clone, Copy)]
pub struct Biocompatibility {
    /// The raw compatibility value (0.0 - 1.0).
    pub value: f32,
}

impl Default for Biocompatibility {
    fn default() -> Self {
        Self { value: 0.5 }
    }
}

/// System that applies damage to pops when atmospheric pollution exceeds their biocompatibility.
///
/// Pops take damage if `Hazard Level > Biocompatibility`.
/// Damage scales with the difference: `(Hazard - Bio) * 5.0`.
/// Traits like `NativeBorn` and `WeakImmunity` modify effective biocompatibility.
pub fn biocompatibility_system(world: &mut World) {
    let mut damages = Vec::new();

    // 1. Read Phase
    {
        // Create query first (requires mutable borrow)
        let mut query =
            world.query::<(Entity, &GridPosition, &Biocompatibility, Option<&Traits>)>();
        // Then get resource (immutable borrow)
        let grid = world.resource::<AtmosphereGrid>();

        for (entity, pos, bio, traits) in query.iter(world) {
            let hazard_level = grid.get(pos.x, pos.y);

            // Calculate effective bio
            let mut effective_bio = bio.value;
            if let Some(t) = traits {
                if t.has(Trait::NativeBorn) {
                    effective_bio += 0.3;
                }
                if t.has(Trait::WeakImmunity) {
                    effective_bio -= 0.2;
                }
            }

            // Check threshold
            // Tolerance buffer: damage only if hazard significantly exceeds bio
            // Ensure hazard is non-trivial (> EPSILON) to prevent damage in clean air for negative bio
            if hazard_level > f32::EPSILON && hazard_level > effective_bio {
                let delta = hazard_level - effective_bio;
                let damage = delta * 5.0; // Scaling factor
                damages.push((entity, damage));
            }
        }
    }

    // 2. Write Phase
    for (entity, damage) in damages {
        if let Some(mut health) = world.get_mut::<Health>(entity) {
            health.take_damage(damage);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::atmosphere::AtmosphereGrid;
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::traits::{Trait, Traits};

    #[test]
    fn test_biocompatibility_component_default() {
        let bio = Biocompatibility::default();
        // Default should be average, e.g., 0.5
        assert_eq!(bio.value, 0.5);
    }

    #[test]
    fn test_high_biocompatibility_resists_hazard() {
        let mut world = World::new();
        // Setup Hazard Grid (Atmosphere)
        let mut grid = AtmosphereGrid::new(10, 10);
        grid.set(0, 0, 0.4); // Moderate hazard
        world.insert_resource(grid);

        // Spawn Pop with High Bio (0.8) > Hazard (0.4)
        let pop = world
            .spawn((
                Pop,
                Biocompatibility { value: 0.8 },
                Health {
                    current: 100.0,
                    max: 100.0,
                    conditions: Vec::new(),
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Run system
        biocompatibility_system(&mut world);

        // Assert: No damage
        let health = world
            .get::<Health>(pop)
            .expect("Missing resource or component");
        assert_eq!(health.current, 100.0);
    }

    #[test]
    fn test_low_biocompatibility_takes_damage() {
        let mut world = World::new();
        let mut grid = AtmosphereGrid::new(10, 10);
        grid.set(0, 0, 0.6); // High hazard
        world.insert_resource(grid);

        // Spawn Pop with Low Bio (0.2) < Hazard (0.6)
        let pop = world
            .spawn((
                Pop,
                Biocompatibility { value: 0.2 },
                Health {
                    current: 100.0,
                    max: 100.0,
                    conditions: Vec::new(),
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        biocompatibility_system(&mut world);

        // Assert: Damage taken
        let health = world
            .get::<Health>(pop)
            .expect("Missing resource or component");
        assert!(health.current < 100.0);
    }

    #[test]
    fn test_traits_modify_biocompatibility() {
        // Verify NativeBorn boosts resistance
        let mut world = World::new();

        let mut grid = AtmosphereGrid::new(10, 10);
        grid.set(0, 0, 0.7); // High hazard
        world.insert_resource(grid);

        // Pop with Base 0.5 + NativeBorn (+0.3) = 0.8 > 0.7
        let pop = world
            .spawn((
                Pop,
                Biocompatibility { value: 0.5 },
                Health {
                    current: 100.0,
                    max: 100.0,
                    conditions: Vec::new(),
                },
                GridPosition { x: 0, y: 0 },
                {
                    let mut t = Traits::default();
                    t.add(Trait::NativeBorn);
                    t
                },
            ))
            .id();

        biocompatibility_system(&mut world);

        let health = world
            .get::<Health>(pop)
            .expect("Missing resource or component");
        assert_eq!(health.current, 100.0, "NativeBorn should resist hazard");
    }

    #[test]
    fn test_weak_immunity_safe_in_clean_air() {
        let mut world = World::new();
        // Setup clean atmosphere (default 0.0)
        let grid = AtmosphereGrid::new(10, 10);
        world.insert_resource(grid);

        // Spawn Pop with very low base bio (0.1) and WeakImmunity (-0.2).
        // Net effective bio = -0.1.
        let pop = world
            .spawn((
                Pop,
                Biocompatibility { value: 0.1 },
                Health {
                    current: 100.0,
                    max: 100.0,
                    conditions: Vec::new(),
                },
                GridPosition { x: 0, y: 0 },
                {
                    let mut t = Traits::default();
                    t.add(Trait::WeakImmunity);
                    t
                },
            ))
            .id();

        // Run system
        biocompatibility_system(&mut world);

        // Assert: No damage should be taken in clean air, even with negative immunity.
        let health = world
            .get::<Health>(pop)
            .expect("Missing resource or component");
        assert_eq!(health.current, 100.0, "Should be safe in clean air");
    }
}
