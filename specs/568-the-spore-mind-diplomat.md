# 568: The Spore-Mind Diplomat

## Overview

Accidentally appointing an alien hive-mind to speak for your people. A Pop heavily infected by "Symbiont Spores" (from native flora) gains a massive boost to their Social and Intellect stats, making them an ideal candidate for an Envoy (Layer 3 diplomat). However, they negotiate on behalf of the colony *and* the planetary fungal network.

## Dependencies

- Symbiont Spores (Spec 556).
- Layer 3 Diplomacy (Spec Z).


- A `SymbiontInfected` trait that vastly boosts Social and Intellect.
- When an infected Pop is assigned as an Envoy (`EnvoyRole`), all treaties they negotiate have hidden `FungalPropagationClause` attached.
- The `FungalPropagationClause` slowly infects the allied Layer 3 empire, leading to a biological crisis.

- Completely random infection of other empires without treaties; it must strictly be a consequence of the player choosing this specific envoy.

## RED Phase: Tests First

```rust
// tests/integration/spore_mind_diplomat.rs

#[test]
fn test_symbiont_infected_envoy_boosts_diplomatic_stats() {
    let mut app = setup_test_app();
    let pop = spawn_test_pop(&mut app);

    // Infect pop with SymbiontSpores
    infect_pop(&mut app, pop, InfectionType::SymbiontSpores);
    app.update();

    // Assert massive boost to Social and Intellect
    let stats = app.world().get::<Stats>(pop).unwrap();
    assert!(stats.social > 50);
    assert!(stats.intellect > 50);
}

#[test]
fn test_infected_envoy_attaches_fungal_clause_to_treaties() {
    let mut app = setup_test_app();
    let envoy = spawn_infected_envoy(&mut app);

    // Negotiate treaty with Layer 3 empire
    let treaty = negotiate_treaty(&mut app, envoy, TargetEmpire::Allied);
    app.update();

    // Assert the treaty contains the hidden fungal propagation clause
    assert!(treaty.clauses.contains(&TreatyClause::FungalPropagation));
}

#[test]
fn test_fungal_clause_spreads_infection_to_allied_empire() {
    let mut app = setup_test_app();
    let empire = setup_layer3_empire(&mut app);
    let treaty = create_treaty_with_fungal_clause(&mut app);

    // Apply treaty to empire
    apply_treaty(&mut app, empire, treaty);

    // Simulate long-term effects
    app.update_n_ticks(100);

    // Assert the allied empire begins accumulating fungal infection
    let infection_level = app.world().get::<FungalInfectionLevel>(empire).unwrap();
    assert!(infection_level.value > 0.0);
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer3/diplomacy.rs
pub struct TreatyClause {
    pub clause_type: ClauseType,
}

// src/layer3/systems/diplomacy_systems.rs
pub fn negotiate_treaty_system(
    mut events: EventReader<NegotiateTreatyEvent>,
    query_envoys: Query<(&Stats, Option<&SymbiontInfected>)>,
) {
    for event in events.read() {
        if let Ok((stats, infected)) = query_envoys.get(event.envoy) {
            let mut clauses = vec![TreatyClause::StandardTrade];

            if infected.is_some() {
                clauses.push(TreatyClause::FungalPropagation);
            }

            // finalize treaty
            finalize_treaty(event.target, clauses);
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Refactoring:** Do not hardcode `StandardTrade`; construct treaties dynamically based on the event intention.
- **Code Smells:** Avoid `Option<&SymbiontInfected>` in the query if only envoys negotiate; filter appropriately.
- **API Improvements:** Make the stats boost a modifier rather than permanently overwriting base stats.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Infected envoys successfully attach fungal clauses to negotiated treaties.

## Technical Guidance

### Components
```rust
#[derive(Component)]
pub struct SymbiontInfected;

#[derive(Component)]
pub struct FungalPropagationClause;
```

### Systems
```rust
pub fn apply_symbiont_stat_boosts_system(...) {}
pub fn resolve_fungal_treaty_spread_system(...) {}
```

### Integration Points
Connect this closely to the `SymbiontSpores` feature (Spec 556) to handle the initial infection mechanic, and bridge the gap to Layer 3 events.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
