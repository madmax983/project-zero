# 584: The Sub-Zero Empathy

## 1. Overview
An extreme, unnatural planetary weather front drops temperatures so low it chemically inhibits the brain's empathy centers in unprotected Pops. Affected Pops temporarily lose all relationship bonds, ignore the sick, and ruthlessly optimize for personal survival.

## 2. Dependencies
- Layer 1: Weather/Climate System
- Layer 1: Pop Relationships & Empathy/Social Systems
- Layer 1: Job Priorities (Medical, Rescue)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_extreme_cold_applies_empathy_inhibition() {
        let mut app = App::new();
        app.add_systems(Update, apply_subzero_empathy_system);

        app.insert_resource(PlanetaryWeather { temperature: -80.0 });

        let pop = app.world_mut().spawn((Pop, ThermalProtection { level: 10.0 })).id();

        app.update();

        // Protection is insufficient, should get EmpathyInhibited
        assert!(app.world().get::<EmpathyInhibited>(pop).is_some());
    }

    #[test]
    fn test_inhibited_pops_ignore_medical_jobs() {
        let mut app = App::new();
        app.add_systems(Update, filter_jobs_by_empathy_system);

        let pop = app.world_mut().spawn((
            Pop,
            EmpathyInhibited,
            JobQueue(vec![JobType::Medical, JobType::PowerMaintenance])
        )).id();

        app.update();

        let queue = app.world().get::<JobQueue>(pop).unwrap();
        // Medical job should be removed
        assert!(!queue.0.contains(&JobType::Medical));
        assert!(queue.0.contains(&JobType::PowerMaintenance));
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Resource)]
pub struct PlanetaryWeather {
    pub temperature: f32,
}

#[derive(Component)]
pub struct ThermalProtection {
    pub level: f32,
}

#[derive(Component)]
pub struct EmpathyInhibited;

#[derive(Component)]
pub struct JobQueue(pub Vec<JobType>);

#[derive(PartialEq)]
pub enum JobType {
    Medical,
    PowerMaintenance,
    Hauling,
}

pub fn apply_subzero_empathy_system(
    mut commands: Commands,
    weather: Res<PlanetaryWeather>,
    query: Query<(Entity, &ThermalProtection), With<Pop>>,
) {
    if weather.temperature < -50.0 {
        for (entity, protection) in query.iter() {
            if protection.level < 50.0 {
                commands.entity(entity).insert(EmpathyInhibited);
            }
        }
    } else {
        // Remove when temperature rises
        // ... implementation omitted for brevity
    }
}

pub fn filter_jobs_by_empathy_system(
    mut query: Query<&mut JobQueue, With<EmpathyInhibited>>,
) {
    for mut queue in query.iter_mut() {
        queue.0.retain(|job| *job != JobType::Medical);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Graceful Degradation:** Instead of completely clearing medical jobs, heavily penalize their priority score.
- **Relationship Suppression:** Temporarily mask relationship bonuses/penalties during interaction calculations while inhibited.
- **Visuals:** Add a visual indicator (e.g., a frozen heart icon) to affected Pops.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified (Low temp + low protection = inhibited; Inhibited drops altruistic jobs)

## 7. Technical Guidance
- Do not permanently delete relationship data. Use the `EmpathyInhibited` component as a filter/modifier during calculations.
- Ensure Pops actively prioritize `PowerMaintenance` or `HeaterRepair` to represent "optimizing for personal survival."

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
