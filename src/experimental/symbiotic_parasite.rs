//! The Symbiotic Parasite (Nova Feature).
//!
//! # The Spark
//! We have `Health` regeneration and `Hunger` needs. What if there was a biological
//! trade-off mechanic?
//!
//! # The Feature
//! Implemented `SymbioticParasite`. Pops infected with this parasite lose hunger much
//! faster but gain passive health regeneration. This serves as a risky biological
//! augment for pops working in hazardous environments.

use crate::layer1::health::Health;
use crate::layer1::needs::Needs;
use bevy_ecs::prelude::*;

/// A biological parasite that heals the host but drastically increases hunger.
#[derive(Component)]
pub struct SymbioticParasite;

const PARASITE_HUNGER_DRAIN: f32 = 0.05;
const PARASITE_HEALTH_REGEN: f32 = 5.0;

/// System that applies the effects of the Symbiotic Parasite.
pub fn symbiotic_parasite_system(
    mut query: Query<(&mut Needs, &mut Health), With<SymbioticParasite>>,
) {
    for (mut needs, mut health) in query.iter_mut() {
        // Drain hunger faster
        needs.hunger = (needs.hunger - PARASITE_HUNGER_DRAIN).max(0.0);

        // Regenerate health
        health.current = (health.current + PARASITE_HEALTH_REGEN).min(health.max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_symbiotic_parasite_effect() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                SymbioticParasite,
                Needs {
                    hunger: 0.5,
                    ..Default::default()
                },
                Health {
                    current: 50.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        world.run_system_once(symbiotic_parasite_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        let health = world.get::<Health>(pop).unwrap();

        assert!((needs.hunger - 0.45).abs() < f32::EPSILON);
        assert!((health.current - 55.0).abs() < f32::EPSILON);
    }
}
