use crate::layer1::map::GridPosition;
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::pop::Pop;
use crate::layer1::unrest::Unrest;
use bevy_ecs::prelude::*;

/// Component marking flora that emits the bio-acoustic chorus.
#[derive(Component, Default)]
pub struct BioAcousticFlora;

/// Generates a `MoodModifier` on nearby Pops based on the overall colony mood.
/// Colony mood is approximated as `1.0 - Unrest`.
pub fn bio_acoustic_chorus_system(
    unrest: Option<Res<Unrest>>,
    flora_query: Query<&GridPosition, With<BioAcousticFlora>>,
    mut pop_query: Query<(&GridPosition, &mut Morale), With<Pop>>,
) {
    let colony_mood = unrest.map_or(0.5, |u| 1.0 - u.level);

    // Spec says: High mood creates a beautiful, relaxing symphony that further buffs morale.
    // Low mood creates a dissonant, stressful screech.
    // Mood > 50.0 is high, <= 50.0 is low. We use 0.0 - 1.0 scale, so 0.5.
    let buff_value = if colony_mood > 0.5 { 0.05 } else { -0.05 };

    // Cache the label string so we don't allocate per pop per tick
    let label = if buff_value > 0.0 {
        "Relaxing Chorus".to_string()
    } else {
        "Stressful Screech".to_string()
    };

    for flora_pos in flora_query.iter() {
        for (pop_pos, mut morale) in pop_query.iter_mut() {
            // Fix underflow risk by casting to f32 before subtracting
            let dx = pop_pos.x as f32 - flora_pos.x as f32;
            let dy = pop_pos.y as f32 - flora_pos.y as f32;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < 10.0 {
                morale.add_modifier(MoodModifier {
                    label: label.clone(),
                    value: buff_value,
                    duration: 1, // Applies continuously while in range, so 1 tick duration
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_flora_emits_positive_chorus_on_high_colony_mood() {
        // Arrange
        let mut app = App::new();
        // Set colony mood to high by setting Unrest low (e.g., Unrest = 0.2 -> Mood = 0.8)
        app.world_mut().insert_resource(Unrest {
            level: 0.2,
            ..Default::default()
        });

        let _flora = app
            .world_mut()
            .spawn((BioAcousticFlora, GridPosition { x: 0, y: 0 }))
            .id();

        let pop = app
            .world_mut()
            .spawn((Pop, GridPosition { x: 1, y: 0 }, Morale::default()))
            .id();

        // Act
        app.add_systems(Update, bio_acoustic_chorus_system);
        app.update();

        // Assert
        let morale = app.world().get::<Morale>(pop).unwrap();
        assert!(
            !morale.modifiers.is_empty(),
            "Pop should receive a mood modifier"
        );
        let modifier = morale.modifiers.last().unwrap();
        assert!(
            modifier.value > 0.0,
            "Pop should receive a positive mood buff from the chorus"
        );
    }

    #[test]
    fn test_flora_emits_negative_screech_on_low_colony_mood() {
        // Arrange
        let mut app = App::new();
        // Set colony mood to low by setting Unrest high (e.g., Unrest = 0.8 -> Mood = 0.2)
        app.world_mut().insert_resource(Unrest {
            level: 0.8,
            ..Default::default()
        });

        let _flora = app
            .world_mut()
            .spawn((BioAcousticFlora, GridPosition { x: 0, y: 0 }))
            .id();

        let pop = app
            .world_mut()
            .spawn((Pop, GridPosition { x: 1, y: 0 }, Morale::default()))
            .id();

        // Act
        app.add_systems(Update, bio_acoustic_chorus_system);
        app.update();

        // Assert
        let morale = app.world().get::<Morale>(pop).unwrap();
        assert!(
            !morale.modifiers.is_empty(),
            "Pop should receive a mood modifier"
        );
        let modifier = morale.modifiers.last().unwrap();
        assert!(
            modifier.value < 0.0,
            "Pop should receive a negative mood debuff from the screech"
        );
    }

    #[test]
    fn test_flora_does_not_emit_out_of_range() {
        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(Unrest {
            level: 0.2,
            ..Default::default()
        });

        let _flora = app
            .world_mut()
            .spawn((BioAcousticFlora, GridPosition { x: 0, y: 0 }))
            .id();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 20, y: 0 }, // Out of range (distance >= 10.0)
                Morale::default(),
            ))
            .id();

        // Act
        app.add_systems(Update, bio_acoustic_chorus_system);
        app.update();

        // Assert
        let morale = app.world().get::<Morale>(pop).unwrap();
        assert!(
            morale.modifiers.is_empty(),
            "Pop should not receive a mood modifier if out of range"
        );
    }
}
