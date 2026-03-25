# The Nostalgia Plagues

## 1. Overview
A psychological condition where colonists become crippled by a collective longing for a dead Earth. Pops infected by "Nostalgia" refuse to consume synthetic food, demand "Earth-like" commodities (like wood), and periodically stop working to gaze at the sky. The condition spreads via social interaction.

## 2. Dependencies
- `src/layer1/needs.rs` (Pop needs and consumption)
- `src/layer1/social.rs` (Social interactions and disease/idea spread)
- `src/layer1/utility_ai.rs` (Action selection and work interruption)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_nostalgia_infection_spreads() {
    let mut app = App::new();
    // Setup two Pops interacting, one with Nostalgia component
    // act: run social interaction system
    // assert: second Pop gains Nostalgia component
}

#[test]
fn test_nostalgia_alters_dietary_preferences() {
    let mut app = App::new();
    // Setup a Pop with Nostalgia and hunger
    // act: evaluate food options (Synthetic vs. Earth-like)
    // assert: Pop refuses Synthetic food, strictly prefers Earth-like
}

#[test]
fn test_nostalgia_interrupts_work() {
    let mut app = App::new();
    // Setup a Pop with Nostalgia working
    // act: run utility AI
    // assert: Pop occasionally selects 'GazeAtSky' action instead of working
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// In src/layer1/psychology.rs
#[derive(Component)]
pub struct NostalgiaInfection;

pub fn spread_nostalgia_system(
    mut commands: Commands,
    interactions: Query<(&Interaction, &Entity, &Entity)>,
    infected: Query<&NostalgiaInfection>,
) {
    for (interaction, pop_a, pop_b) in interactions.iter() {
        if infected.get(*pop_a).is_ok() && infected.get(*pop_b).is_err() {
            commands.entity(*pop_b).insert(NostalgiaInfection);
        }
    }
}

pub fn nostalgia_diet_filter(
    mut query: Query<&mut DietPreferences, With<NostalgiaInfection>>,
) {
    for mut diet in query.iter_mut() {
        diet.refuses_synthetic = true;
        diet.requires_earth_like = true;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate `NostalgiaInfection` tightly with the Utility AI so the "GazeAtSky" action has dynamic weight based on infection severity.
- Abstract the contagion mechanic so it can be reused for other psychological phenomena.
- Ensure UI properly surfaces the reason Pops are starving (refusing synthetic food) to the player.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Nostalgia spreads between Pops via social interaction.
- [ ] Infected Pops reject synthetic food.
- [ ] Infected Pops periodically interrupt their work to perform the `GazeAtSky` action.

## 7. Technical Guidance
- Add `NostalgiaInfection` component.
- Modify the `evaluate_actions_system` to heavily weigh a new `GazeAtSky` action if the component is present.
- Update `eat_system` or food selection logic to filter out `ItemType::SyntheticFood` for infected Pops.

## 8. Questions
*Builder: add questions here if spec is unclear.*
