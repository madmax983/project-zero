# The Debt of the Dead

## 1. Overview
When a Pop with high Credit debt dies, their debt is not erased. Instead, it is inherited by their closest relatives or social connections (determined by the `Relationships` component). These inheritors suffer an immediate "Inherited Burden" morale penalty. If no connections exist, the debt is socialized to the entire colony, causing a slight increase in local prices. This creates an economic system where debt drives multigenerational misery, reflecting the brutal reality of frontier economics.

## 2. Dependencies
- `031-pop-morale.md` (for Morale penalties)
- `047-pop-relationships.md` (to find relatives/connections)
- `194-company-scrip.md` (for personal pop wealth and debt)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_dead_pop_debt_transferred_to_relative() {
    let mut app = App::new();
    // Setup dying Pop with Debt
    // Setup relative Pop
    // Trigger PopDiedEvent
    // Assert relative Pop has Debt added and InheritedBurden morale penalty
}

#[test]
fn test_dead_pop_debt_socialized_if_no_relatives() {
    let mut app = App::new();
    // Setup dying Pop with Debt, no relatives
    // Trigger PopDiedEvent
    // Assert ColonyEconomy resource has socialized_debt increased
}

#[test]
fn test_socialized_debt_increases_prices() {
    let mut app = App::new();
    // Setup ColonyEconomy with socialized_debt
    // Query price of an item
    // Assert price is higher than base price
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation to pass the tests

#[derive(Component)]
pub struct PopWealth {
    pub credits: i32, // Negative means debt
}

#[derive(Resource, Default)]
pub struct ColonyEconomy {
    pub socialized_debt: u32,
}

pub fn handle_dead_pop_debt_system(
    mut events: EventReader<PopDiedEvent>,
    wealth_query: Query<&PopWealth>,
    relationships_query: Query<&Relationships>,
    mut target_wealth_query: Query<(&mut PopWealth, &mut Morale), Without<Dead>>,
    mut economy: ResMut<ColonyEconomy>,
) {
    for event in events.read() {
        if let Ok(wealth) = wealth_query.get(event.entity) {
            if wealth.credits < 0 {
                let debt_amount = wealth.credits.abs() as u32;

                let mut debt_passed = false;
                if let Ok(relationships) = relationships_query.get(event.entity) {
                    if let Some(closest_relative) = relationships.get_closest_relative() {
                        if let Ok((mut relative_wealth, mut morale)) = target_wealth_query.get_mut(closest_relative) {
                            relative_wealth.credits -= debt_amount as i32;
                            morale.add_modifier(MoodModifier {
                                source: "Inherited Burden".to_string(),
                                value: -20.0,
                                duration: Some(Timer::from_seconds(600.0, TimerMode::Once)),
                            });
                            debt_passed = true;
                        }
                    }
                }

                if !debt_passed {
                    economy.socialized_debt += debt_amount;
                }
            }
        }
    }
}

pub fn calculate_local_prices(
    base_price: f32,
    economy: &ColonyEconomy,
) -> f32 {
    let inflation_multiplier = 1.0 + (economy.socialized_debt as f32 / 1000.0);
    base_price * inflation_multiplier
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**: The `Relationships` component method `get_closest_relative` is used as a mock. Ensure the actual implementation from `047-pop-relationships.md` is correctly utilized to find the strongest bond if blood relatives don't exist.
- **Code Smells**: The inline calculation for price inflation should ideally be handled within an existing market/trade system rather than as a standalone function.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Debt from a dying pop is correctly transferred to a living relative
- [ ] Relative receives `Inherited Burden` Morale modifier
- [ ] Debt is added to `socialized_debt` if no relatives exist
- [ ] Prices increase based on `socialized_debt`

## 7. Technical Guidance
- **Code Structure**: Ensure the `handle_dead_pop_debt_system` runs after the death event is emitted but before the entity is despawned if components are queried. Alternatively, pass the debt value inside the `PopDiedEvent`.
- **Gotchas**: If a relative inherits debt and immediately dies, the debt should chain to the next relative or be socialized.

## 8. Questions
*Builder: add questions here if spec is unclear.*