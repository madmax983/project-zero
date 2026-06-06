//! Cultural Artifacts and Auras.
//!
//! This module introduces `CulturalArtifact`s, which act as focal points of cultural expression.
//! Artifacts project an aura based on their `ArtifactTheme` (e.g., Victory, Tragedy).
//! Pops passing within this radius receive temporary `MoodModifier`s, allowing players
//! to strategically place art to manipulate colony sentiment.

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

/// A cultural artifact that projects a mood-altering aura over its surroundings.
///
/// Pops walking within the `aura_radius` will temporarily gain morale modifiers
/// corresponding to the artifact's `theme`.
///
/// # Examples
/// ```rust
/// use scale::layer1::culture::cultural_artifacts::{CulturalArtifact, ArtifactTheme};
/// use scale::layer1::map::GridPosition;
/// use bevy_ecs::prelude::*;
///
/// let mut world = World::new();
/// let artifact = world.spawn((
///     GridPosition { x: 10, y: 10 },
///     CulturalArtifact {
///         theme: ArtifactTheme::Victory,
///         aura_radius: 5,
///     }
/// )).id();
///
/// assert_eq!(world.get::<CulturalArtifact>(artifact).unwrap().aura_radius, 5);
/// ```
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
            let dx = art_pos.x.abs_diff(pop_pos.x);
            let dy = art_pos.y.abs_diff(pop_pos.y);

            // Chebychev distance for simplicity
            let distance = dx.max(dy);

            if distance <= artifact.aura_radius as u32 {
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
