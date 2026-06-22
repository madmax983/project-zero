#![allow(clippy::type_complexity)]
use crate::layer2::fleet::{Fleet, FleetHealth, InOrbit, MovementSpeed};
use crate::layer2::sensors::VisibilityStatus;
use bevy_ecs::prelude::*;
use rand::prelude::*;

/// Component indicating the presence of a planetary ring around an orbital body.
#[derive(Component, Debug, Clone, Copy)]
pub struct PlanetaryRing {
    /// Density of the ring, affecting both stealth bonus and collision risk.
    pub density: f32,
}

/// Applies effects to fleets currently orbiting a planet with rings.
pub fn apply_planetary_ring_effects_system(
    ring_query: Query<&PlanetaryRing>,
    mut fleet_query: Query<
        (
            &InOrbit,
            Option<&mut VisibilityStatus>,
            Option<&mut MovementSpeed>,
            Option<&mut FleetHealth>,
        ),
        With<Fleet>,
    >,
) {
    let mut rng = thread_rng();

    for (in_orbit, mut maybe_visibility, mut maybe_speed, mut maybe_health) in &mut fleet_query {
        if let Ok(ring) = ring_query.get(in_orbit.parent) {
            // Apply stealth bonus (reduce visibility)
            if let Some(visibility) = maybe_visibility.as_mut() {
                // If the ring is dense enough, fleet becomes invisible
                if ring.density > 0.5 {
                    visibility.is_visible = false;
                }
            }

            // Apply speed bonus
            if let Some(speed) = maybe_speed.as_mut() {
                // E.g., a 20% speed boost multiplied by density
                let speed_boost = 1.0 + (0.2 * ring.density);
                speed.current = speed.base * speed_boost;
            }

            // Apply collision damage risk
            if let Some(health) = maybe_health.as_mut() {
                // 5% base chance multiplied by density
                let collision_chance = (0.05 * ring.density).clamp(0.0, 1.0);
                if rng.gen_bool(collision_chance as f64) {
                    // Deal 10 damage
                    health.current -= 10.0;
                    if health.current < 0.0 {
                        health.current = 0.0;
                    }
                }
            }
        } else {
            // Reset speed if not in ring
            if let Some(speed) = maybe_speed.as_mut() {
                speed.current = speed.base;
            }
            // We do not reset visibility to true here unconditionally,
            // as it may be affected by other systems (e.g. sensor_occlusion_system)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_fleet_stealth_in_rings() {
        let mut app = App::new();
        app.add_systems(Update, apply_planetary_ring_effects_system);

        let planet = app.world_mut().spawn(PlanetaryRing { density: 1.0 }).id();
        let fleet = app
            .world_mut()
            .spawn((
                Fleet,
                InOrbit { parent: planet },
                VisibilityStatus { is_visible: true },
            ))
            .id();

        app.update();

        let visibility = app.world().get::<VisibilityStatus>(fleet).unwrap();
        assert!(
            !visibility.is_visible,
            "Fleet should be hidden in a dense ring"
        );
    }

    #[test]
    fn test_fleet_speed_in_rings() {
        let mut app = App::new();
        app.add_systems(Update, apply_planetary_ring_effects_system);

        let planet = app.world_mut().spawn(PlanetaryRing { density: 1.0 }).id();
        let fleet = app
            .world_mut()
            .spawn((
                Fleet,
                InOrbit { parent: planet },
                MovementSpeed {
                    base: 100.0,
                    current: 100.0,
                },
            ))
            .id();

        app.update();

        let speed = app.world().get::<MovementSpeed>(fleet).unwrap();
        assert!(speed.current > 100.0, "Fleet should move faster in a ring");
    }

    #[test]
    fn test_ring_collision_damage() {
        let mut app = App::new();
        app.add_systems(Update, apply_planetary_ring_effects_system);

        let planet = app
            .world_mut()
            .spawn(PlanetaryRing { density: 1000.0 })
            .id(); // Extremely dense for test
        let fleet = app
            .world_mut()
            .spawn((
                Fleet,
                InOrbit { parent: planet },
                FleetHealth {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        // 1000 density guarantees a collision
        app.update();

        let health = app.world().get::<FleetHealth>(fleet).unwrap();
        assert!(
            health.current < 100.0,
            "Fleet should take damage in a dense ring"
        );
    }
}
