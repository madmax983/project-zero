use crate::layer1::mind::UtilityWeights;
use crate::layer1::needs::Needs;
use bevy_ecs::prelude::*;

#[derive(Component, Default)]
pub struct CyberneticIntegration {
    pub integration_level: f32, // 0.0 to 1.0
}

#[derive(Resource, Default)]
pub struct ColonyAverageUtility {
    pub weights: UtilityWeights,
}

pub fn cybernetic_integration_system(mut query: Query<&mut Needs, With<CyberneticIntegration>>) {
    for mut needs in query.iter_mut() {
        needs.hunger = 1.0;
        needs.rest = 1.0;
    }
}

pub fn calculate_colony_average_utility_system(
    mut avg: ResMut<ColonyAverageUtility>,
    query: Query<&UtilityWeights, With<CyberneticIntegration>>,
) {
    let mut total_distance = 0.0;
    let mut total_availability = 0.0;
    let mut count = 0.0;

    for weights in query.iter() {
        total_distance += weights.distance_weight;
        total_availability += weights.availability_weight;
        count += 1.0;
    }

    if count > 0.0 {
        avg.weights.distance_weight = total_distance / count;
        avg.weights.availability_weight = total_availability / count;
    } else {
        avg.weights = UtilityWeights::default();
    }
}

pub fn cybernetic_mind_merge_system(
    avg: Res<ColonyAverageUtility>,
    mut query: Query<(&mut UtilityWeights, &CyberneticIntegration)>,
) {
    for (mut weights, integration) in query.iter_mut() {
        let blend = integration.integration_level;
        weights.distance_weight =
            weights.distance_weight * (1.0 - blend) + avg.weights.distance_weight * blend;
        weights.availability_weight =
            weights.availability_weight * (1.0 - blend) + avg.weights.availability_weight * blend;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;
    use bevy::prelude::*;

    #[test]
    fn test_cybernetic_integration_removes_biological_needs() {
        let mut app = App::new();
        app.add_systems(Update, cybernetic_integration_system);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                Needs {
                    hunger: 0.5,
                    rest: 0.5,
                    leisure: 0.5,
                    hygiene: 0.5,
                },
                CyberneticIntegration::default(),
            ))
            .id();

        app.update();

        let needs = app.world().get::<Needs>(pop_entity).unwrap();
        assert_eq!(needs.hunger, 1.0, "Integrated Pop should have max Hunger");
        assert_eq!(needs.rest, 1.0, "Integrated Pop should have max Rest");
    }

    #[test]
    fn test_utility_weights_merge_towards_average() {
        let mut app = App::new();
        app.insert_resource(ColonyAverageUtility {
            weights: UtilityWeights {
                distance_weight: 0.8,
                availability_weight: 0.2,
            },
        });
        app.add_systems(Update, cybernetic_mind_merge_system);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                UtilityWeights {
                    distance_weight: 0.2,
                    availability_weight: 0.9,
                },
                CyberneticIntegration {
                    integration_level: 0.5,
                },
            ))
            .id();

        app.update();

        let weights = app.world().get::<UtilityWeights>(pop_entity).unwrap();
        assert!(weights.distance_weight > 0.2);
        assert!(weights.availability_weight < 0.9);
    }

    #[test]
    fn test_calculate_colony_average_utility_system() {
        let mut app = App::new();
        app.init_resource::<ColonyAverageUtility>();
        app.add_systems(Update, calculate_colony_average_utility_system);

        app.world_mut().spawn((
            Pop,
            UtilityWeights {
                distance_weight: 0.5,
                availability_weight: 0.5,
            },
            CyberneticIntegration::default(),
        ));

        app.world_mut().spawn((
            Pop,
            UtilityWeights {
                distance_weight: 1.5,
                availability_weight: 1.5,
            },
            CyberneticIntegration::default(),
        ));

        app.update();

        let avg = app.world().get_resource::<ColonyAverageUtility>().unwrap();
        assert_eq!(avg.weights.distance_weight, 1.0);
        assert_eq!(avg.weights.availability_weight, 1.0);
    }
}
