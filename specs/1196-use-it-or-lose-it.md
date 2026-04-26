# 1196: Use It Or Lose It

## 1. Overview
**Layer:** 3 -> 1

**Fantasy:** The absurdity of corporate budgeting. Spending money on useless things just to keep your funding.

**Mechanic:** You receive a yearly "Operating Budget" from the Layer 3 Faction. Any unspent Credits at year-end are deducted from *next year's* budget allocation ("Surplus Reduction"). You are forced to buy "statues" or "expensive carpets" to hit the zero-balance target.

**Emergence:** You pave the streets with gold during a famine, not because you are rich, but because if you don't spend the grant money now, you won't get it next year when the famine might be worse.

**Tension:** Efficient saving (Fiscal Responsibility) vs. Bureaucratic spending (Funding Security).

## 2. Dependencies
- Base ECS system (`src/layer1/mod.rs` or relevant system)
- Layer 3 -> 1 core modules

## 3. RED Phase: Tests First
```rust
// specs/1196-use-it-or-lose-it.md - doctest for TDD
// These tests should fail until implementation is complete.

#[test]
fn test_surplus_reduction_deducts_unspent_credits() {
    let mut world = World::new();
    world.insert_resource(Budget { target: 1000, unspent: 200, next_year: 1000 });

    // Act: Run the Layer 3 budget allocation system
    run_yearly_budget_system(&mut world);

    // Assert: Verify next year's budget allocation is reduced by the surplus amount
    let budget = world.resource::<Budget>();
    assert_eq!(budget.next_year, 800);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal code to turn tests green.
// Define required components and resources.
// Update relevant systems to process the new data.
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Avoid tight coupling.
- **Performance**: Ensure systems are optimized.
- **Design**: Consider integration points.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for new code.
- [ ] Basic logic and edge cases are verified.

## 7. Technical Guidance
- Register all new systems, events, and resources in the main app.
- Check relevant integration points in `src/`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
