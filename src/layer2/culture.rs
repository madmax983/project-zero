//! Layer 2 Cultural Systems.
//!
//! Handles macro-scale cultural phenomena across star systems, such as the Founder Effect.
//!
//! Layer 2 Culture.
//!
//! Layer 2 Culture.
//!
pub mod founder_effect {
    use crate::layer1::psychology::traits::Trait;
    use bevy_ecs::prelude::*;
    use std::collections::HashMap;

    #[derive(Component)]
    pub struct ColonyShip {
        pub crew_traits: HashMap<Trait, u32>,
    }

    #[derive(Component)]
    pub struct FoundColonyAction {
        pub target_system: u32,
    }

    #[derive(Component)]
    pub struct ColonyCulture {
        pub dominant_trait: Trait,
        pub trait_distribution: HashMap<Trait, u32>,
    }

    pub fn found_colony_system(
        mut commands: Commands,
        query: Query<(Entity, &ColonyShip, &FoundColonyAction)>,
    ) {
        for (entity, ship, _) in query.iter() {
            let dominant_trait = ship
                .crew_traits
                .iter()
                .max_by_key(|&(_, count)| count)
                .map(|(t, _)| *t)
                .unwrap_or(Trait::HardWorker); // Fallback

            // Spawn the new colony with inherited culture
            commands.spawn((ColonyCulture {
                dominant_trait,
                trait_distribution: ship.crew_traits.clone(),
            },));

            commands.entity(entity).despawn();
        }
    }

    pub fn update_colony_culture_system(mut query: Query<&mut ColonyCulture>) {
        for mut culture in query.iter_mut() {
            if let Some(dominant_trait) = culture
                .trait_distribution
                .iter()
                .max_by_key(|&(_, count)| count)
                .map(|(t, _)| *t)
            {
                if culture.dominant_trait != dominant_trait {
                    culture.dominant_trait = dominant_trait;
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::layer1::psychology::traits::Trait;
        use bevy_app::{App, Update};

        #[test]
        fn test_colony_inherits_founder_traits() {
            // Arrange
            let mut app = App::new();
            app.add_systems(Update, found_colony_system);

            let mut ship_traits = HashMap::new();
            ship_traits.insert(Trait::Volatile, 10);
            ship_traits.insert(Trait::Compassionate, 1);

            let ship_entity = app
                .world_mut()
                .spawn((
                    ColonyShip {
                        crew_traits: ship_traits.clone(),
                    },
                    FoundColonyAction { target_system: 42 },
                ))
                .id();

            // Act
            app.update();

            // Assert
            let mut found_colony = false;
            let mut query = app.world_mut().query::<&ColonyCulture>();
            for colony in query.iter(app.world()) {
                assert_eq!(colony.dominant_trait, Trait::Volatile);
                found_colony = true;
            }
            assert!(found_colony);
            assert!(app.world().get_entity(ship_entity).is_err()); // Ship despawned
        }

        #[test]
        fn test_update_colony_culture_system_shifts_dominant_trait() {
            // Arrange
            let mut app = App::new();
            app.add_systems(Update, update_colony_culture_system);

            let mut distribution = HashMap::new();
            distribution.insert(Trait::Volatile, 5);
            distribution.insert(Trait::Compassionate, 10); // Compassionate is now dominant

            let colony_entity = app
                .world_mut()
                .spawn(ColonyCulture {
                    dominant_trait: Trait::Volatile, // Previously dominant
                    trait_distribution: distribution,
                })
                .id();

            // Act
            app.update();

            // Assert
            let colony = app.world().get::<ColonyCulture>(colony_entity).unwrap();
            assert_eq!(colony.dominant_trait, Trait::Compassionate);
        }
    }
}
pub use founder_effect::*;
