use crate::layer1::entities::pop::{Caution, Courage, Mood};
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArtifactTheme {
    Victory,
    Tragedy,
    Feast,
}

#[derive(Component)]
pub struct CulturalArtifact {
    pub theme: ArtifactTheme,
    pub aura_radius: i32,
}

#[allow(clippy::type_complexity)]
pub fn cultural_aura_system(
    artifacts: Query<(&CulturalArtifact, &GridPosition)>,
    mut pops: Query<(&mut Courage, &mut Caution, &mut Mood, &GridPosition)>,
) {
    for (artifact, art_pos) in artifacts.iter() {
        for (mut courage, mut caution, mut mood, pop_pos) in pops.iter_mut() {
            let dx = (art_pos.x - pop_pos.x).abs();
            let dy = (art_pos.y - pop_pos.y).abs();

            // Chebychev distance for simplicity
            let distance = dx.max(dy);

            if distance <= artifact.aura_radius {
                match artifact.theme {
                    ArtifactTheme::Victory => {
                        courage.value = (courage.value + 1.0).min(100.0);
                    }
                    ArtifactTheme::Tragedy => {
                        caution.value = (caution.value + 1.0).min(100.0);
                        mood.value = (mood.value - 1.0).max(0.0);
                    }
                    ArtifactTheme::Feast => {
                        mood.value = (mood.value + 1.0).min(100.0);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::entities::pop::Pop;
    use super::*;
    use crate::layer1::map::GridPosition;
    use bevy_app::{App, Update};

    #[test]
    fn test_victory_art_aura_buffs_courage() {
        let mut app = App::new();
        app.add_systems(Update, cultural_aura_system);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Courage { value: 50.0 },
                Caution { value: 50.0 },
                Mood { value: 50.0 },
            ))
            .id();

        let _art_entity = app
            .world_mut()
            .spawn((
                CulturalArtifact {
                    theme: ArtifactTheme::Victory,
                    aura_radius: 5,
                },
                GridPosition { x: 5, y: 5 }, // Same tile as Pop
            ))
            .id();

        app.update();

        let courage = app.world().get::<Courage>(pop_entity).unwrap();
        assert!(
            courage.value > 50.0,
            "Victory art should buff Courage within its aura"
        );
    }

    #[test]
    fn test_tragedy_art_aura_buffs_caution_and_lowers_mood() {
        let mut app = App::new();
        app.add_systems(Update, cultural_aura_system);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Courage { value: 50.0 },
                Caution { value: 50.0 },
                Mood { value: 50.0 },
            ))
            .id();

        let _art_entity = app
            .world_mut()
            .spawn((
                CulturalArtifact {
                    theme: ArtifactTheme::Tragedy,
                    aura_radius: 5,
                },
                GridPosition { x: 7, y: 7 }, // Within radius
            ))
            .id();

        app.update();

        let caution = app.world().get::<Caution>(pop_entity).unwrap();
        let mood = app.world().get::<Mood>(pop_entity).unwrap();

        assert!(caution.value > 50.0, "Tragedy art should buff Caution");
        assert!(mood.value < 50.0, "Tragedy art should lower Mood");
    }

    #[test]
    fn test_aura_range_limit() {
        let mut app = App::new();
        app.add_systems(Update, cultural_aura_system);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 15, y: 15 },
                Courage { value: 50.0 },
                Caution { value: 50.0 },
                Mood { value: 50.0 },
            ))
            .id();

        let _art_entity = app
            .world_mut()
            .spawn((
                CulturalArtifact {
                    theme: ArtifactTheme::Victory,
                    aura_radius: 5,
                },
                GridPosition { x: 5, y: 5 }, // Out of radius
            ))
            .id();

        app.update();

        let courage = app.world().get::<Courage>(pop_entity).unwrap();
        assert_eq!(
            courage.value, 50.0,
            "Art outside aura radius should not affect Pop"
        );
    }
}
