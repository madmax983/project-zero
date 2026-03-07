# 419: The Bio-Acoustic Chorus

## 1. Overview
**Layer:** 1
**Fantasy:** The forest sings your colony's health.
**Mechanic:** A specific type of native flora emits a low hum that harmonizes with the overall Mood of the colony. High mood creates a beautiful, relaxing symphony that further buffs morale. Low mood creates a dissonant, stressful screech.
**Emergence:** Your colony is starving. The forest starts screaming, creating a feedback loop of stress that drives the colonists insane before the hunger does.
**Tension:** Do you clear-cut the forest for silence, or try to fix the colony's problems to restore the song?

## 2. Dependencies
- Base ECS system
- Colony Mood/Morale system
- Flora/Terrain grid system
- Area-of-effect buff/debuff system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_flora_emits_positive_chorus_on_high_colony_mood() {
        // Arrange
        let mut world = World::new();
        let mut app = App::new();
        // Set colony mood to high (e.g., 80)
        world.insert_resource(ColonyMood { current_mood: 80.0 });

        let flora = world.spawn((
            BioAcousticFlora,
            Position { x: 0, y: 0 },
        )).id();

        let pop = world.spawn((
            Pop,
            Position { x: 1, y: 0 },
            MoodModifier::default(),
        )).id();

        // Act
        app.add_systems(Update, bio_acoustic_chorus_system);
        app.update();

        // Assert
        let modifier = world.get::<MoodModifier>(pop).unwrap();
        assert!(modifier.value > 0.0, "Pop should receive a positive mood buff from the chorus");
    }

    #[test]
    fn test_flora_emits_negative_screech_on_low_colony_mood() {
        // Arrange
        let mut world = World::new();
        let mut app = App::new();
        // Set colony mood to low (e.g., 20)
        world.insert_resource(ColonyMood { current_mood: 20.0 });

        let flora = world.spawn((
            BioAcousticFlora,
            Position { x: 0, y: 0 },
        )).id();

        let pop = world.spawn((
            Pop,
            Position { x: 1, y: 0 },
            MoodModifier::default(),
        )).id();

        // Act
        app.add_systems(Update, bio_acoustic_chorus_system);
        app.update();

        // Assert
        let modifier = world.get::<MoodModifier>(pop).unwrap();
        assert!(modifier.value < 0.0, "Pop should receive a negative mood debuff from the screech");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct BioAcousticFlora;

#[derive(Resource)]
pub struct ColonyMood {
    pub current_mood: f32,
}

pub fn bio_acoustic_chorus_system(
    mood: Res<ColonyMood>,
    flora_query: Query<&Position, With<BioAcousticFlora>>,
    mut pop_query: Query<(&Position, &mut MoodModifier), With<Pop>>,
) {
    let buff_value = if mood.current_mood > 50.0 { 5.0 } else { -5.0 };

    for flora_pos in flora_query.iter() {
        for (pop_pos, mut mood_mod) in pop_query.iter_mut() {
            if flora_pos.distance(pop_pos) < 10.0 {
                mood_mod.value += buff_value;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Optimize the area-of-effect check to use a spatial grid or bounding volume hierarchy to avoid O(F*P) complexity.
- Parameterize the radii and mood thresholds.
- Smooth out the transition between buff and debuff based on exact colony mood instead of a hard threshold.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Colony mood properly influences surrounding pops through Bio-Acoustic Flora

## 7. Technical Guidance
- Implement `BioAcousticFlora` on a specific flora type when generated.
- Ensure the buff/debuff wears off appropriately when the Pop moves out of range or the colony mood stabilizes.

## 8. Questions
*Builder: add questions here if spec is unclear.*
