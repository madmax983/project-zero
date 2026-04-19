# Prohibition & Contraband

## 1. Overview
Banning items (e.g., "No Alcohol") creates a "Black Market" price premium, increasing smuggling activities. The core fantasy revolves around forbidden pleasures, creating a tension between Legalization (Control/Taxation) and Prohibition (Health/Crime). This specification enables the mechanical designation of items as prohibited and models the resulting economic and criminal consequences.

## 2. Dependencies
- Trade System (`specs/039-trade-system.md`)
- Resource Capacities (`specs/360-resource-capacities.md`)
- The Black Market (`specs/219-the-black-market.md`)
- Crime System (`specs/410-crime-and-punishment.md`)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_prohibited_item_flag() {
    let mut app = App::new();
    app.add_plugins(ProhibitionPlugin);

    // Arrange: Create an item type and mark it as prohibited
    let item_entity = app.world_mut().spawn(ItemType::new("Stim-pack")).id();

    // Act: Prohibit the item via event
    app.world_mut().send_event(ProhibitItemEvent { item: item_entity });
    app.update();

    // Assert: Item has Prohibited component
    assert!(app.world().entity(item_entity).contains::<Prohibited>());
}

#[test]
fn test_prohibition_increases_black_market_value() {
    let mut app = App::new();
    app.add_plugins((TradePlugin, ProhibitionPlugin, BlackMarketPlugin));

    let item_entity = app.world_mut().spawn((
        ItemType::new("Engine-Coolant Moonshine"),
        BaseValue(10.0),
    )).id();

    // Act: Item becomes prohibited
    app.world_mut().send_event(ProhibitItemEvent { item: item_entity });
    app.update();

    // Assert: Black market price multiplier is applied
    let query = app.world_mut().query::<&BlackMarketValue>().iter(app.world()).next().unwrap();
    assert!(query.0 > 10.0, "Black market value should be higher than base value for prohibited items");
}

#[test]
fn test_smuggling_increases_with_prohibition() {
    let mut app = App::new();
    app.add_plugins((CrimePlugin, ProhibitionPlugin));

    // Arrange: Setup colony with base crime rate
    let colony_entity = app.world_mut().spawn((Colony, SmugglingRate(0.0))).id();
    let item_entity = app.world_mut().spawn((ItemType::new("Alcohol"), Prohibited)).id();

    // Act: Run smuggling calculation tick
    app.update();

    // Assert: Smuggling rate increased
    let smuggling = app.world().get::<SmugglingRate>(colony_entity).unwrap();
    assert!(smuggling.0 > 0.0, "Smuggling rate should increase when prohibited items exist");
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct Prohibited;

#[derive(Event)]
pub struct ProhibitItemEvent {
    pub item: Entity,
}

pub struct ProhibitionPlugin;

impl Plugin for ProhibitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProhibitItemEvent>()
           .add_systems(Update, (
               process_prohibition_events,
               update_black_market_prices,
               calculate_smuggling_rates,
           ));
    }
}

fn process_prohibition_events(
    mut events: EventReader<ProhibitItemEvent>,
    mut commands: Commands,
) {
    for event in events.read() {
        commands.entity(event.item).insert(Prohibited);
    }
}

fn update_black_market_prices(
    mut commands: Commands,
    query: Query<(Entity, &BaseValue), With<Prohibited>>,
) {
    for (entity, base_value) in query.iter() {
        // Simple 2x multiplier for minimal implementation
        commands.entity(entity).insert(BlackMarketValue(base_value.0 * 2.0));
    }
}

fn calculate_smuggling_rates(
    prohibited_items: Query<(), With<Prohibited>>,
    mut colonies: Query<&mut SmugglingRate, With<Colony>>,
) {
    if !prohibited_items.is_empty() {
        for mut rate in colonies.iter_mut() {
            // Flat increase per prohibited item
            rate.0 += 5.0 * prohibited_items.iter().count() as f32;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: The `calculate_smuggling_rates` system uses a flat increase which will compound every tick if not clamped or decayed.
- **Performance**: Querying all prohibited items every tick for the smuggling rate might be slow if the list of items grows large. We should maintain a count resource instead.
- **API Improvements**: `ProhibitItemEvent` should probably include a `severity` or `enforcement_level` field to modulate the Black Market premium and Smuggling rate dynamically.
- **Integration**: Link the `Prohibited` status to Pop health systems to ensure that while smuggling goes up, overall colony health might improve (if toxic items are successfully kept out) or decline (if toxic substitutes are brewed).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Items can be dynamically prohibited, increasing their black market value.

## 7. Technical Guidance
- Ensure that the Black Market Plugin is loaded and can read the `BlackMarketValue` component added by this feature.
- Use `app.add_plugins(ProhibitionPlugin)` in the main application setup.
- Remember to reset or decay `SmugglingRate` appropriately in the broader Crime System to prevent it from reaching infinity.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
