# Specification 490: The Echo of the Void

## 1. Overview
This feature introduces "The Echo", a psychological condition that affects Layer 1 Pops after prolonged exposure to deep space operations in Layer 2 (e.g., long-haul mining routes or stationary platforms). Upon returning, infected Pops begin hearing voices, suffer from chronic insomnia, and exhibit a strange, compulsive need to align objects to face the galaxy's center. If enough Pops have The Echo, a new, uncontrollable cult forms, demanding the colony turn off localized gravity and drift free, creating a tension between the immense profit of deep space operations and the psychological health of the workforce.

## 2. Dependencies
- `042` Energy System (For power grid and shield interactions)
- `152` Orbital Stations / Layer 2 Fleet Mechanics (To track time in deep space)
- `358` Stress Breakdowns / Pop Mental Health Systems

## 3. RED Phase: Tests First

```rust
// tests/echo_of_the_void_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use scale::layer1::pop::{Pop, Stress};
    use scale::layer2::fleet::{Fleet, MissionDuration};

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<EchoCultState>();
        // Add minimal required systems and resources
        world
    }

    #[test]
    fn test_echo_infection_after_prolonged_exposure() {
        let mut world = setup_world();

        let pop_entity = world.spawn((
            Pop,
            MissionDuration { days_in_deep_space: 40 }, // Above threshold
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_echo_infection_system);
        schedule.run(&mut world);

        assert!(world.get::<TheEchoInfection>(pop_entity).is_some());
    }

    #[test]
    fn test_echo_symptoms_stress_and_insomnia() {
        let mut world = setup_world();

        let pop_entity = world.spawn((
            Pop,
            TheEchoInfection { severity: 1.0 },
            Stress { current: 10.0, max: 100.0 },
            Rest { current: 100.0, max: 100.0 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_echo_symptoms_system);
        schedule.run(&mut world);

        let stress = world.get::<Stress>(pop_entity).unwrap();
        let rest = world.get::<Rest>(pop_entity).unwrap();

        assert!(stress.current > 10.0); // Stress increased
        assert!(rest.current < 100.0); // Rest decreased (Insomnia)
    }

    #[test]
    fn test_cult_formation_threshold() {
        let mut world = setup_world();

        // Spawn multiple infected pops
        for _ in 0..15 {
            world.spawn((Pop, TheEchoInfection { severity: 2.0 }));
        }

        let mut schedule = Schedule::default();
        schedule.add_systems(check_echo_cult_formation_system);
        schedule.run(&mut world);

        let cult_state = world.get_resource::<EchoCultState>().unwrap();
        assert!(cult_state.is_active);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/tech/echo_of_the_void.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::{Pop, Stress};

#[derive(Component, Debug, Clone)]
pub struct MissionDuration {
    pub days_in_deep_space: u32,
}

#[derive(Component, Debug, Clone)]
pub struct TheEchoInfection {
    pub severity: f32,
}

#[derive(Component, Debug, Clone)]
pub struct Rest {
    pub current: f32,
    pub max: f32,
}

#[derive(Resource, Default)]
pub struct EchoCultState {
    pub is_active: bool,
    pub infected_count: u32,
}

pub fn apply_echo_infection_system(
    mut commands: Commands,
    query: Query<(Entity, &MissionDuration), (With<Pop>, Without<TheEchoInfection>)>,
) {
    for (entity, duration) in query.iter() {
        if duration.days_in_deep_space > 30 {
            commands.entity(entity).insert(TheEchoInfection { severity: 1.0 });
        }
    }
}

pub fn process_echo_symptoms_system(
    mut query: Query<(&TheEchoInfection, &mut Stress, &mut Rest)>,
) {
    for (infection, mut stress, mut rest) in query.iter_mut() {
        stress.current = (stress.current + (5.0 * infection.severity)).min(stress.max);
        rest.current = (rest.current - (10.0 * infection.severity)).max(0.0);
    }
}

pub fn check_echo_cult_formation_system(
    query: Query<&TheEchoInfection>,
    mut cult_state: ResMut<EchoCultState>,
) {
    let count = query.iter().count() as u32;
    cult_state.infected_count = count;

    if count >= 10 && !cult_state.is_active {
        cult_state.is_active = true;
        // In full implementation, this would trigger an event for sabotage, etc.
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Sabotage Events**: Instead of just tracking `is_active`, the cult should periodically emit `SabotageEvent`s that specifically target gravity plating and planetary shields.
- **Visual Feedback**: Add a subtle shader effect or particle system around Pops with `TheEchoInfection` to indicate their condition to the player.
- **Progressive Severity**: The severity of the infection should increase over time if not treated or if the pop returns to deep space.
- **Cure/Treatment**: Introduce an advanced medical facility or research project that can slowly reduce infection severity.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/tech/echo_of_the_void.rs`.
- [ ] Pops returning from long missions correctly receive `TheEchoInfection`.
- [ ] Infected Pops suffer increased stress and decreased rest.
- [ ] Cult state activates when sufficient Pops are infected.

## 7. Technical Guidance
- Integration with Layer 2 requires careful tracking of when a Pop goes onto a ship and when they return to the colony. Use events or state transitions for this.
- The cult behavior shouldn't instantly destroy the colony; start with minor disruptions and escalate as `infected_count` grows.
- Use `Option<&TheEchoInfection>` or `Has<TheEchoInfection>` in existing query logic if needed, rather than heavy system ordering.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
