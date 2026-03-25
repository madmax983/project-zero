# 587: Medical Triage Policies

## 1. Overview
Making the hard, cold calculus of survival when resources are scarce. A policy setting for Hospitals: "Save Everyone", "Workers First", "Soldiers First", "Leaders Only". Doctors prioritize patients based on this policy when beds/medicine are limited. Tension between utilitarian efficiency vs. Moral equality. Layer: 1.

## 2. Dependencies
- Hospital and Bed systems
- Pop roles/classes (Worker, Soldier, Leader)
- Policy management system

## 3. RED Phase: Tests First
```rust
#[test]
fn test_triage_policy_prioritizes_workers() {
    let mut world = World::new();
    world.insert_resource(TriagePolicy(PolicyType::WorkersFirst));

    let worker = world.spawn((Pop, Role::Worker, Health { current: 10.0 })).id();
    let leader = world.spawn((Pop, Role::Leader, Health { current: 10.0 })).id();

    let hospital = world.spawn(Hospital { available_beds: 1 }).id();

    hospital_admission_system(&mut world);

    assert!(world.get::<Admitted>(worker).is_some());
    assert!(world.get::<Admitted>(leader).is_none());
}

#[test]
fn test_triage_policy_save_everyone_fifo() {
    let mut world = World::new();
    world.insert_resource(TriagePolicy(PolicyType::SaveEveryone));

    let p1 = world.spawn((Pop, Health { current: 10.0 }, ArrivalTime(1))).id();
    let p2 = world.spawn((Pop, Health { current: 10.0 }, ArrivalTime(2))).id();

    let hospital = world.spawn(Hospital { available_beds: 1 }).id();

    hospital_admission_system(&mut world);

    assert!(world.get::<Admitted>(p1).is_some());
    assert!(world.get::<Admitted>(p2).is_none());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
pub enum PolicyType { SaveEveryone, WorkersFirst, SoldiersFirst, LeadersOnly }
pub struct TriagePolicy(pub PolicyType);
pub fn hospital_admission_system(world: &mut World) { ... }
```

## 5. REFACTOR Phase: Quality & Design
- Make hospital admission a reusable generic function taking a priority closure based on the policy.
- Consider what happens when a higher-priority patient arrives while a lower-priority one occupies a bed.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: policies correctly dictate bed assignment during resource shortages.

## 7. Technical Guidance
- Admission should probably sort the queue of sick pops based on the policy and then assign beds up to the limit.

## 8. Questions
*Builder: add questions here if spec is unclear.*
