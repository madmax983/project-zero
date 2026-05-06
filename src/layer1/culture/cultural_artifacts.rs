use crate::layer1::entities::pop::Pop;
use crate::layer1::map::GridPosition;
use crate::layer1::social::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArtifactTheme {
    Victory,
    Tragedy,
    Feast,
}

#[derive(Component, Debug)]
pub struct CulturalArtifact {
    pub theme: ArtifactTheme,
    pub aura_radius: i32,
}

pub fn cultural_aura_system(
    artifacts: Query<(&CulturalArtifact, &GridPosition)>,
    mut pops: Query<(&mut Morale, &GridPosition), With<Pop>>,
) {
    for (artifact, art_pos) in artifacts.iter() {
        for (mut morale, pop_pos) in pops.iter_mut() {
            let dx = (art_pos.x - pop_pos.x).abs();
            let dy = (art_pos.y - pop_pos.y).abs();

            // Chebychev distance for simplicity
            let distance = dx.max(dy);

            if distance <= artifact.aura_radius {
                match artifact.theme {
                    ArtifactTheme::Victory => {
                        morale.add_modifier(MoodModifier {
                            label: "Victory Art Aura".to_string(),
                            value: 0.1,
                            duration: 1, // transient
                        });
                    }
                    ArtifactTheme::Tragedy => {
                        morale.add_modifier(MoodModifier {
                            label: "Tragedy Art Aura".to_string(),
                            value: -0.1,
                            duration: 1, // transient
                        });
                    }
                    ArtifactTheme::Feast => {
                        morale.add_modifier(MoodModifier {
                            label: "Feast Art Aura".to_string(),
                            value: 0.1,
                            duration: 1, // transient
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::social::morale::Morale;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_victory_art_aura_buffs_morale() {
        let mut app = App::new();
        app.add_systems(Update, cultural_aura_system);

        let pop_entity = app
            .world_mut()
            .spawn((Pop, GridPosition { x: 5, y: 5 }, Morale::default()))
            .id();

        app.world_mut().spawn((
            CulturalArtifact {
                theme: ArtifactTheme::Victory,
                aura_radius: 5,
            },
            GridPosition { x: 5, y: 5 }, // Same tile as Pop
        ));

        app.update();

        let morale = app.world().get::<Morale>(pop_entity).unwrap();
        assert!(morale
            .modifiers
            .iter()
            .any(|m| m.label == "Victory Art Aura"));
    }

    #[test]
    fn test_tragedy_art_aura_lowers_morale() {
        let mut app = App::new();
        app.add_systems(Update, cultural_aura_system);

        let pop_entity = app
            .world_mut()
            .spawn((Pop, GridPosition { x: 5, y: 5 }, Morale::default()))
            .id();

        app.world_mut().spawn((
            CulturalArtifact {
                theme: ArtifactTheme::Tragedy,
                aura_radius: 5,
            },
            GridPosition { x: 7, y: 7 }, // Within radius
        ));

        app.update();

        let morale = app.world().get::<Morale>(pop_entity).unwrap();
        assert!(morale
            .modifiers
            .iter()
            .any(|m| m.label == "Tragedy Art Aura"));
    }

    #[test]
    fn test_aura_range_limit() {
        let mut app = App::new();
        app.add_systems(Update, cultural_aura_system);

        let pop_entity = app
            .world_mut()
            .spawn((Pop, GridPosition { x: 15, y: 15 }, Morale::default()))
            .id();

        app.world_mut().spawn((
            CulturalArtifact {
                theme: ArtifactTheme::Victory,
                aura_radius: 5,
            },
            GridPosition { x: 5, y: 5 }, // Out of radius
        ));

        app.update();

        let morale = app.world().get::<Morale>(pop_entity).unwrap();
        assert!(!morale
            .modifiers
            .iter()
            .any(|m| m.label == "Victory Art Aura"));
    }
}
