# 592: The Whispering Ore

## 1. Overview
**Layer:** 1
**Fantasy:** A colony enriched by a strange new material that slowly alters the minds of its miners, birthing an emergent, unsettling religion.
**Mechanic:** A rare, hyper-valuable deep-crust ore is discovered. Miners extracting it slowly accumulate a hidden "Resonant" trait. Resonant Pops begin speaking a dead language and forming a cult around the ore vein. They refuse to work other jobs but mine at 200% efficiency. If the vein is depleted or sealed, the cult violently rebels to "reopen the eye."
**Emergence:** Your economy becomes entirely dependent on the incredibly lucrative ore exports. Suddenly, the miners demand the colony's primary power reactor be dismantled to construct a massive, humming shrine over the mine entrance.
**Tension:** Do you exploit the massive economic boom and risk a localized, fanatical uprising, or seal the mine and starve the colony of crucial funds?

## 2. Dependencies
- 014-mining
- 060-factions

## 3. RED Phase: Tests First
```rust
#[test]
fn test_mining_whispering_ore_adds_resonant_trait() {
    let mut app = setup_test_app();
    let miner = app.world.spawn((Pop, Job::Miner)).id();

    app.world.send_event(MinedOreEvent { miner, ore_type: OreType::Whispering });
    app.update();

    assert!(app.world.get::<ResonantTrait>(miner).is_some());
}

#[test]
fn test_resonant_pops_rebel_if_vein_sealed() {
    let mut app = setup_test_app();
    let miner = app.world.spawn((Pop, ResonantTrait)).id();

    app.world.send_event(MineSealedEvent { vein_id: 1 });
    app.update();

    assert!(app.world.get::<Rebelling>(miner).is_some());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/mining.rs
pub fn process_whispering_ore_system(
    mut events: EventReader<MinedOreEvent>,
    mut commands: Commands,
) {
    for event in events.iter() {
        if event.ore_type == OreType::Whispering {
            commands.entity(event.miner).insert(ResonantTrait);
        }
    }
}

pub fn handle_mine_sealing_system(
    mut events: EventReader<MineSealedEvent>,
    query: Query<Entity, With<ResonantTrait>>,
    mut commands: Commands,
) {
    for _ in events.iter() {
        for entity in query.iter() {
            commands.entity(entity).insert(Rebelling);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Tie the Resonant trait to a gradual build-up rather than instant acquisition.
- Implement the 200% mining efficiency modifier for Resonant pops.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops mining Whispering Ore gain the Resonant trait.
- [ ] Resonant pops enter a rebellious state if their mine is sealed.

## 7. Technical Guidance
- Consider using a distinct `Faction` for the cult to manage their demands and global effects.

## 8. Questions
*Builder: add questions here if spec is unclear.*
