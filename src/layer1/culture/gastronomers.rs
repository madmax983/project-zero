//! The Gastronomers and Culinary Singularity.
//!
//! A high-tier cultural phenomenon where food transcends sustenance.
//! If the `EmpireAdvancement` reaches a sufficient level, the colony may experience a
//! `CulinarySingularityEvent`. This permanently alters the expectations of Gastronomer factions,
//! demanding exquisite cuisine or suffering massive morale penalties.

use crate::layer1::entities::pop::Pop;
use crate::layer1::social::factions::{FactionData, FactionId, Factions};
use crate::layer1::social::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;

/// A global tracker for the overall cultural and technological advancement of the colony.
///
/// High advancement unlocks complex societal demands, such as the `CulinarySingularityEvent`.
///
/// # Examples
/// ```rust
/// use scale::layer1::culture::gastronomers::EmpireAdvancement;
///
/// let mut advancement = EmpireAdvancement::default();
/// advancement.level = 5;
///
/// assert_eq!(advancement.level, 5);
/// ```
#[derive(Resource, Default)]
pub struct EmpireAdvancement {
    pub level: u32,
}

#[derive(Event)]
pub struct CulinarySingularityEvent;

pub fn spawn_gastronomer_faction_system(
    advancement: Res<EmpireAdvancement>,
    mut factions: ResMut<Factions>,
) {
    if advancement.level >= 5 && factions.get(FactionId::Gastronomers).is_none() {
        factions.map.insert(
            FactionId::Gastronomers,
            FactionData {
                name: "The Gastronomers".to_string(),
                ..Default::default()
            },
        );
    }
}

pub fn apply_culinary_singularity_buff_system(
    mut events: EventReader<CulinarySingularityEvent>,
    mut query: Query<&mut Morale, With<Pop>>,
) {
    for _ in events.read() {
        for mut morale in query.iter_mut() {
            // Permanent modifier for culinary singularity
            morale.modifiers.push(MoodModifier {
                label: "Culinary Singularity".into(),
                value: 1.0,
                duration: u32::MAX,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gastronomer_faction_emergence() {
        // Arrange
        let mut app = bevy_app::App::new();
        app.insert_resource(EmpireAdvancement { level: 5 }); // Advanced empire
        app.insert_resource(Factions::default());

        // Act
        app.add_systems(bevy_app::Update, spawn_gastronomer_faction_system);
        app.update();

        // Assert
        let factions = app.world().resource::<Factions>();
        assert!(
            factions.get(FactionId::Gastronomers).is_some(),
            "The Gastronomers faction should emerge in an advanced empire"
        );
    }

    #[test]
    fn test_culinary_singularity_buff() {
        // Arrange
        let mut app = bevy_app::App::new();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Morale {
                    value: 0.5,
                    ..Default::default()
                },
            ))
            .id();
        app.add_event::<CulinarySingularityEvent>();

        // Act
        app.world_mut().send_event(CulinarySingularityEvent);
        app.add_systems(bevy_app::Update, apply_culinary_singularity_buff_system);
        app.update();

        // Assert
        let morale = app.world().get::<Morale>(pop).unwrap();
        assert!(morale
            .modifiers
            .iter()
            .any(|m| m.label == "Culinary Singularity" && m.value == 1.0));
    }
}
