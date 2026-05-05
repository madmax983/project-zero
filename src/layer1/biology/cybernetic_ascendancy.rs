use bevy_ecs::prelude::*;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::mind::utility_types::UtilityWeights;

#[derive(Component, Default)]
pub struct CyberneticIntegration {
    pub integration_level: f32, // 0.0 to 1.0
}

#[derive(Resource, Default)]
pub struct ColonyAverageUtility {
    pub distance_weight: f32,
    pub availability_weight: f32,
}

pub fn cybernetic_integration_system(
    mut query: Query<&mut Needs, With<CyberneticIntegration>>
) {
    for mut needs in query.iter_mut() {
        // Instead of removing components which breaks the PopBundle, we freeze hunger/rest to 1.0 (satisfied)
        needs.hunger = 1.0;
        needs.rest = 1.0;
    }
}

pub fn update_colony_average_utility_system(
    mut avg: ResMut<ColonyAverageUtility>,
    query: Query<(&UtilityWeights, &CyberneticIntegration)>
) {
    let mut total_dist = 0.0;
    let mut total_avail = 0.0;
    let mut count = 0;

    for (weights, integration) in query.iter() {
        if integration.integration_level >= 1.0 {
            total_dist += weights.distance_weight;
            total_avail += weights.availability_weight;
            count += 1;
        }
    }

    if count > 0 {
        avg.distance_weight = total_dist / count as f32;
        avg.availability_weight = total_avail / count as f32;
    }
}

pub fn cybernetic_mind_merge_system(
    avg: Res<ColonyAverageUtility>,
    mut query: Query<(&mut UtilityWeights, &CyberneticIntegration)>
) {
    for (mut weights, integration) in query.iter_mut() {
        let blend = integration.integration_level;
        weights.distance_weight = weights.distance_weight * (1.0 - blend) + avg.distance_weight * blend;
        weights.availability_weight = weights.availability_weight * (1.0 - blend) + avg.availability_weight * blend;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};
    use crate::layer1::entities::pop::Pop;

    #[test]
    fn test_cybernetic_integration_removes_biological_needs() {
        let mut app = App::new();
        app.add_systems(Update, cybernetic_integration_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Needs { hunger: 0.5, rest: 0.5, leisure: 0.5, hygiene: 0.5 },
            CyberneticIntegration::default(),
        )).id();

        app.update();

        let needs = app.world().get::<Needs>(pop_entity).unwrap();
        assert_eq!(needs.hunger, 1.0, "Integrated Pop should have Hunger frozen at 1.0");
        assert_eq!(needs.rest, 1.0, "Integrated Pop should have Rest frozen at 1.0");
    }

    #[test]
    fn test_utility_weights_merge_towards_average() {
        let mut app = App::new();
        app.insert_resource(ColonyAverageUtility { distance_weight: 0.8, availability_weight: 0.2 });
        app.add_systems(Update, cybernetic_mind_merge_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            UtilityWeights { distance_weight: 0.2, availability_weight: 0.9 },
            CyberneticIntegration { integration_level: 0.5 },
        )).id();

        app.update();

        let weights = app.world().get::<UtilityWeights>(pop_entity).unwrap();
        // Should move towards the average
        assert!(weights.distance_weight > 0.2);
        assert!(weights.availability_weight < 0.9);
    }

    #[test]
    fn test_update_colony_average_utility_system() {
        let mut app = App::new();
        app.insert_resource(ColonyAverageUtility::default());
        app.add_systems(Update, update_colony_average_utility_system);

        app.world_mut().spawn((
            Pop,
            UtilityWeights { distance_weight: 1.0, availability_weight: 0.5 },
            CyberneticIntegration { integration_level: 1.0 },
        ));

        app.world_mut().spawn((
            Pop,
            UtilityWeights { distance_weight: 0.0, availability_weight: 0.5 },
            CyberneticIntegration { integration_level: 1.0 },
        ));

        app.update();

        let avg = app.world().get_resource::<ColonyAverageUtility>().unwrap();
        assert!((avg.distance_weight - 0.5).abs() < f32::EPSILON);
        assert!((avg.availability_weight - 0.5).abs() < f32::EPSILON);
    }
}
