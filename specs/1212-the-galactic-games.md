# 1212: The Galactic Games

## 1. Overview
**Layer:** 3 -> 1

**Fantasy:** Proving your civilization's superiority in the arena, not the battlefield.

**Mechanic:** Periodic galactic event. Factions send "Champions" (Pops with high Physical/Skill stats) to compete. Winning grants Influence/Peace. Losing causes National Shame.

**Emergence:** You train a "Super-Athlete" for years. Just before the games, he gets the "Flu". You send a random miner instead. He wins the "Rock Drilling" event, becoming a folk hero.

**Tension:** Invest in non-productive athletes (Prestige) vs. Productive workers (Economy).

## 2. Dependencies
- Diplomacy / Factions
- Event bus

## 3. RED Phase: Tests First
```rust
#[test]
fn test_champion_victory_grants_influence() {
    let mut app = App::new();
    app.add_systems(Update, process_galactic_games_results);

    app.world_mut().insert_resource(Economy { influence: 0.0 });
    app.world_mut().insert_resource(Events::<GamesResultEvent>::default());

    app.world_mut().send_event(GamesResultEvent { won: true });

    app.update();

    let economy = app.world().resource::<Economy>();
    assert!(economy.influence > 0.0);
}

#[test]
fn test_champion_loss_causes_shame() {
    let mut app = App::new();
    app.add_systems(Update, process_galactic_games_results);

    app.world_mut().insert_resource(Economy { influence: 100.0 });
    app.world_mut().insert_resource(Unrest { level: 0.0 });
    app.world_mut().insert_resource(Events::<GamesResultEvent>::default());

    app.world_mut().send_event(GamesResultEvent { won: false });

    app.update();

    let unrest = app.world().resource::<Unrest>();
    assert!(unrest.level > 0.0); // Shame increases unrest
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_galactic_games_results(
    mut events: EventReader<GamesResultEvent>,
    mut economy: ResMut<Economy>,
    mut unrest: ResMut<Unrest>,
) {
    for ev in events.read() {
        if ev.won {
            economy.influence += 50.0;
        } else {
            unrest.level += 20.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Implement the system that actually triggers the event and evaluates the stats.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Integrate into the Economy and Faction logic.

## 8. Questions
*Builder: add questions here if spec is unclear.*
