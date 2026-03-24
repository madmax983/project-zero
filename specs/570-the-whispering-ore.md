# 570: The Whispering Ore

## Overview

A colony enriched by a strange new material that slowly alters the minds of its miners, birthing an emergent, unsettling religion. A rare, hyper-valuable deep-crust ore is discovered. Miners extracting it slowly accumulate a hidden "Resonant" trait. Resonant Pops begin speaking a dead language and forming a cult around the ore vein. They refuse to work other jobs but mine at 200% efficiency. If the vein is depleted or sealed, the cult violently rebels to "reopen the eye."

## Dependencies

- Mining logic (Spec C).
- Pop traits and factions (Spec D).


- `WhisperingOre` resource and vein.
- A `Resonant` trait slowly accumulating on pops actively mining the `WhisperingOre`.
- Pops with the `Resonant` trait refuse non-mining jobs and mine the ore twice as fast.
- A new `CultFaction` that forms automatically from highly Resonant pops.
- Sealing or depleting the mine triggers a violent `CultUprisingEvent`.

- Completely ignoring player agency; the player should be able to dismantle the mine before resonance reaches dangerous levels, at the cost of the ore.

## RED Phase: Tests First

```rust
// tests/integration/whispering_ore.rs

#[test]
fn test_mining_whispering_ore_adds_resonant_trait() {
    let mut app = setup_test_app();
    let pop = spawn_miner(&mut app);
    let vein = spawn_whispering_ore_vein(&mut app);

    // Assign pop to mine the whispering ore
    assign_pop_to_mine(&mut app, pop, vein);

    // Simulate time passing
    app.update_n_ticks(100);

    // Assert the pop gained the Resonant trait
    let resonant = app.world().get::<ResonantTrait>(pop);
    assert!(resonant.is_some());
    assert!(resonant.unwrap().intensity > 0.0);
}

#[test]
fn test_highly_resonant_pops_form_cult_faction() {
    let mut app = setup_test_app();
    let pop1 = spawn_resonant_pop(&mut app, 100.0);
    let pop2 = spawn_resonant_pop(&mut app, 100.0);

    app.update();

    // Assert Cult Faction formed
    let factions = app.world().resource::<FactionList>();
    assert!(factions.contains(FactionType::WhisperingCult));

    // Assert pops belong to cult
    let allegiance1 = app.world().get::<FactionAllegiance>(pop1).unwrap();
    assert_eq!(allegiance1.faction, FactionType::WhisperingCult);
}

#[test]
fn test_closing_whispering_mine_triggers_cult_uprising() {
    let mut app = setup_test_app();
    let _pop = spawn_cultist_miner(&mut app);
    let mine = spawn_active_whispering_mine(&mut app);

    // Player manually closes the mine
    close_mine(&mut app, mine);
    app.update();

    // Assert an uprising event was dispatched
    let uprisings = app.world().resource::<Events<CultUprisingEvent>>();
    assert!(!uprisings.is_empty());
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer1/mining.rs
#[derive(Component)]
pub struct WhisperingOreVein;

#[derive(Component)]
pub struct ResonantTrait {
    pub intensity: f32,
}

// src/layer1/systems/mining_systems.rs
pub fn accumulate_resonance_system(
    mut query_miners: Query<(&ActiveJob, &mut Option<&mut ResonantTrait>)>,
    query_veins: Query<&WhisperingOreVein>,
) {
    for (job, mut resonant_opt) in query_miners.iter_mut() {
        if query_veins.get(job.target_entity).is_ok() {
            // Apply resonance logic
            if let Some(mut resonant) = resonant_opt {
                resonant.intensity += 1.0;
            } else {
                // Should insert component, simplified for example
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Refactoring:** Instead of raw `Option<&mut Component>`, use Bevy commands to insert the trait dynamically when a pop starts mining the ore.
- **Code Smells:** The `CultUprisingEvent` generation shouldn't be hardcoded to `close_mine`; monitor the state of the active vein component so depletion triggers it naturally too.
- **API Improvements:** Encapsulate faction membership logic into dedicated helpers rather than directly mutating `FactionAllegiance` in arbitrary systems.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Whispering ore veins slowly convert miners to resonant cultists.
- [ ] Closing the mine causes the cultists to riot.

## Technical Guidance

### Components
```rust
#[derive(Component)]
pub struct ResonantTrait {
    pub intensity: f32,
}

pub struct CultUprisingEvent {
    pub origin_mine: Entity,
}
```

### Systems
```rust
pub fn process_whispering_ore_mining_system(...) {}
pub fn monitor_cult_uprising_conditions_system(...) {}
```

### Integration Points
Connect `CultUprisingEvent` to the Combat/Security systems to spawn hostile cultist entities or shift allegiance flags for the affected pops.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
