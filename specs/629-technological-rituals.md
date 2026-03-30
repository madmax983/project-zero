# 629: Technological Rituals

## 1. Overview
Over time, as a colony ages, its machines develop "Quirks." Instead of logical maintenance, Pops develop "Rituals" (kicking, chanting) to keep them running. Failure to perform these specific, non-sensical rituals reduces machine efficiency or causes breakdowns. This forces players to dedicate specific specialists just to appease the "machine spirit," creating tension between proper repair (expensive) and ritual maintenance (time-consuming).

## 2. Dependencies
- `009` Job System
- `112` Maintenance Debt
- `030` Tool Economy

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_machine_develops_quirk_over_time() {
    // Arrange: Setup test data
    let mut app = App::new();
    let machine = app.world.spawn((Machine { age: 100 }, Efficiency(100))).id();

    // Act: Call the feature
    age_machines(&mut app.world);

    // Assert: Verify expected behavior
    assert!(app.world.get::<Quirk>(machine).is_some());
}

#[test]
fn test_unperformed_ritual_lowers_efficiency() {
    // Test boundary conditions
    let mut app = App::new();
    let machine = app.world.spawn((Machine { age: 110 }, Quirk::HardStart, Efficiency(100))).id();

    // Act
    tick_rituals(&mut app.world); // No pop performed ritual

    // Assert
    let efficiency = app.world.get::<Efficiency>(machine).unwrap();
    assert!(efficiency.0 < 100);
}

#[test]
fn test_performed_ritual_restores_efficiency() {
    // Test boundary conditions
    let mut app = App::new();
    let machine = app.world.spawn((Machine { age: 110 }, Quirk::HardStart, Efficiency(50))).id();
    let pop = app.world.spawn((Pop, RitualPerformer { target: machine })).id();

    // Act
    perform_ritual(&mut app.world, pop, machine);

    // Assert
    let efficiency = app.world.get::<Efficiency>(machine).unwrap();
    assert_eq!(efficiency.0, 100);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN

pub struct Machine { pub age: u32 }
pub enum Quirk { HardStart, SpontaneousVent }
pub struct Efficiency(pub u32);
pub struct Pop;
pub struct RitualPerformer { pub target: Entity }

pub fn age_machines(world: &mut World) {
    let mut query = world.query::<(Entity, &mut Machine)>();
    let mut to_quirk = vec![];
    for (entity, mut machine) in query.iter_mut(world) {
        machine.age += 1;
        if machine.age > 100 {
            to_quirk.push(entity);
        }
    }
    for e in to_quirk {
        world.entity_mut(e).insert(Quirk::HardStart);
    }
}

pub fn tick_rituals(world: &mut World) {
    let mut query = world.query::<&mut Efficiency, With<Quirk>>();
    for mut eff in query.iter_mut(world) {
        if eff.0 > 10 {
            eff.0 -= 10;
        }
    }
}

pub fn perform_ritual(world: &mut World, pop: Entity, target: Entity) {
    let performer = world.get::<RitualPerformer>(pop).unwrap();
    if performer.target == target {
        let mut eff = world.get_mut::<Efficiency>(target).unwrap();
        eff.0 = 100;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** Do not iterate over all machines; use an event `MachineAgedEvent` instead. Replace direct modification with an `AppeaseQuirkEvent`.
- **Code Smells:** `Quirk::HardStart` shouldn't be the only result. Use a randomized weighted generation table based on the machine type.
- **Design:** Rituals shouldn't just be "click button." Pops should physically pathfind, animate, and spend time (e.g., waiting 5 seconds while chanting). It should use the `ActionType` utility AI.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- `Quirk` should require a specific `JobType::Ritualist` to appease efficiently.
- When `Maintenance Debt` hits zero, a `Quirk` should also be a potential consequence instead of just breaking down.
- Properly fixing the machine with a repair tool should clear the `Quirk` entirely.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
