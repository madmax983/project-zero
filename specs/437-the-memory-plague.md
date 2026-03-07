# 437: The Memory Plague

## Overview

A pathogen spreads through the colony, doing no physical damage but slowly erasing a Pop's "Memory" and "XP". Highly skilled veterans regress to novices. Curing it requires a complete quarantine, halting production.

## Dependencies

- `051` Pop Skills
- `036` Pop Memory
- `406` Quarantine Protocols

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, (memory_plague_transmission_system, memory_plague_effects_system));
        app
    }

    #[test]
    fn test_memory_plague_transmission() {
        let mut app = setup_test_app();

        // Setup two Pops near each other, one infected
        let infected_pop = app.world.spawn((PopBundle::default(), MemoryPlagueInfection { progress: 0.5 }, Transform::from_translation(Vec3::ZERO))).id();
        let uninfected_pop = app.world.spawn((PopBundle::default(), Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)))).id();

        app.update();

        // Check if uninfected pop caught the plague
        assert!(app.world.get::<MemoryPlagueInfection>(uninfected_pop).is_some(), "Uninfected Pop close to infected Pop should contract the Memory Plague");
    }

    #[test]
    fn test_memory_plague_erases_xp() {
        let mut app = setup_test_app();

        let mut skills = PopSkills::default();
        skills.set_skill("Surgery", 100.0);

        let pop = app.world.spawn((skills, MemoryPlagueInfection { progress: 0.1 })).id();

        // Advance time to allow delta_seconds to be non-zero
        let mut time = Time::default();
        time.advance_by(std::time::Duration::from_secs(1));
        app.world.insert_resource(time);

        app.update();

        // Skill should be reduced
        let skills = app.world.get::<PopSkills>(pop).unwrap();
        assert!(skills.get_skill("Surgery") < 100.0, "Infected Pop's skills should slowly decay");
    }

    #[test]
    fn test_quarantine_stops_transmission() {
        let mut app = setup_test_app();

        let infected_pop = app.world.spawn((PopBundle::default(), MemoryPlagueInfection { progress: 0.5 }, Quarantined, Transform::from_translation(Vec3::ZERO))).id();
        let uninfected_pop = app.world.spawn((PopBundle::default(), Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)))).id();

        app.update();

        // Check if uninfected pop caught the plague
        assert!(app.world.get::<MemoryPlagueInfection>(uninfected_pop).is_none(), "Uninfected Pop should not catch the plague from a Quarantined Pop");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct MemoryPlagueInfection {
    pub progress: f32, // 0.0 to 1.0
}

#[derive(Component)]
pub struct Quarantined; // Likely exists from 406-Quarantine Protocols

#[derive(Component, Default)]
pub struct PopSkills {
    skills: std::collections::HashMap<String, f32>,
}

impl PopSkills {
    pub fn set_skill(&mut self, name: &str, level: f32) {
        self.skills.insert(name.to_string(), level);
    }

    pub fn get_skill(&self, name: &str) -> f32 {
        *self.skills.get(name).unwrap_or(&0.0)
    }

    pub fn decay_skills(&mut self, rate: f32) {
        for val in self.skills.values_mut() {
            *val -= rate;
            if *val < 0.0 { *val = 0.0; }
        }
    }
}

pub fn memory_plague_transmission_system(
    mut commands: Commands,
    infected_query: Query<(Entity, &Transform), (With<MemoryPlagueInfection>, Without<Quarantined>)>,
    uninfected_query: Query<(Entity, &Transform), Without<MemoryPlagueInfection>>,
) {
    let transmission_radius = 5.0;

    for (uninf_entity, uninf_transform) in uninfected_query.iter() {
        for (inf_entity, inf_transform) in infected_query.iter() {
            if uninf_transform.translation.distance(inf_transform.translation) < transmission_radius {
                commands.entity(uninf_entity).insert(MemoryPlagueInfection { progress: 0.01 });
                break;
            }
        }
    }
}

pub fn memory_plague_effects_system(
    mut query: Query<(&mut PopSkills, &mut MemoryPlagueInfection)>,
    time: Res<Time>,
) {
    let decay_rate = 1.0 * time.delta_seconds(); // arbitrary rate for test

    for (mut skills, mut infection) in query.iter_mut() {
        skills.decay_skills(decay_rate * infection.progress);
        infection.progress += 0.01 * time.delta_seconds(); // plague worsens over time
    }
}
```

## REFACTOR Phase: Quality & Design

- Use an event-based approach for disease transmission `PlagueTransmissionEvent` instead of calculating O(N^2) distances each frame, or use spatial hashing for the queries.
- Add visual indicators for Pops suffering from the Memory Plague so players can identify outbreaks before the entire workforce forgets their jobs.
- The `Quarantined` status should be integrated thoroughly so infected Pops are unable to access their workplaces, forcing the player to choose between economic damage and the loss of skill.

## Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Memory Plague decays Pop Skills.
- [ ] Disease transmits between un-quarantined Pops in close proximity.
- [ ] Quarantined Pops cannot transmit the disease.

## Technical Guidance

- Pay attention to `PopSkills` and ensure the decay logic affects all stored skills correctly without deleting the keys entirely.
- Ensure the spatial queries in the transmission system are optimized to prevent performance drops during large population outbreaks.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
