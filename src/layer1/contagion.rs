use bevy::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::{Morale, MoodModifier};

#[derive(Component)]
pub struct EmotionalContagion {
    pub contagion_type: ContagionType,
    pub radius: i32,
    pub strength: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContagionType {
    Panic,
    Joy,
    Rage,
}

pub fn trigger_contagion_system(
    mut commands: Commands,
    query: Query<(Entity, &Morale), Changed<Morale>>,
) {
    for (entity, morale) in query.iter() {
        if morale.value <= 0.2 {
            commands.entity(entity).insert(EmotionalContagion {
                contagion_type: ContagionType::Panic,
                radius: 5,
                strength: -0.05,
            });
        } else if morale.value >= 0.8 {
            commands.entity(entity).insert(EmotionalContagion {
                contagion_type: ContagionType::Joy,
                radius: 5,
                strength: 0.05,
            });
        } else {
            commands.entity(entity).remove::<EmotionalContagion>();
        }
    }
}

pub fn emotional_contagion_system(
    sources: Query<(&GridPosition, &EmotionalContagion)>,
    mut targets: Query<(&GridPosition, &mut Morale), Without<EmotionalContagion>>,
) {
    for (source_pos, contagion) in sources.iter() {
        for (target_pos, mut target_morale) in targets.iter_mut() {
            let distance = (source_pos.x - target_pos.x).abs().max((source_pos.y - target_pos.y).abs());

            if distance <= contagion.radius {
                let modifier_name = match contagion.contagion_type {
                    ContagionType::Panic => "Contagion: Panic",
                    ContagionType::Joy => "Contagion: Joy",
                    ContagionType::Rage => "Contagion: Rage",
                };

                // Add or refresh modifier
                if !target_morale.modifiers.iter().any(|m| m.label == modifier_name) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::morale::Morale;

    #[test]
    fn test_contagion_spreads_to_nearby_pops() {
        let mut app = App::new();
        app.add_systems(Update, emotional_contagion_system);

        // Arrange: Two pops, one panicking, one neutral, within radius
        let _source = app.world_mut().spawn((
            GridPosition { x: 0, y: 0 },
            Morale { value: 10.0, ..default() }, // Very low morale
            EmotionalContagion {
                contagion_type: ContagionType::Panic,
                radius: 5,
                strength: -0.15,
            },
        )).id();

        let target = app.world_mut().spawn((
            GridPosition { x: 3, y: 0 },
            Morale { value: 0.5, ..default() },
        )).id();

        // Act
        app.update();

        // Assert: Target should receive a negative mood modifier from contagion
        let target_morale = app.world().get::<Morale>(target).unwrap();
        assert!(
            target_morale.modifiers.iter().any(|m| m.label == "Contagion: Panic"),
            "Target did not receive Panic contagion modifier"
        );
    }

    #[test]
    fn test_contagion_ignores_distant_pops() {
        let mut app = App::new();
        app.add_systems(Update, emotional_contagion_system);

        // Arrange: Target is outside the 5.0 radius
        let _source = app.world_mut().spawn((
            GridPosition { x: 0, y: 0 },
            EmotionalContagion {
                contagion_type: ContagionType::Joy,
                radius: 5,
                strength: 0.10,
            },
        )).id();

        let target = app.world_mut().spawn((
            GridPosition { x: 10, y: 0 },
            Morale { value: 0.5, ..default() },
        )).id();

        // Act
        app.update();

        // Assert: Target should NOT receive a mood modifier
        let target_morale = app.world().get::<Morale>(target).unwrap();
        assert!(
            !target_morale.modifiers.iter().any(|m| m.label == "Contagion: Joy"),
            "Target incorrectly received contagion outside radius"
        );
    }

    #[test]
    fn test_contagion_stacking_limits() {
        // Tests that a pop doesn't get infinitely depressed by standing next to a panicking pop for a long time.
        // Ensure the modifier resets or caps.
        let mut app = App::new();
        app.add_systems(Update, emotional_contagion_system);

        let _source = app.world_mut().spawn((
            GridPosition { x: 0, y: 0 },
            EmotionalContagion {
                contagion_type: ContagionType::Panic,
                radius: 5,
                strength: -0.15,
            },
        )).id();

        let target = app.world_mut().spawn((
            GridPosition { x: 1, y: 0 },
            Morale { value: 0.5, ..default() },
        )).id();

        app.update();
        app.update();
        app.update(); // Multiple updates

        let target_morale = app.world().get::<Morale>(target).unwrap();
        let panic_count = target_morale.modifiers.iter().filter(|m| m.label == "Contagion: Panic").count();
        assert_eq!(panic_count, 1, "Modifiers should not stack indefinitely");
    }
}
