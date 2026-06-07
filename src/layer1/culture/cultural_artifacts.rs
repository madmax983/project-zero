//! Cultural Artifacts and Localized Mood Auras.
//!
//! This module manages `CulturalArtifact` entities, which emit localized mood-altering
//! auras over the colony map. Depending on the artifact's `ArtifactTheme`, pops
//! within the `aura_radius` will receive temporary positive or negative morale modifiers.
//!
//! # Examples
//!
//! ```rust
//! use bevy_app::prelude::*;
//! use scale::layer1::culture::cultural_artifacts::{ArtifactTheme, CulturalArtifact, cultural_aura_system};
//! use scale::layer1::entities::pop::Pop;
//! use scale::layer1::map::GridPosition;
//! use scale::layer1::social::morale::Morale;
//!
//! let mut app = App::new();
//! app.add_systems(Update, cultural_aura_system);
//!
//! // Spawn a pop at (5, 5) with baseline morale
//! let pop = app.world_mut().spawn((
//!     Pop,
//!     GridPosition { x: 5, y: 5 },
//!     Morale::default()
//! )).id();
//!
//! // Spawn a Victory artifact at the same location with a radius of 5
//! app.world_mut().spawn((
//!     CulturalArtifact {
//!         theme: ArtifactTheme::Victory,
//!         aura_radius: 5,
//!     },
//!     GridPosition { x: 5, y: 5 },
//! ));
//!
//! app.update();
//!
//! // The pop's morale is boosted by the artifact's aura
//! let morale = app.world().get::<Morale>(pop).unwrap();
//! assert!(morale.modifiers.iter().any(|m| m.label == "Victory Art Aura"));
//! ```

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
