# 1160: The Orbital Secession

## Overview
A massive orbital habitat built at Layer 2 can develop its own distinct culture. If its wealth and population eclipse the planet below, and unrest is high, it can declare itself an independent nation. It will immediately embargo the planet and demand tribute. This creates a tension balancing the economic powerhouse of orbital structures against the danger of allowing them to become entirely self-sufficient and culturally disconnected from the homeworld.

## Dependencies
- Layer 1 colony management
- Layer 2 orbital structures
- Diplomacy and Unrest systems
- Economy/Wealth tracking for colonies and orbital structures

## RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_orbital_secession_conditions() {
    // Arrange: Setup an orbital habitat and a planet. Give habitat high wealth/population and unrest.
    // Act: Run the secession evaluation system
    // Assert: The orbital habitat changes faction to independent and embargoes the planet
}

#[test]
fn test_orbital_secession_tribute_demand() {
    // Arrange: An independent orbital habitat exists above a planet
    // Act: Advance simulation
    // Assert: A tribute demand event is generated targeting the planet's faction
}

#[test]
fn test_orbital_secession_embargo_effect() {
    // Arrange: A secession has occurred
    // Act: Attempt to transfer resources between the planet and the orbital habitat
    // Assert: The transfer fails or is blocked due to the embargo
}
```

## GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN

pub fn evaluate_orbital_secession_system(
    mut commands: Commands,
    mut orbital_habitats: Query<(Entity, &mut Faction, &Wealth, &Population, &Unrest, &Orbiting)>,
    planets: Query<(&Wealth, &Population)>,
) {
    for (entity, mut faction, wealth, population, unrest, orbiting) in orbital_habitats.iter_mut() {
        if let Ok((planet_wealth, planet_pop)) = planets.get(orbiting.target) {
            if wealth.value > planet_wealth.value &&
               population.count > planet_pop.count &&
               unrest.level > 50.0 {
                // Secede!
                let old_faction = faction.clone();
                *faction = Faction::Independent;
                commands.entity(entity).insert(Seceded {
                    original_faction: old_faction,
                });
                commands.entity(entity).insert(Embargo {
                    target: orbiting.target,
                });
            }
        }
    }
}

pub fn generate_tribute_demand_system(
    mut commands: Commands,
    seceded_habitats: Query<(Entity, &Seceded, &Embargo)>,
) {
    for (entity, _seceded, embargo) in seceded_habitats.iter() {
        // Generate a demand
        commands.spawn(TributeDemand {
            from: entity,
            to: embargo.target,
            amount: 1000.0,
        });
    }
}
```

## REFACTOR Phase: Quality & Design
- **Code Smell:** The secession logic hardcodes threshold values (e.g., `unrest.level > 50.0`). These should be configurable parameters or derived from simulation difficulty.
- **Optimization:** Extract the conditions into a separate function to make testing easier and avoid deeply nested `if` statements.
- **Integration:** Integrate with the Chronicle system so the secession event is recorded properly with generated lore text.
- **Refactoring:** Ensure `Embargo` affects pathfinding or trade route evaluation properly instead of just an empty component.

## Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code in `src/layer2/secession.rs`
- [ ] Orbital Secession event correctly generates a Chronicle entry

## Technical Guidance
- **System Placement:** `evaluate_orbital_secession_system` should run in the `SimulationSchedule` (Layer 2 update).
- **Components:** Create new components `Seceded` and `TributeDemand`. Ensure `Embargo` ties into the trade logic correctly.
- **Events:** Trigger a `SecessionEvent` that the UI and Chronicle systems can listen to.

## Questions
*Builder: add questions here if spec is unclear. Architect will address.*
