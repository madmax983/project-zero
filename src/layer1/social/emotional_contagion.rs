#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_contagion_spreads_to_nearby_pops() {
        let mut app = App::new();
        app.add_systems(Update, contagion_system);

        // Arrange: Two pops, one panicking, one neutral, within radius
        let _source = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Morale {
                    value: 10.0,
                    ..default()
                }, // Very low morale
                EmotionalContagion {
                    contagion_type: ContagionType::Panic,
                    radius: 5.0,
                    strength: -15.0,
                },
            ))
            .id();

        let target = app
            .world_mut()
            .spawn((
                Transform::from_xyz(3.0, 0.0, 0.0),
                Morale {
                    value: 50.0,
                    ..default()
                },
            ))
            .id();

        // Act
        app.update();

        // Assert: Target should receive a negative mood modifier from contagion
        let target_morale = app.world().get::<Morale>(target).unwrap();
        assert!(
            target_morale
                .modifiers
                .iter()
                .any(|m| m.label == "Contagion: Panic"),
            "Target did not receive Panic contagion modifier"
        );
    }

    #[test]
    fn test_contagion_ignores_distant_pops() {
        let mut app = App::new();
        app.add_systems(Update, contagion_system);

        // Arrange: Target is outside the 5.0 radius
        let _source = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                EmotionalContagion {
                    contagion_type: ContagionType::Joy,
                    radius: 5.0,
                    strength: 10.0,
                },
            ))
            .id();

        let target = app
            .world_mut()
            .spawn((
                Transform::from_xyz(10.0, 0.0, 0.0),
                Morale {
                    value: 50.0,
                    ..default()
                },
            ))
            .id();

        // Act
        app.update();

        // Assert: Target should NOT receive a mood modifier
        let target_morale = app.world().get::<Morale>(target).unwrap();
        assert!(
            !target_morale
                .modifiers
                .iter()
                .any(|m| m.label == "Contagion: Joy"),
            "Target incorrectly received contagion outside radius"
        );
    }
}

use crate::layer1::social::morale::{MoodModifier, Morale};
use bevy::prelude::*;

#[derive(Component)]
pub struct EmotionalContagion {
    pub contagion_type: ContagionType,
    pub radius: f32,
    pub strength: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContagionType {
    Panic,
    Joy,
    Rage,
}

pub fn contagion_system(
    sources: Query<(&Transform, &EmotionalContagion)>,
    mut targets: Query<(&Transform, &mut Morale), Without<EmotionalContagion>>,
) {
    for (source_transform, contagion) in sources.iter() {
        for (target_transform, mut target_morale) in targets.iter_mut() {
            let distance = source_transform
                .translation
                .distance(target_transform.translation);

            if distance <= contagion.radius {
                let modifier_name = match contagion.contagion_type {
                    ContagionType::Panic => "Contagion: Panic",
                    ContagionType::Joy => "Contagion: Joy",
                    ContagionType::Rage => "Contagion: Rage",
                };

                // Add or refresh modifier
                if !target_morale
                    .modifiers
                    .iter()
                    .any(|m| m.label == modifier_name)
                {
                    target_morale.modifiers.push(MoodModifier {
                        label: modifier_name.to_string(),
                        value: contagion.strength,
                        duration: 100, // Minimal fixed duration
                    });
                }
            }
        }
    }
}
