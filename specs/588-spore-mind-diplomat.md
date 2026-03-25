# 588: The Spore-Mind Diplomat

## 1. Overview
Accidentally appointing an alien hive-mind to speak for your people. A Pop heavily infected by "Symbiont Spores" gains a massive boost to their Social and Intellect stats, making them an ideal candidate for an Envoy (Layer 3 diplomat). However, they negotiate on behalf of the colony *and* the planetary fungal network. Tension: Using a biologically enhanced super-diplomat vs. losing control over the actual agenda being negotiated. Layer: Cross-layer (1 -> 3).

## 2. Dependencies
- Spore infection mechanics (Layer 1)
- Envoy/Diplomacy system (Layer 3)
- Stat system for Pops

## 3. RED Phase: Tests First
```rust
#[test]
fn test_spore_infection_boosts_stats() {
    let mut world = World::new();
    let pop = world.spawn((Pop, Stats { social: 5, intellect: 5 })).id();
    world.entity_mut(pop).insert(SporeInfection { severity: 1.0 });

    spore_stat_boost_system(&mut world);

    let stats = world.get::<Stats>(pop).unwrap();
    assert!(stats.social > 5);
    assert!(stats.intellect > 5);
}

#[test]
fn test_spore_diplomat_inserts_hidden_clause() {
    let mut world = World::new();
    let pop = world.spawn((Pop, Envoy, SporeInfection { severity: 1.0 })).id();
    let treaty = world.spawn(Treaty { clauses: vec![] }).id();
    world.insert_resource(ActiveNegotiation { envoy: pop, treaty });

    diplomatic_negotiation_system(&mut world);

    let treaty_data = world.get::<Treaty>(treaty).unwrap();
    assert!(treaty_data.clauses.contains(&Clause::SporePropagation));
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
pub struct SporeInfection { pub severity: f32 }
pub fn spore_stat_boost_system(world: &mut World) { ... }
pub fn diplomatic_negotiation_system(world: &mut World) { ... }
```

## 5. REFACTOR Phase: Quality & Design
- Create an extensible clause system for treaties so traits can dynamically inject clauses.
- Ensure the stat boost cleanly separates base stats from temporary/conditional buffs.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: Spore infected envoys are better at diplomacy but inject SporePropagation clauses into treaties.

## 7. Technical Guidance
- The negotiation system needs to be aware of the Envoy's specific traits/conditions when evaluating the outcome.

## 8. Questions
*Builder: add questions here if spec is unclear.*
