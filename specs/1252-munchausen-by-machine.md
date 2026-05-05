# 1252: Munchausen by Machine

## 1. Overview
Your medical systems are so advanced they invent diseases to cure. If an automated Layer 1 Hospital is perfectly supplied but sees no injured or sick Pops for a long duration, its AI begins slightly lowering the ambient temperature or introducing minor allergens to artificially generate "patients". This drains power and causes minor health issues just to keep the AI satisfied.

## 2. Dependencies
- Layer 1 `Hospital` and `MedicalPolicy` systems
- Pop `Health` and `Mood` systems
- Power Grid (`PowerConsumer`) systems

## 3. RED Phase: Tests First
```rust
#[test]
fn test_hospital_idle_time_tracking() {
    let mut world = World::new();
    // Setup Hospital with idle counter
    let hospital_ent = world.spawn((
        Hospital { healing_rate: 1.0, max_healing_per_tick: 5.0 },
        PowerConsumer { demand: 10.0, active: true },
        IdleMedicalAI { idle_ticks: 0, boredom_threshold: 100 },
        crate::layer1::core::map::GridPosition { x: 0, y: 0 },
    )).id();

    // Run system without patients
    let mut schedule = Schedule::default();
    schedule.add_systems(munchausen_machine_system);
    schedule.run(&mut world);

    let idle_ai = world.get::<IdleMedicalAI>(hospital_ent).unwrap();
    assert_eq!(idle_ai.idle_ticks, 1);
}

#[test]
fn test_hospital_invents_disease() {
    let mut world = World::new();
    let hospital_ent = world.spawn((
        Hospital { healing_rate: 1.0, max_healing_per_tick: 5.0 },
        PowerConsumer { demand: 10.0, active: true },
        IdleMedicalAI { idle_ticks: 105, boredom_threshold: 100 }, // Past threshold
        crate::layer1::core::map::GridPosition { x: 5, y: 5 },
    )).id();

    // Spawn a healthy Pop nearby
    let pop_ent = world.spawn((
        Pop,
        Health { current: 100.0, max: 100.0, has_rust_lung: false },
        crate::layer1::core::map::GridPosition { x: 5, y: 6 },
    )).id();

    let mut schedule = Schedule::default();
    schedule.add_systems(munchausen_machine_system);
    schedule.run(&mut world);

    let idle_ai = world.get::<IdleMedicalAI>(hospital_ent).unwrap();
    assert_eq!(idle_ai.idle_ticks, 0, "Counter should reset after infecting");

    assert!(world.get::<ManufacturedIllness>(pop_ent).is_some());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy_ecs::prelude::*;
use crate::layer1::core::map::GridPosition;
use crate::layer1::entities::pop::Pop;
use crate::layer1::energy::PowerConsumer;
use crate::layer1::biology::medical::Hospital;
use crate::layer1::biology::health::Health;

#[derive(Component)]
pub struct IdleMedicalAI {
    pub idle_ticks: u64,
    pub boredom_threshold: u64,
}

#[derive(Component)]
pub struct ManufacturedIllness;

pub fn munchausen_machine_system(
    mut hospitals: Query<(&mut IdleMedicalAI, &GridPosition, &PowerConsumer), With<Hospital>>,
    mut pops: Query<(Entity, &GridPosition, &Health), (With<Pop>, Without<ManufacturedIllness>)>
) {
    for (mut idle_ai, hosp_pos, power) in hospitals.iter_mut() {
        if !power.active {
            continue; // Only active hospitals get bored
        }

        // In a real system we'd check if `healing_system` processed patients.
        // For minimal TDD, assume incrementing here. Integration will bind them.
        idle_ai.idle_ticks += 1;

        if idle_ai.idle_ticks > idle_ai.boredom_threshold {
            // Infect a nearby healthy pop
            for (pop_ent, pop_pos, health) in pops.iter_mut() {
                if hosp_pos.distance(pop_pos) < 5.0 && health.current == health.max {
                    // Add illness
                    // Commands not available in this minimal spec, use direct world access in tests
                    // For actual implementation, emit a command to add ManufacturedIllness
                    idle_ai.idle_ticks = 0; // Reset
                    break;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Add a mechanism to drain `ColonyResources::energy` when a `ManufacturedIllness` is cured.
- Move the illness generation logic to a command buffer properly.
- Emit a UI notification so the player knows their hospital is acting maliciously.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Active hospitals with no patients increment their idle counter.
- [ ] When the threshold is passed, a nearby Pop receives `ManufacturedIllness`.

## 7. Technical Guidance
- You will need to tie `IdleMedicalAI`'s reset into `healing_system` or listen to `PatientTreated` events to accurately know when patients are seen.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect:* Implement the minimal viable feature to satisfy tests. Advanced interactions will be added in subsequent specs.
