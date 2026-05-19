use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct ColonyLanguage {
    pub base_id: u32,
    pub drift_vector: Vec<f32>,
}

impl ColonyLanguage {
    pub fn calculate_distance(&self, other: &ColonyLanguage) -> f32 {
        if self.base_id != other.base_id {
            return f32::MAX;
        }
        self.drift_vector
            .iter()
            .zip(other.drift_vector.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

#[derive(Component)]
pub struct DialectDrift(pub f32);

#[derive(Component)]
pub struct Translator;

#[derive(Component)]
pub struct LinguisticNetwork {
    pub nodes: Vec<Entity>,
}

pub fn apply_linguistic_drift(
    mut query: Query<(Entity, &mut ColonyLanguage)>,
    network_query: Query<&LinguisticNetwork>,
) {
    let mut rng = rand::thread_rng();
    let mut connected_nodes = std::collections::HashSet::new();
    for network in network_query.iter() {
        for &node in &network.nodes {
            connected_nodes.insert(node);
        }
    }

    for (entity, mut language) in query.iter_mut() {
        if !connected_nodes.contains(&entity) {
            // Apply drift randomly to avoid identical drift vectors
            for val in &mut language.drift_vector {
                *val += rng.gen_range(-1.0..1.0);
            }
        }
    }
}

pub fn translation_modifier(
    world: &World,
    pop1: Entity,
    pop2: Entity,
    translator: Option<Entity>,
) -> f32 {
    let diff;

    if let (Some(lang1), Some(lang2)) = (
        world.get::<ColonyLanguage>(pop1),
        world.get::<ColonyLanguage>(pop2),
    ) {
        diff = lang1.calculate_distance(lang2);
    } else {
        let drift1 = world.get::<DialectDrift>(pop1).map(|d| d.0).unwrap_or(0.0);
        let drift2 = world.get::<DialectDrift>(pop2).map(|d| d.0).unwrap_or(0.0);
        diff = (drift1 - drift2).abs();
    }

    if translator.is_some() {
        return 1.0;
    }

    if diff > 5.0 {
        return 0.0;
    }

    if diff > 0.0 {
        let modifier = 1.0 - (diff * 0.1);
        return modifier.max(0.1);
    }

    1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;

    #[derive(Component)]
    struct Pop;

    #[test]
    fn test_linguistic_drift_over_time() {
        let mut app = App::new();

        let base_lang_id = 1;
        let colony_a = app
            .world_mut()
            .spawn(ColonyLanguage {
                base_id: base_lang_id,
                drift_vector: vec![0.0, 0.0],
            })
            .id();
        let colony_b = app
            .world_mut()
            .spawn(ColonyLanguage {
                base_id: base_lang_id,
                drift_vector: vec![0.0, 0.0],
            })
            .id();

        app.world_mut().spawn(LinguisticNetwork {
            nodes: vec![colony_a],
        }); // Keep A stationary

        app.add_systems(Update, apply_linguistic_drift);
        app.update();
        app.update();

        let lang_a = app.world().get::<ColonyLanguage>(colony_a).unwrap();
        let lang_b = app.world().get::<ColonyLanguage>(colony_b).unwrap();

        let drift_distance = lang_a.calculate_distance(lang_b);
        assert!(
            drift_distance > 0.0,
            "Language drift should occur over time"
        );
    }

    #[test]
    fn test_dialect_social_penalty() {
        let mut app = App::new();
        let pop1 = app.world_mut().spawn((Pop, DialectDrift(1.0))).id();
        let pop2 = app.world_mut().spawn((Pop, DialectDrift(3.0))).id();

        let modifier = translation_modifier(app.world(), pop1, pop2, None);

        assert!(
            modifier < 1.0 && modifier > 0.5,
            "Expected a minor social penalty for dialect mismatch"
        );
    }

    #[test]
    fn test_language_barrier_requires_translator() {
        let mut app = App::new();
        let pop1 = app.world_mut().spawn((Pop, DialectDrift(1.0))).id();
        let pop2 = app.world_mut().spawn((Pop, DialectDrift(10.0))).id();

        let modifier = translation_modifier(app.world(), pop1, pop2, None);

        assert_eq!(
            modifier, 0.0,
            "Communication should fail completely across a language barrier without a translator"
        );
    }

    #[test]
    fn test_translator_enables_communication() {
        let mut app = App::new();
        let pop1 = app.world_mut().spawn((Pop, DialectDrift(1.0))).id();
        let pop2 = app.world_mut().spawn((Pop, DialectDrift(10.0))).id();
        let translator = app.world_mut().spawn((Pop, Translator)).id();

        let modifier = translation_modifier(app.world(), pop1, pop2, Some(translator));

        assert!(
            modifier > 0.0,
            "Translator should enable communication across a language barrier"
        );
    }

    #[test]
    fn test_communication_network_prevents_drift() {
        let mut app = App::new();
        let base_lang_id = 1;
        let colony_a = app
            .world_mut()
            .spawn(ColonyLanguage {
                base_id: base_lang_id,
                drift_vector: vec![0.0, 0.0],
            })
            .id();
        let colony_b = app
            .world_mut()
            .spawn(ColonyLanguage {
                base_id: base_lang_id,
                drift_vector: vec![0.0, 0.0],
            })
            .id();

        app.world_mut().spawn(LinguisticNetwork {
            nodes: vec![colony_a, colony_b],
        });

        app.add_systems(Update, apply_linguistic_drift);
        app.update();

        let lang_a = app.world().get::<ColonyLanguage>(colony_a).unwrap();
        let lang_b = app.world().get::<ColonyLanguage>(colony_b).unwrap();

        let drift_distance = lang_a.calculate_distance(lang_b);
        assert!(
            drift_distance < 0.1,
            "Communication networks should prevent linguistic drift"
        );
    }
}
