use crate::layer1::map::GridPosition;
use crate::layer1::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct EmotionalContagion {
    pub contagion_type: ContagionType,
    pub radius: f32, // Let's interpret radius in grid cells since we are using GridPosition
    pub strength: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContagionType {
    Panic,
    Joy,
    Rage,
}

pub fn emotional_contagion_system(
    sources: Query<(&GridPosition, &EmotionalContagion)>,
    mut targets: Query<(&GridPosition, &mut Morale), Without<EmotionalContagion>>,
) {
    for (source_pos, contagion) in sources.iter() {
        for (target_pos, mut target_morale) in targets.iter_mut() {
            let distance = ((source_pos.x - target_pos.x).pow(2) as f32
                + (source_pos.y - target_pos.y).pow(2) as f32)
                .sqrt();

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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_contagion_spreads_to_nearby_pops() {
        let mut world = World::new();

        // Arrange: Two pops, one panicking, one neutral, within radius
        let _source = world
            .spawn((
                GridPosition { x: 0, y: 0 },
                Morale {
                    value: 0.1,
                    ..Default::default()
                }, // Very low morale
                EmotionalContagion {
                    contagion_type: ContagionType::Panic,
                    radius: 5.0,
                    strength: -0.15,
                },
            ))
            .id();

        let target = world
            .spawn((
                GridPosition { x: 3, y: 0 },
                Morale {
                    value: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        // Act
        let _ = world.run_system_once(emotional_contagion_system);

        // Assert: Target should receive a negative mood modifier from contagion
        let target_morale = world.get::<Morale>(target).unwrap();
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
        let mut world = World::new();

        // Arrange: Target is outside the 5.0 radius
        let _source = world
            .spawn((
                GridPosition { x: 0, y: 0 },
                Morale {
                    value: 0.9,
                    ..Default::default()
                },
                EmotionalContagion {
                    contagion_type: ContagionType::Joy,
                    radius: 5.0,
                    strength: 0.10,
                },
            ))
            .id();

        let target = world
            .spawn((
                GridPosition { x: 10, y: 0 },
                Morale {
                    value: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        // Act
        let _ = world.run_system_once(emotional_contagion_system);

        // Assert: Target should NOT receive a mood modifier
        let target_morale = world.get::<Morale>(target).unwrap();
        assert!(
            !target_morale
                .modifiers
                .iter()
                .any(|m| m.label == "Contagion: Joy"),
            "Target incorrectly received contagion outside radius"
        );
    }

    #[test]
    fn test_contagion_stacking_limits() {
        let mut world = World::new();

        let _source = world
            .spawn((
                GridPosition { x: 0, y: 0 },
                Morale {
                    value: 0.1,
                    ..Default::default()
                },
                EmotionalContagion {
                    contagion_type: ContagionType::Panic,
                    radius: 5.0,
                    strength: -0.15,
                },
            ))
            .id();

        let target = world
            .spawn((
                GridPosition { x: 1, y: 0 },
                Morale {
                    value: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        // Run multiple times
        for _ in 0..5 {
            let _ = world.run_system_once(emotional_contagion_system);
        }

        // Ensure we don't have 5 Panic modifiers
        let target_morale = world.get::<Morale>(target).unwrap();
        let panic_count = target_morale
            .modifiers
            .iter()
            .filter(|m| m.label == "Contagion: Panic")
            .count();
        assert!(panic_count <= 1, "Modifiers should not stack indefinitely");
    }
}
