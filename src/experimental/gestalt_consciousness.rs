//! The Gestalt Consciousness (Nova Feature).
//!
//! # The Spark
//! We have individual needs, health, and traits. What if some Pops lost their individuality
//! completely and operated as a single hivemind?
//!
//! # The Feature
//! Implemented `GestaltParticipant`. A system calculates the average needs and health of all
//! participants and slowly drags their individual values towards the collective mean.
//! This creates a biological network where one starving Pop is sustained by the well-fed
//! others, but if the whole hive starves, they all suffer equally.

use crate::layer1::health::Health;
use crate::layer1::needs::Needs;
use bevy_ecs::prelude::*;

/// Marker component indicating a Pop is part of the hivemind.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GestaltParticipant;

const INTERPOLATION_FACTOR: f32 = 0.1;

/// Averages the needs and health of all Gestalt participants and pulls individuals towards the mean.
pub fn gestalt_equilibrium_system(
    mut query: Query<(&mut Needs, &mut Health), With<GestaltParticipant>>,
) {
    if query.is_empty() {
        return;
    }

    let mut total_hunger = 0.0;
    let mut total_rest = 0.0;
    let mut total_leisure = 0.0;
    let mut total_hygiene = 0.0;
    let mut total_health = 0.0;
    let count = query.iter().count() as f32;

    for (needs, health) in query.iter() {
        total_hunger += needs.hunger;
        total_rest += needs.rest;
        total_leisure += needs.leisure;
        total_hygiene += needs.hygiene;
        total_health += health.current;
    }

    let avg_hunger = total_hunger / count;
    let avg_rest = total_rest / count;
    let avg_leisure = total_leisure / count;
    let avg_hygiene = total_hygiene / count;
    let avg_health = total_health / count;

    for (mut needs, mut health) in query.iter_mut() {
        needs.hunger += (avg_hunger - needs.hunger) * INTERPOLATION_FACTOR;
        needs.rest += (avg_rest - needs.rest) * INTERPOLATION_FACTOR;
        needs.leisure += (avg_leisure - needs.leisure) * INTERPOLATION_FACTOR;
        needs.hygiene += (avg_hygiene - needs.hygiene) * INTERPOLATION_FACTOR;

        health.current += (avg_health - health.current) * INTERPOLATION_FACTOR;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::health::Health;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_gestalt_equilibrium() {
        let mut world = World::new();

        let pop1 = world
            .spawn((
                Pop,
                GestaltParticipant,
                Needs {
                    hunger: 0.0, // Starving
                    rest: 1.0,
                    leisure: 0.5,
                    hygiene: 0.5,
                },
                Health {
                    current: 50.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        let pop2 = world
            .spawn((
                Pop,
                GestaltParticipant,
                Needs {
                    hunger: 1.0, // Full
                    rest: 1.0,
                    leisure: 0.5,
                    hygiene: 0.5,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        world.run_system_once(gestalt_equilibrium_system).unwrap();

        let needs1 = world.get::<Needs>(pop1).unwrap();
        let health1 = world.get::<Health>(pop1).unwrap();
        let needs2 = world.get::<Needs>(pop2).unwrap();
        let health2 = world.get::<Health>(pop2).unwrap();

        // They should average out (hunger -> 0.5, health -> 75.0)
        // With an interpolation factor, they will move towards 0.5/75.0.
        assert!(needs1.hunger > 0.0, "Hunger should increase for pop1");
        assert!(needs2.hunger < 1.0, "Hunger should decrease for pop2");
        assert!(health1.current > 50.0, "Health should increase for pop1");
        assert!(health2.current < 100.0, "Health should decrease for pop2");
    }
}
