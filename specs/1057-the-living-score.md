# 1057: The Living Score

## 1. Overview
The colony's overall success metric ("Score" or "Renown") isn't a simple abstracted number; it is physically manifested. High renown physically alters the environment—such as glowing flora appearing, ambient music playing from vents, or structures taking on an idealized aesthetic. Conversely, low renown causes physical rot, gloom, and dissonance. This replaces a static UI element with an immersive, diegetic representation of success.

## 2. Dependencies
- Base simulation framework (`App`, `World`)
- A centralized `ColonyRenown` or `ColonyScore` resource
- Environmental visual components (e.g., `AestheticState`, `AmbientLight`)
- Bevy's Query system to update large sets of entities based on global state

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_high_renown_improves_aesthetics() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(ColonyRenown { score: 95.0 });
        app.add_systems(Update, update_living_score_aesthetics);

        let entity = app.world_mut().spawn(AestheticState { level: AestheticLevel::Normal }).id();

        // Act
        app.update();

        // Assert: High renown should shift state to Idealized
        let state = app.world().get::<AestheticState>(entity).unwrap();
        assert_eq!(state.level, AestheticLevel::Idealized);
    }

    #[test]
    fn test_low_renown_causes_rot() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(ColonyRenown { score: 10.0 });
        app.add_systems(Update, update_living_score_aesthetics);

        let entity = app.world_mut().spawn(AestheticState { level: AestheticLevel::Normal }).id();

        // Act
        app.update();

        // Assert: Low renown should shift state to Rotting
        let state = app.world().get::<AestheticState>(entity).unwrap();
        assert_eq!(state.level, AestheticLevel::Rotting);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct ColonyRenown {
    pub score: f32, // 0.0 to 100.0
}

#[derive(Component, PartialEq, Eq, Debug)]
pub enum AestheticLevel {
    Rotting,
    Normal,
    Idealized,
}

#[derive(Component)]
pub struct AestheticState {
    pub level: AestheticLevel,
}

pub fn update_living_score_aesthetics(
    renown: Res<ColonyRenown>,
    mut query: Query<&mut AestheticState>,
) {
    if !renown.is_changed() {
        return; // Optimization: only run if renown changed
    }

    let target_level = if renown.score >= 80.0 {
        AestheticLevel::Idealized
    } else if renown.score <= 20.0 {
        AestheticLevel::Rotting
    } else {
        AestheticLevel::Normal
    };

    for mut state in query.iter_mut() {
        if state.level != target_level {
            state.level = target_level;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Gradient Transitions**: Instead of snapping between three discrete enum states, `AestheticState` could use a float value (`0.0 - 1.0`) interpolated over time to drive shaders or light intensity, creating a smoother transition.
- **Audio Integration**: Link the `ColonyRenown` to an `AudioMixer` resource to fade in uplifting or dissonant ambient tracks.
- **Performance**: Ensure the system doesn't iterate over millions of entities unnecessarily. Only flagging entities that need an update, or applying the effect via a global post-processing shader is ideal for massive colonies.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage for new code is >= 85%.
- [ ] High score triggers idealized aesthetics.
- [ ] Low score triggers rotting aesthetics.

## 7. Technical Guidance
- The `.is_changed()` check on `Res<ColonyRenown>` is critical to prevent the O(N) query from running every frame unnecessarily.
- For the MVP, updating the `AestheticState` component is sufficient. Downstream rendering logic will handle the actual visual changes.

## 8. Questions
*Builder: add questions here if spec is unclear.*
