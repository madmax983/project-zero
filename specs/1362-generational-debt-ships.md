# 1362: Generational Debt Ships

## Overview

When desperate for expansion, players can launch "Debt Ships"—cheap colony vessels funded by mega-corporations. The resulting colony starts with a massive resource deficit owed to the corporation, automatically siphoning a percentage of its output back to the core worlds. A frontier world paying off its centuries-long debt triggers a massive economic boom but also a fierce independence movement driven by generations of resentment.

## Dependencies

- None

## RED Phase: Tests First

```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_debt_ship_colony_starts_with_deficit() {
    let mut app = setup_test_app();
    let colony_id = launch_debt_ship(&mut app);

    let debt = app.world().get::<CorporateDebt>(colony_id).unwrap();
    assert!(debt.amount > 10000.0);
    assert_eq!(debt.siphon_percentage, 0.15); // 15% siphon
}

#[test]
fn test_debt_siphon_reduces_colony_output() {
    let mut app = setup_test_app();
    let colony_id = setup_colony_with_debt(&mut app, 10000.0, 0.15);

    // Simulate resource production
    produce_resources(&mut app, colony_id, 100.0);
    app.update();

    let stockpile = app.world().get::<ResourceStockpile>(colony_id).unwrap();
    let debt = app.world().get::<CorporateDebt>(colony_id).unwrap();

    // 85 to stockpile, 15 pays off debt
    assert_eq!(stockpile.amount, 85.0);
    assert_eq!(debt.amount, 10000.0 - 15.0);
}

#[test]
fn test_debt_payoff_triggers_independence_movement() {
    let mut app = setup_test_app();
    // Setup colony with exactly 15 debt left
    let colony_id = setup_colony_with_debt(&mut app, 15.0, 0.15);

    produce_resources(&mut app, colony_id, 100.0);
    app.update();

    // Debt should be 0, trait should be added
    assert!(app.world().get::<CorporateDebt>(colony_id).is_none());
    assert!(app.world().get::<IndependenceMovement>(colony_id).is_some());
}
```

## GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass

#[derive(Component)]
pub struct CorporateDebt {
    pub amount: f32,
    pub siphon_percentage: f32,
}

#[derive(Component)]
pub struct IndependenceMovement;

pub fn process_debt_siphon(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CorporateDebt, &mut ResourceStockpile, &ProducedResources)>,
) {
    for (entity, mut debt, mut stockpile, produced) in query.iter_mut() {
        let payment = produced.amount * debt.siphon_percentage;
        let actual_payment = payment.min(debt.amount);

        stockpile.amount += produced.amount - actual_payment;
        debt.amount -= actual_payment;

        if debt.amount <= 0.0 {
            commands.entity(entity).remove::<CorporateDebt>();
            commands.entity(entity).insert(IndependenceMovement);
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- Move debt processing to an economic system step.
- Ensure floating point precision doesn't cause issues with debt payoff.
- Extract constants for default debt amount and siphon percentage.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Debt correctly siphons output until paid, then triggers independence movement.

## Technical Guidance

- Use existing `ResourceStockpile` and production systems.
- Consider UI implications for siphoned resources to ensure the player understands the penalty.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
