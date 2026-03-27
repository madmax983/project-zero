# Cognitive Overclocking

## 1. Overview
**Layer:** 1
**Fantasy:** Pushing the human mind beyond its limits to solve impossible problems, burning out the brightest sparks in the process.
**Mechanic:** An edict or specialized module that allows you to "overclock" the brains of specific Pops (usually researchers or engineers). Their work speed and learning rate increase by 500%, but they rapidly accumulate permanent "Neural Burnout" trauma, eventually leading to catatonia or unpredictable manic episodes.

## 2. Dependencies
- None specific, relies on base Pop mechanics (Needs, Utility AI).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_cognitive_overclocking_increases_work_speed() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_cognitive_overclocking_system);

        let entity = app.world.spawn((
            Pop,
            WorkStats { base_speed: 1.0, current_speed: 1.0 },
            CognitiveOverclock { active: true, time_active: 0.0 },
        )).id();

        // Act
        app.update();

        // Assert
        let work_stats = app.world.get::<WorkStats>(entity).unwrap();
        assert_eq!(work_stats.current_speed, 5.0); // 500% speed
    }

    #[test]
    fn test_cognitive_overclocking_accumulates_burnout() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(Time::new_with(())); // Mock time
        app.add_systems(Update, process_neural_burnout_system);

        let entity = app.world.spawn((
            Pop,
            CognitiveOverclock { active: true, time_active: 0.0 },
            NeuralTrauma { burnout_level: 0.0 },
        )).id();

        // Simulate some time passing
        app.world.get_mut::<CognitiveOverclock>(entity).unwrap().time_active = 10.0;

        // Act
        app.update();

        // Assert
        let trauma = app.world.get::<NeuralTrauma>(entity).unwrap();
        assert!(trauma.burnout_level > 0.0);
    }

    #[test]
    fn test_burnout_leads_to_catatonia() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, check_burnout_threshold_system);

        let entity = app.world.spawn((
            Pop,
            NeuralTrauma { burnout_level: 100.0 }, // Past threshold
            ActiveState::Working,
        )).id();

        // Act
        app.update();

        // Assert
        let state = app.world.get::<ActiveState>(entity).unwrap();
        assert_eq!(*state, ActiveState::Catatonic);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct WorkStats {
    pub base_speed: f32,
    pub current_speed: f32,
}

#[derive(Component)]
pub struct CognitiveOverclock {
    pub active: bool,
    pub time_active: f32,
}

#[derive(Component)]
pub struct NeuralTrauma {
    pub burnout_level: f32,
}

#[derive(Component, PartialEq, Debug)]
pub enum ActiveState {
    Working,
    Catatonic,
}

pub fn apply_cognitive_overclocking_system(
    mut query: Query<(&mut WorkStats, &CognitiveOverclock)>
) {
    for (mut stats, overclock) in query.iter_mut() {
        if overclock.active {
            stats.current_speed = stats.base_speed * 5.0;
        } else {
            stats.current_speed = stats.base_speed;
        }
    }
}

pub fn process_neural_burnout_system(
    time: Res<Time>,
    mut query: Query<(&mut NeuralTrauma, &mut CognitiveOverclock)>
) {
    for (mut trauma, mut overclock) in query.iter_mut() {
        if overclock.active {
            overclock.time_active += time.delta_seconds();
            trauma.burnout_level += 1.0 * time.delta_seconds(); // 1 burnout per second
        }
    }
}

pub fn check_burnout_threshold_system(
    mut query: Query<(&NeuralTrauma, &mut ActiveState)>
) {
    for (trauma, mut state) in query.iter_mut() {
        if trauma.burnout_level >= 100.0 {
            *state = ActiveState::Catatonic;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- `WorkStats` might already exist in a different form. The builder should integrate with the existing Pop needs and utility weights.
- `CognitiveOverclock` could be implemented as a buff/debuff or a toggleable state depending on how player edicts or modules are applied.
- Extract magical numbers (like `5.0` speed multiplier and `100.0` burnout threshold) into consts or component fields for easier balancing.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Overclocked pops complete work tasks 5x faster.
- [ ] Overclocked pops eventually fall into a Catatonic state if left overclocked too long.

## 7. Technical Guidance
- The mechanic should ideally be an action the player triggers on a specific building or zone, applying the component to the pops working there.
- Catatonia should interact with the Pop's utility AI, forcing their action evaluation to only yield "Do Nothing" or a specific "Catatonic" action.

## 8. Questions
*Builder: add questions here if spec is unclear.*
