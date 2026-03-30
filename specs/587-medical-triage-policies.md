# 587: Medical Triage Policies

## 1. Overview
**Layer:** 1
**Fantasy:** Making the hard, cold calculus of survival when resources are scarce.
**Mechanic:** A policy setting for Hospitals: "Save Everyone", "Workers First", "Soldiers First", "Leaders Only". Doctors prioritize patients based on this policy when beds/medicine are limited.

## 2. Dependencies
- Layer 1 Health/Sickness System
- Layer 1 Jobs (Doctor)
- Layer 1 Roles (Worker, Soldier, Leader)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_triage_workers_first_policy() {
        let mut app = App::new();
        app.insert_resource(TriagePolicy(TriageType::WorkersFirst));

        let leader = app.world_mut().spawn((Pop::default(), Role::Leader, NeedsCare)).id();
        let worker = app.world_mut().spawn((Pop::default(), Role::Worker, NeedsCare)).id();
        let hospital = app.world_mut().spawn((Hospital { beds_available: 1 }, PatientQueue(vec![leader, worker]))).id();

        app.add_systems(Update, admit_patients_system);
        app.update();

        let h = app.world().get::<PatientQueue>(hospital).unwrap();
        // Worker should be admitted first, leaving the leader in the queue
        assert_eq!(h.0.len(), 1);
        assert_eq!(h.0[0], leader);
    }

    #[test]
    fn test_triage_soldiers_first_policy() {
        let mut app = App::new();
        app.insert_resource(TriagePolicy(TriageType::SoldiersFirst));

        let worker = app.world_mut().spawn((Pop::default(), Role::Worker, NeedsCare)).id();
        let soldier = app.world_mut().spawn((Pop::default(), Role::Soldier, NeedsCare)).id();
        let hospital = app.world_mut().spawn((Hospital { beds_available: 1 }, PatientQueue(vec![worker, soldier]))).id();

        app.add_systems(Update, admit_patients_system);
        app.update();

        let h = app.world().get::<PatientQueue>(hospital).unwrap();
        assert_eq!(h.0.len(), 1);
        assert_eq!(h.0[0], worker);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Resource)]
pub struct TriagePolicy(pub TriageType);

#[derive(PartialEq, Clone, Copy)]
pub enum TriageType {
    SaveEveryone,
    WorkersFirst,
    SoldiersFirst,
    LeadersOnly,
}

#[derive(Component)]
pub struct PatientQueue(pub Vec<Entity>);

#[derive(Component, PartialEq, Clone, Copy)]
pub enum Role { Worker, Soldier, Leader }

pub fn admit_patients_system(
    policy: Res<TriagePolicy>,
    mut hospitals: Query<(&mut Hospital, &mut PatientQueue)>,
    roles: Query<&Role>,
) {
    for (mut hospital, mut queue) in hospitals.iter_mut() {
        if hospital.beds_available > 0 {
            queue.0.sort_by(|a, b| {
                let role_a = roles.get(*a).unwrap_or(&Role::Worker);
                let role_b = roles.get(*b).unwrap_or(&Role::Worker);
                match policy.0 {
                    TriageType::WorkersFirst => {
                        if *role_a == Role::Worker { std::cmp::Ordering::Less }
                        else if *role_b == Role::Worker { std::cmp::Ordering::Greater }
                        else { std::cmp::Ordering::Equal }
                    },
                    TriageType::SoldiersFirst => {
                        if *role_a == Role::Soldier { std::cmp::Ordering::Less }
                        else if *role_b == Role::Soldier { std::cmp::Ordering::Greater }
                        else { std::cmp::Ordering::Equal }
                    },
                    _ => std::cmp::Ordering::Equal,
                }
            });
            // Admit the top patient
            if !queue.0.is_empty() {
                queue.0.remove(0);
                hospital.beds_available -= 1;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** The sorting logic inside the system is bulky. It should be extracted into a `TriagePolicy::score(entity)` method to keep the system clean.
- **CodeSmells:** Hard panics on `unwrap_or` if an entity dies in queue. Need to validate entity existence before sorting.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Hospitals correctly admit patients according to the active `TriagePolicy` priority.

## 7. Technical Guidance
- Ensure that implementing "Leaders Only" actively ejects non-leaders from beds if a leader needs one and the hospital is full (requires expanding the `PatientQueue` to track admitted patients).
- Add a morale penalty to the colony when a patient is denied a bed due to policy, simulating the "cold calculus".

## 8. Questions
*Builder: add questions here if spec is unclear.*
