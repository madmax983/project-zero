# 1213: Trade Embargoes

## 1. Overview
**Layer:** 3

**Fantasy:** Economic warfare. Starving the enemy without firing a shot.

**Mechanic:** You can convince the Galactic Council (or use Influence) to "Embargo" a resource. That resource cannot be traded on the Galactic Market.

**Emergence:** You have a monopoly on "Durasteel". You embargo it. Your enemy's fleet construction halts. They are forced to attack you to break the blockade.

**Tension:** Free Market (Profit) vs. Economic Weaponization (War).

## 2. Dependencies
- Galactic Market system
- Resource system

## 3. RED Phase: Tests First
```rust
#[test]
fn test_embargoed_resource_cannot_be_traded() {
    let mut app = App::new();
    app.add_systems(Update, process_market_trades);

    let mut market = GalacticMarket::default();
    market.embargo(ResourceType::Durasteel);
    app.world_mut().insert_resource(market);
    app.world_mut().insert_resource(Events::<TradeRequestEvent>::default());
    app.world_mut().insert_resource(Events::<TradeSuccessEvent>::default());

    // Request a trade
    app.world_mut().send_event(TradeRequestEvent { resource: ResourceType::Durasteel, amount: 100 });

    app.update();

    // The trade should fail
    let success_events = app.world().resource::<Events<TradeSuccessEvent>>();
    let mut reader = success_events.get_reader();
    assert_eq!(reader.read(success_events).count(), 0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_market_trades(
    mut requests: EventReader<TradeRequestEvent>,
    mut successes: EventWriter<TradeSuccessEvent>,
    market: Res<GalacticMarket>,
) {
    for req in requests.read() {
        if !market.is_embargoed(req.resource) {
            successes.send(TradeSuccessEvent);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Implement cost/influence mechanics for enacting an embargo.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Hook into the layer 3 trade logic.

## 8. Questions
*Builder: add questions here if spec is unclear.*
