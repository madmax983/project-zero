use crate::layer1::core::map::GridPosition;
use crate::layer1::nature::fire::Fire;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

pub fn pyromaniac_euphoria_system(
    mut pops: Query<(&GridPosition, &mut Needs, &Traits), With<Pop>>,
    fires: Query<&GridPosition, With<Fire>>,
) {
    for (pop_pos, mut needs, traits) in pops.iter_mut() {
        if traits.has(Trait::Pyromaniac) {
            // Check if any fire is within 3 tiles (Chebyshev distance)
            let is_near_fire = fires
                .iter()
                .any(|fire_pos| pop_pos.distance_chebyshev(*fire_pos) <= 3);

            if is_near_fire {
                // Passively regenerate leisure
                needs.leisure = (needs.leisure + 0.05).min(1.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pyromaniac_euphoria() {
        let mut world = World::new();

        // 1. Spawn a fire
        let _fire = world
            .spawn((Fire::default(), GridPosition { x: 5, y: 5 }))
            .id();

        // 2. Spawn a pyromaniac near the fire (distance 2)
        let mut pyro_traits = Traits::default();
        pyro_traits.add(Trait::Pyromaniac);
        let pyro = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
                pyro_traits,
                GridPosition { x: 7, y: 5 },
            ))
            .id();

        // 3. Spawn a normal pop near the fire
        let normal = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
                Traits::default(),
                GridPosition { x: 6, y: 5 },
            ))
            .id();

        // 4. Spawn a pyromaniac far from the fire (distance 10)
        let mut pyro_traits2 = Traits::default();
        pyro_traits2.add(Trait::Pyromaniac);
        let far_pyro = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
                pyro_traits2,
                GridPosition { x: 15, y: 5 },
            ))
            .id();

        // Run the system once
        use bevy_ecs::system::RunSystemOnce;
        world
            .run_system_once(super::pyromaniac_euphoria_system)
            .unwrap();

        // Verify pyro near fire gained leisure
        let pyro_needs = world.get::<Needs>(pyro).unwrap();
        assert!(
            pyro_needs.leisure > 0.5,
            "Pyromaniac near fire should gain leisure"
        );
        assert!((pyro_needs.leisure - 0.55).abs() < 0.001);

        // Verify normal pop near fire did not gain leisure
        let normal_needs = world.get::<Needs>(normal).unwrap();
        assert!(
            (normal_needs.leisure - 0.5).abs() < 0.001,
            "Normal pop near fire should not gain leisure"
        );

        // Verify pyro far from fire did not gain leisure
        let far_pyro_needs = world.get::<Needs>(far_pyro).unwrap();
        assert!(
            (far_pyro_needs.leisure - 0.5).abs() < 0.001,
            "Pyromaniac far from fire should not gain leisure"
        );
    }
}
