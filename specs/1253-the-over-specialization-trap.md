# 1253: The Over-Specialization Trap

## 1. Overview
Perfect division of labor leads to highly skilled but fundamentally helpless individuals. Pops assigned to the same job for decades gain massive efficiency bonuses but permanently lose "Generalist" capabilities, making them unable to perform basic maintenance or medical care natively.

## 2. Dependencies
- Pop `Job` and `JobTenure` systems in `src/layer1/specialization.rs`
- Traits system (`Trait` and `get_job_efficiency_modifier`)
- Assignment checks

## 3. RED Phase: Tests First
```rust
#[test]
fn test_pop_gains_hyper_specialization_trait() {
    let mut world = World::new();

    // Setup Pop with Job and Tenure
    let pop_ent = world.spawn((
        Pop,
        Job { workplace: Entity::PLACEHOLDER, job_type: AssignmentType::FarmWorker },
        JobTenure::with_ticks(AssignmentType::FarmWorker, 49999), // Just below threshold
        Traits::default()
    )).id();

    // Run tenure system
    let mut schedule = Schedule::default();
    schedule.add_systems(update_hyper_specialization_system);
    schedule.run(&mut world);

    let traits = world.get::<Traits>(pop_ent).unwrap();
    assert!(traits.has(Trait::HyperSpecializedFarmWorker));
}

#[test]
fn test_hyper_specialized_pop_efficiency() {
    let mut traits = Traits::default();
    traits.add(Trait::HyperSpecializedFarmWorker);

    // Efficiency should be massive for their specialty
    let bonus = get_job_efficiency_modifier(&traits, AssignmentType::FarmWorker);
    assert!(bonus >= 2.0);

    // Efficiency should be terrible for generalist tasks
    let penalty = get_job_efficiency_modifier(&traits, AssignmentType::LibraryWorker);
    assert!(penalty <= 0.1);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy_ecs::prelude::*;
use crate::layer1::specialization::JobTenure;
use crate::layer1::psychology::traits::{Traits, Trait};
use crate::layer1::utility_types::AssignmentType;

pub const HYPER_MUTATION_THRESHOLD: u64 = 50000;

pub fn update_hyper_specialization_system(
    mut query: Query<(&JobTenure, &mut Traits), Changed<JobTenure>>
) {
    for (tenure, mut traits) in query.iter_mut() {
        if tenure.get_ticks(AssignmentType::FarmWorker) >= HYPER_MUTATION_THRESHOLD && !traits.has(Trait::HyperSpecializedFarmWorker) {
            traits.add(Trait::HyperSpecializedFarmWorker);
        }
        if tenure.get_ticks(AssignmentType::Administrator) >= HYPER_MUTATION_THRESHOLD && !traits.has(Trait::HyperSpecializedAdministrator) {
            traits.add(Trait::HyperSpecializedAdministrator);
        }
    }
}

// In src/layer1/psychology/traits.rs modify get_job_efficiency_modifier
/*
if traits.has(Trait::HyperSpecializedFarmWorker) {
    if job == AssignmentType::FarmWorker {
        multiplier *= 2.0;
    } else {
        multiplier *= 0.1;
    }
}
*/
```

## 5. REFACTOR Phase: Quality & Design
- Add `HyperSpecialized` variants for all major `AssignmentType` variants.
- Emit an event when a Pop becomes hyper-specialized for UI notification.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Long-tenure Pops get massive output bonuses via a new `Trait`.
- [ ] Long-tenure Pops suffer severe penalties to out-of-domain assignments.

## 7. Technical Guidance
- Update the `Trait` enum in `src/layer1/psychology/traits.rs` to include the new hyper-specialization variants.
- Modify `get_job_efficiency_modifier` to handle these new traits.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect:* Implement the minimal viable feature to satisfy tests. Advanced interactions will be added in subsequent specs.
