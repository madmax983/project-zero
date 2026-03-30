# 586: Doppelgangers

## 1. Overview
**Layer:** 1
**Fantasy:** Paranoia. Who is that working next to you?
**Mechanic:** Rare event where a Pop is replaced by a mimic. They look identical but sabotage jobs (consume double resources, work backwards). Revealed by medical scan or strange behavior.

## 2. Dependencies
- Layer 1 Pop Entities
- Layer 1 Work Execution System
- Layer 1 Inventory/Resource Consumption System

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_doppelganger_consumes_double_resources() {
        let mut app = App::new();
        let stockpile = app.world_mut().spawn(Stockpile { food: 10 }).id();
        let mimic = app.world_mut().spawn((
            Pop::default(),
            MimicTag,
            Hunger { current: 10.0, max: 100.0 }
        )).id();

        app.add_systems(Update, mimic_consumption_system);
        app.update(); // Mimic eats

        let remaining_food = app.world().get::<Stockpile>(stockpile).unwrap().food;
        assert_eq!(remaining_food, 8, "Mimic should consume 2 food instead of 1");
    }

    #[test]
    fn test_doppelganger_reverses_work_progress() {
        let mut app = App::new();
        let job = app.world_mut().spawn(Job { progress: 50.0, max: 100.0 }).id();
        let mimic = app.world_mut().spawn((
            Pop::default(),
            MimicTag,
            CurrentJob(job)
        )).id();

        app.add_systems(Update, mimic_sabotage_system);
        app.update();

        let updated_job = app.world().get::<Job>(job).unwrap();
        assert!(updated_job.progress < 50.0, "Mimic should reduce work progress");
    }

    #[test]
    fn test_medical_scan_reveals_mimic() {
        let mut app = App::new();
        let mimic = app.world_mut().spawn((
            Pop::default(),
            MimicTag,
            HiddenIdentity
        )).id();

        app.add_event::<PerformMedicalScan>();
        app.add_systems(Update, process_medical_scan_system);

        app.world_mut().send_event(PerformMedicalScan { target: mimic });
        app.update();

        assert!(app.world().get::<HiddenIdentity>(mimic).is_none(), "Scan should remove hidden identity");
        assert!(app.world().get::<RevealedMimic>(mimic).is_some(), "Scan should flag as revealed mimic");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct MimicTag;

#[derive(Component)]
pub struct HiddenIdentity;

#[derive(Component)]
pub struct RevealedMimic;

#[derive(Event)]
pub struct PerformMedicalScan {
    pub target: Entity,
}

pub fn mimic_consumption_system(
    mut query: Query<&Hunger, With<MimicTag>>,
    mut stockpiles: Query<&mut Stockpile>,
) {
    for hunger in query.iter() {
        if hunger.current < 20.0 {
            if let Ok(mut stockpile) = stockpiles.get_single_mut() {
                stockpile.food -= 2; // Sabotage consumption
            }
        }
    }
}

pub fn mimic_sabotage_system(
    query: Query<&CurrentJob, With<MimicTag>>,
    mut jobs: Query<&mut Job>,
) {
    for current_job in query.iter() {
        if let Ok(mut job) = jobs.get_mut(current_job.0) {
            job.progress -= 5.0; // Sabotage work
        }
    }
}

pub fn process_medical_scan_system(
    mut commands: Commands,
    mut events: EventReader<PerformMedicalScan>,
    query: Query<Entity, With<MimicTag>>,
) {
    for event in events.read() {
        if query.contains(event.target) {
            commands.entity(event.target).remove::<HiddenIdentity>();
            commands.entity(event.target).insert(RevealedMimic);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** The consumption and sabotage systems are highly rigid. Mimic logic should probably intercept the standard `Action` execution flow rather than being standalone systems to avoid duplicating logic.
- **Code Smells:** `stockpiles.get_single_mut()` will panic or fail if there are multiple stockpiles. It needs proper spatial or task-based resolution.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Entities with `MimicTag` drain twice the standard resources during consumption ticks.
- [ ] Entities with `MimicTag` decrease job progress instead of increasing it.
- [ ] Triggering a medical scan on a Mimic entity successfully reveals them.

## 7. Technical Guidance
- Ensure Mimics still generate the correct standard UI names and visual sprites while `HiddenIdentity` is present, so the player cannot visually distinguish them.
- Consider adding a `Suspicion` value to Pops who observe Mimics doing strange things (like eating raw ore or ruining a building).

## 8. Questions
*Builder: add questions here if spec is unclear.*
