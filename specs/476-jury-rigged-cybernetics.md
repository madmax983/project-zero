# Specification 476: Jury-Rigged Cybernetics

## 1. Overview
When a Pop loses a limb, they can be fitted with a "Jury-Rigged" prosthetic made from scrap components. This restores full mobility and work speed but adds a random, dangerous side-effect based on the scrap used (e.g., "Combustible", "Loud", "High Energy Drain"). This presents a tension between fast, cheap, dangerous workforce restoration versus slow, expensive, safe medical care.

## 2. Dependencies
- Health/Injury system (`LimbLoss` or equivalent injury component).
- Scrap resource economy.
- Pop trait and efficiency modifier systems (`WorkSpeed`, `Mobility`).

## 3. RED Phase: Tests First

```rust
#[test]
fn test_jury_rigged_prosthetic_restores_mobility() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // ... add required custom plugins

    // Pop with a missing limb and reduced mobility
    let pop = app.world_mut().spawn((
        PopBundle::default(),
        MissingLimb { location: Limb::Leg },
        Mobility { value: 0.5 },
    )).id();

    // Act: Apply the scrap prosthetic
    app.world_mut().entity_mut(pop).insert(JuryRiggedProsthetic { side_effect: SideEffect::Loud });
    app.update();

    // Assert: Mobility is restored to full
    let mobility = app.world().get::<Mobility>(pop).unwrap();
    assert_eq!(mobility.value, 1.0);
}

#[test]
fn test_combustible_prosthetic_side_effect() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let pop = app.world_mut().spawn((
        PopBundle::default(),
        JuryRiggedProsthetic { side_effect: SideEffect::Combustible },
        Stress { value: 90.0 }, // High stress triggers the accident
    )).id();

    // Act: Run the prosthetic side-effect system
    app.update();

    // Assert: Pop has a chance to catch fire when stressed
    assert!(app.world().entity(pop).contains::<OnFire>());
}

#[test]
fn test_energy_drain_prosthetic_side_effect() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let pop = app.world_mut().spawn((
        PopBundle::default(),
        JuryRiggedProsthetic { side_effect: SideEffect::HighEnergyDrain },
        EnergyReserves { value: 100.0 },
    )).id();

    // Act: Run one tick of energy consumption
    app.update();

    // Assert: The Pop drains significantly more energy than a baseline Pop
    let energy = app.world().get::<EnergyReserves>(pop).unwrap();
    assert!(energy.value < 90.0); // Baseline might drain 1.0 per tick, this drains 10.0+
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct JuryRiggedProsthetic {
    pub side_effect: SideEffect,
}

#[derive(PartialEq)]
pub enum SideEffect {
    Combustible,
    Loud,
    HighEnergyDrain,
}

pub fn prosthetic_restoration_system(
    mut query: Query<(&mut Mobility, &JuryRiggedProsthetic)>,
) {
    for (mut mobility, _prosthetic) in query.iter_mut() {
        // Minimal pass: Just force mobility to 1.0 if they have the prosthetic
        mobility.value = 1.0;
    }
}

pub fn prosthetic_side_effect_system(
    mut commands: Commands,
    mut query: Query<(Entity, &JuryRiggedProsthetic, &Stress, &mut EnergyReserves)>,
) {
    for (entity, prosthetic, stress, mut energy) in query.iter_mut() {
        match prosthetic.side_effect {
            SideEffect::Combustible => {
                if stress.value > 80.0 {
                    commands.entity(entity).insert(OnFire);
                }
            }
            SideEffect::HighEnergyDrain => {
                energy.value -= 10.0;
            }
            SideEffect::Loud => {
                // Add noise radius or similar
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Side Effect RNG**: Replace deterministic side effects with randomized assignment upon the creation of the `JuryRiggedProsthetic`, heavily weighted by the type of scrap used in surgery.
- **Lexicon Integration**: Ensure logs reference terms like "scrap-limb" or "welded-flesh" when a Pop acts erratically due to side effects.
- **Lore Context**: Generational vows or certain trait configurations might explicitly forbid receiving a scrap-limb.
- **Work Speed**: Implement the `WorkSpeed` restoration alongside `Mobility`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Applying a `JuryRiggedProsthetic` restores `Mobility` and `WorkSpeed` values.
- [ ] Pops with `SideEffect::Combustible` periodically trigger fire hazards under high stress.
- [ ] Pops with `SideEffect::HighEnergyDrain` deplete personal `EnergyReserves` faster.

## 7. Technical Guidance
- The `prosthetic_side_effect_system` should run in the `Layer1SystemSet::Execution` schedule to properly affect simulation logic every tick or interval.
- Be careful with the `OnFire` component insertion: ensure it doesn't just loop infinitely on every tick if already present.
- Consider utilizing Bevy's Event system to trigger accidents, rather than hard-coding the component swap directly, as fires may need to propagate to surrounding buildings (e.g., `FirePropagation` spec 033).

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
