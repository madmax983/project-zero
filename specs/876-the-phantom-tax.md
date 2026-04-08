# The Phantom Tax

## 1. Overview
The "Phantom Tax" represents a glitch in the galactic banking system where a minor, randomized fraction of all Credits transferred on the Layer 3 Galactic Market "disappears." This tax accumulates in a forgotten digital account (the "Slush Fund"). A Pop on Layer 1 with the `Hacker` trait can occasionally tap into this account, instantly injecting massive wealth into the colony's economy. However, doing so immediately triggers an event where an invincible, automated repo-fleet from the Galactic Bank is dispatched to collect the debt. This mechanic forces players to balance incredible short-term wealth against an overwhelming military response.

## 2. Dependencies
- `039-trade-system.md` (for Credit transfers and economy)
- `159-fleet-combat.md` (for fleet interactions)
- `543-the-galactic-market.md` (for Layer 3 market trades)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_market_trades_accumulate_phantom_tax() {
    let mut app = App::new();
    // Setup Galactic Market and Slush Fund resource
    // Execute a trade
    // Assert trade value is taxed and Slush Fund increases appropriately
}

#[test]
fn test_hacker_pop_can_drain_slush_fund() {
    let mut app = App::new();
    // Setup Hacker Pop and Slush Fund with credits
    // Trigger hack event
    // Assert Slush Fund is 0 and Colony Credits increased
}

#[test]
fn test_hack_triggers_repo_fleet_spawn() {
    let mut app = App::new();
    // Setup Layer 2 map and Hacker Pop
    // Trigger hack event
    // Assert RepoFleet is spawned targeting the colony
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation to pass the tests

#[derive(Resource, Default)]
pub struct SlushFund(pub u32);

#[derive(Component)]
pub struct HackerTrait;

#[derive(Component)]
pub struct RepoFleet {
    pub target_colony: Entity,
}

#[derive(Event)]
pub struct HackSlushFundEvent {
    pub hacker_entity: Entity,
    pub colony_entity: Entity,
}

pub fn accumulate_phantom_tax_system(
    mut slush_fund: ResMut<SlushFund>,
    mut trade_events: EventReader<MarketTradeEvent>,
) {
    for event in trade_events.read() {
        let tax = (event.value as f32 * 0.01) as u32; // 1% tax
        slush_fund.0 += tax;
    }
}

pub fn execute_hack_system(
    mut commands: Commands,
    mut hack_events: EventReader<HackSlushFundEvent>,
    mut slush_fund: ResMut<SlushFund>,
    mut colony_resources: Query<&mut ColonyResources>,
) {
    for event in hack_events.read() {
        if let Ok(mut resources) = colony_resources.get_mut(event.colony_entity) {
            resources.credits += slush_fund.0;
            slush_fund.0 = 0;

            // Spawn Repo Fleet (simplified logic for green phase)
            commands.spawn((
                RepoFleet { target_colony: event.colony_entity },
                Transform::default(), // Spawned somewhere in Layer 2
            ));
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: The `execute_hack_system` assumes the Hacker trait is valid just by receiving the event. The system should verify the entity still exists and possesses the `HackerTrait` to prevent bugs if the Pop dies before the event is processed.
- **Integration Points**: Ensure the `RepoFleet` behaves uniquely, bypassing normal diplomacy and engaging immediately upon reaching the colony's orbit.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Market trades contribute a small percentage to the `SlushFund`
- [ ] Hacks transfer the `SlushFund` balance to the colony
- [ ] Hacks spawn a `RepoFleet` targeting the colony

## 7. Technical Guidance
- **Gotchas**: Ensure the `RepoFleet` target is valid. If the colony is destroyed before the fleet arrives, the fleet should despawn or reassess its behavior to avoid panicking.
- **Gotchas**: The percentage for the tax should be configurable, potentially through a resource or constant.

## 8. Questions
*Builder: add questions here if spec is unclear.*