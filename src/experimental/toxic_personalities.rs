//! Toxic Personalities (Nova Feature)
//!
//! Connects psychological traits (`Trait::Volatile`, `Trait::Spiteful`),
//! `StressTracker`, and the physical `AtmosphereGrid`.
//!
//! Pops with "toxic" personalities literally emit physical pollution into the
//! air when they are highly stressed.

use crate::layer1::entities::pop::Pop;
use crate::layer1::map::GridPosition;
use crate::layer1::nature::atmosphere::AtmosphereGrid;
use crate::layer1::psychology::stress::StressTracker;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

const TOXIC_STRESS_THRESHOLD: f32 = 80.0;
const TOXIC_EMISSION_AMOUNT: f32 = 0.5;

pub fn toxic_personalities_system(
    query: Query<(&GridPosition, &Traits, &StressTracker), With<Pop>>,
    mut atmosphere: ResMut<AtmosphereGrid>,
) {
    for (pos, traits, stress) in query.iter() {
        if stress.accumulated_stress > TOXIC_STRESS_THRESHOLD
            && (traits.has(Trait::Volatile) || traits.has(Trait::Spiteful))
        {
            atmosphere.add(pos.x, pos.y, TOXIC_EMISSION_AMOUNT);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_toxic_personalities_emit_smog() {
        let mut world = World::new();
        world.insert_resource(AtmosphereGrid::new(10, 10));

        let mut toxic_traits = Traits::default();
        toxic_traits.add(Trait::Volatile);

        // Toxic pop, highly stressed
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            toxic_traits,
            StressTracker {
                accumulated_stress: 90.0,
            },
        ));

        // Normal pop, highly stressed (should not emit)
        world.spawn((
            Pop,
            GridPosition { x: 2, y: 2 },
            Traits::default(),
            StressTracker {
                accumulated_stress: 90.0,
            },
        ));

        // Toxic pop, low stress (should not emit)
        let mut spiteful_traits = Traits::default();
        spiteful_traits.add(Trait::Spiteful);
        world.spawn((
            Pop,
            GridPosition { x: 7, y: 7 },
            spiteful_traits,
            StressTracker {
                accumulated_stress: 10.0,
            },
        ));

        world.run_system_once(toxic_personalities_system).unwrap();

        let atmosphere = world.resource::<AtmosphereGrid>();

        // Toxic stressed pop emitted
        assert!(atmosphere.get(5, 5) > 0.0);
        // Normal stressed pop did not emit
        assert_eq!(atmosphere.get(2, 2), 0.0);
        // Toxic unstressed pop did not emit
        assert_eq!(atmosphere.get(7, 7), 0.0);
    }
}
