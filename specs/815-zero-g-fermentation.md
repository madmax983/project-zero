# 815: Zero-G Fermentation

## 1. Overview
Some things just taste better when they haven't touched gravity.
Specific luxury goods (e.g., "Void-Ale", "Foam-Cake") can *only* be produced in Orbital Stations (Layer 2). They trade for massive value on the ground.

Your ground colony is a grim industrial hellscape, but the Governor demands his Void-Ale. You launch a shuttle just to pick up a keg, wasting tons of fuel for a drink. The tension lies in the logistics cost vs. luxury/trade value.

## 2. Dependencies
- `152-orbital-stations.md` for orbital stations.
- `039-trade-system.md` for ground/orbit trade.
- `031-pop-morale.md` for handling luxury demands.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_zero_g_fermentation_production() {
    // Arrange: Setup world with an orbital station capable of fermentation
    let mut app = setup_test_app();
    let orbital_station = spawn_orbital_station(&mut app.world, StationType::Brewery);

    // Act: Advance simulation time for a production cycle
    app.update(); // Initial tick
    advance_simulation_time(&mut app.world, Duration::days(1));
    app.update();

    // Assert: Verify Void-Ale was produced in the station's inventory
    let inventory = get_station_inventory(&app.world, orbital_station);
    assert!(inventory.contains(ItemType::VoidAle), "Zero-G fermentation should produce Void-Ale");
}

#[test]
fn test_zero_g_fermentation_consumption_morale() {
    // Arrange: Setup world with a colonist consuming Void-Ale
    let mut app = setup_test_app();
    let colony_entity = spawn_colony(&mut app.world);
    let pop_entity = spawn_pop(&mut app.world, colony_entity);
    give_item_to_pop(&mut app.world, pop_entity, ItemType::VoidAle);

    // Act: Advance simulation to trigger consumption
    let initial_morale = get_pop_morale(&app.world, pop_entity);
    app.update(); // Pop consumes item

    // Assert: Verify morale increases significantly
    let new_morale = get_pop_morale(&app.world, pop_entity);
    assert!(new_morale > initial_morale, "Consuming Void-Ale should increase morale");
}

#[test]
fn test_zero_g_fermentation_requires_zero_g() {
    // Arrange: Attempt to produce Void-Ale on the ground
    let mut app = setup_test_app();
    let colony_entity = spawn_colony(&mut app.world);
    let ground_brewery = spawn_brewery(&mut app.world, colony_entity);

    // Act: Advance simulation time
    advance_simulation_time(&mut app.world, Duration::days(1));
    app.update();

    // Assert: Verify Void-Ale was NOT produced
    let inventory = get_building_inventory(&app.world, ground_brewery);
    assert!(!inventory.contains(ItemType::VoidAle), "Void-Ale cannot be produced on the ground");
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Core components and systems for Zero-G Fermentation
#[derive(Component)]
pub struct ZeroGBrewery {
    pub production_time: Timer,
}

pub fn zero_g_fermentation_system(
    time: Res<Time>,
    mut query: Query<(&mut ZeroGBrewery, &mut Inventory, &OrbitalStation)>,
) {
    for (mut brewery, mut inventory, station) in query.iter_mut() {
        if station.gravity == GravityLevel::ZeroG {
            brewery.production_time.tick(time.delta());
            if brewery.production_time.just_finished() {
                inventory.add_item(ItemType::VoidAle, 1);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Logistics Integration:** Ensure the shuttles fetching the Void-Ale properly consume fuel, creating the intended tension of trading essential resources for luxury items.
- **Governor Demand:** Integrate with the `054-colony-edicts.md` or similar systems to have high-tier Pops or Governors specifically demand these items, penalizing morale heavily if the demand is not met.
- **Item Expansion:** Generalize the `ZeroGBrewery` to a `ZeroGManufacturer` that can produce other items like "Foam-Cake" based on recipes.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Void-Ale is only producible in zero-g environments and grants significant morale buffs when consumed.

## 7. Technical Guidance
- **Recipe System:** If a generic crafting system exists, tie the Void-Ale recipe to a `RequiresZeroG` tag instead of a custom building component.
- **Consumption:** Tap into the existing `Needs` or `UtilityAI` to make Pops prioritize consuming `ItemType::VoidAle` when available, especially if their morale is low.

## 8. Questions
*Builder: add questions here if spec is unclear.*
