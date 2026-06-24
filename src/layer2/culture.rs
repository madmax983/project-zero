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

pub mod cultural_drift {
    use bevy_ecs::prelude::*;
    use bevy::prelude::Transform;
    use bevy::prelude::Vec2;

    #[derive(Resource)]
    pub struct HomeworldLocation {
        pub position: Vec2,
    }

    #[derive(Component)]
    pub struct ColonyMarker;

    #[derive(Component)]
    pub struct CulturalDrift {
        pub value: f32,
        pub independence_threshold: f32,
    }

    #[derive(Component)]
    pub struct CommsRelay {
        pub is_active: bool,
    }

    #[derive(Component)]
    pub struct Faction {
        pub id: u32,
    }

    pub fn calculate_cultural_drift_system(
        homeworld: Res<HomeworldLocation>,
        mut colonies: Query<(&Transform, &CommsRelay, &mut CulturalDrift), With<ColonyMarker>>,
    ) {
        for (transform, comms, mut drift) in colonies.iter_mut() {
            let pos2d = Vec2::new(transform.translation.x, transform.translation.y);
            let distance = pos2d.distance(homeworld.position);

            let mut rate = distance / 1000.0; // Base drift based on distance

            if comms.is_active {
                rate *= 0.1; // 90% reduction in drift if talking to homeworld
            }

            drift.value += rate;
        }
    }

    pub fn handle_independence_system(
        mut colonies: Query<(&CulturalDrift, &mut Faction), With<ColonyMarker>>,
    ) {
        for (drift, mut faction) in colonies.iter_mut() {
            if drift.value >= drift.independence_threshold {
                // Assign a new faction ID to symbolize independence
                // In a real implementation, this would trigger diplomacy events
                if faction.id == 0 {
                    faction.id = 999;
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use bevy_app::{App, Update};

        fn setup_app() -> App {
            let mut app = App::new();
            app.add_systems(Update, (calculate_cultural_drift_system, handle_independence_system));
            app.insert_resource(HomeworldLocation { position: Vec2::ZERO });
            app
        }

        #[test]
        fn test_colony_drift_increases_with_distance_and_time() {
            let mut app = setup_app();

            // Spawn a colony far away
            let colony_id = app.world_mut().spawn((
                ColonyMarker,
                Transform::from_xyz(1000.0, 0.0, 0.0),
                CulturalDrift { value: 0.0, independence_threshold: 100.0 },
                CommsRelay { is_active: false },
            )).id();

            app.update(); // Tick 1

            let drift = app.world().get::<CulturalDrift>(colony_id).unwrap();
            assert!(drift.value > 0.0); // Drift should increase
        }

        #[test]
        fn test_active_comms_reduce_drift_rate() {
            let mut app = setup_app();

            let far_colony_id = app.world_mut().spawn((
                ColonyMarker,
                Transform::from_xyz(1000.0, 0.0, 0.0),
                CulturalDrift { value: 0.0, independence_threshold: 100.0 },
                CommsRelay { is_active: false },
            )).id();

            let comms_colony_id = app.world_mut().spawn((
                ColonyMarker,
                Transform::from_xyz(1000.0, 0.0, 0.0),
                CulturalDrift { value: 0.0, independence_threshold: 100.0 },
                CommsRelay { is_active: true }, // Active comms!
            )).id();

            app.update();

            let drift_no_comms = app.world().get::<CulturalDrift>(far_colony_id).unwrap().value;
            let drift_with_comms = app.world().get::<CulturalDrift>(comms_colony_id).unwrap().value;

            assert!(drift_with_comms < drift_no_comms);
        }

        #[test]
        fn test_colony_declares_independence() {
            let mut app = setup_app();

            let colony_id = app.world_mut().spawn((
                ColonyMarker,
                Transform::from_xyz(1000.0, 0.0, 0.0),
                CulturalDrift { value: 101.0, independence_threshold: 100.0 },
                CommsRelay { is_active: false },
                Faction { id: 0 }, // Player faction
            )).id();

            app.update(); // Independence check runs

            let faction = app.world().get::<Faction>(colony_id).unwrap();
            assert_ne!(faction.id, 0); // Faction should have changed
        }
    }
}
