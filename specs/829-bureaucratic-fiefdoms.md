# Bureaucratic Fiefdoms

## 1. Overview
The government isn't a monolith; it's a hundred petty kings fighting over office supplies. As a colony grows, distinct administrative sectors form "Fiefdoms" led by powerful Mayor Pops. These autonomous regions tax resources moving across their physical boundaries and fiercely hoard their own stockpiles—even refusing to share life-saving supplies with a neighboring starving sector unless politically coerced.

## 2. Dependencies
- Needs `Grid` and `Inventory`/Stockpile mechanics in `src/layer1/`.
- Needs `Pop` movement and `Task` execution.
- Needs an administrative or sector tracking system (Layer 1 -> Layer 3 integration).

## 3. RED Phase: Tests First

```rust
#[test]
fn test_resource_transfer_between_fiefdoms_is_taxed() {
    let mut app = App::new();
    // Setup two sectors
    app.world.spawn(Fiefdom { id: 1, tax_rate: 0.1 });
    app.world.spawn(Fiefdom { id: 2, tax_rate: 0.2 });

    // Act: Attempt to move 100 food from Sector 1 to Sector 2
    let mut transfer = ResourceTransfer { amount: 100.0, from_sector: 1, to_sector: 2, resource: ItemType::Food };
    app.world.send_event(transfer);

    app.add_systems(Update, process_inter_fiefdom_transfers);
    app.update();

    // Assert: Only 80 food arrives (20% tax applied by receiving sector)
    // Or 10% tax applied by sender, depending on design. Let's say receiving sector taxes it.
    let sector_2_inventory = app.world.get_resource::<SectorInventories>().unwrap().get(2);
    assert_eq!(sector_2_inventory.food, 80.0);
}

#[test]
fn test_fiefdom_hoards_resources_during_crisis() {
    let mut app = App::new();
    // Setup Fiefdom 1 with a crisis (starving)
    app.world.spawn(Fiefdom { id: 1, state: FiefdomState::Starving });

    // Setup Fiefdom 2 with excess food
    let f2 = app.world.spawn(Fiefdom { id: 2, state: FiefdomState::Normal }).id();
    app.world.resource_mut::<SectorInventories>().set_food(2, 500.0);

    // Act: Sector 1 requests aid
    app.world.send_event(RequestAidEvent { from: 2, to: 1, amount: 100.0 });
    app.add_systems(Update, handle_fiefdom_aid_requests);
    app.update();

    // Assert: Sector 2 refuses (hoarding behavior)
    let sector_2_inventory = app.world.get_resource::<SectorInventories>().unwrap().get(2);
    assert_eq!(sector_2_inventory.food, 500.0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct Fiefdom {
    pub id: u32,
    pub tax_rate: f32,
}

#[derive(Event)]
pub struct ResourceTransfer {
    pub amount: f32,
    pub from_sector: u32,
    pub to_sector: u32,
    pub resource: ItemType,
}

#[derive(Event)]
pub struct RequestAidEvent {
    pub from: u32,
    pub to: u32,
    pub amount: f32,
}

pub fn process_inter_fiefdom_transfers(
    mut events: EventReader<ResourceTransfer>,
    fiefdoms: Query<&Fiefdom>,
    mut inventories: ResMut<SectorInventories>,
) {
    for event in events.read() {
        let receiving_fief = fiefdoms.iter().find(|f| f.id == event.to_sector).unwrap();
        let net_amount = event.amount * (1.0 - receiving_fief.tax_rate);
        inventories.add(event.to_sector, event.resource, net_amount);
    }
}

pub fn handle_fiefdom_aid_requests(
    mut events: EventReader<RequestAidEvent>,
    fiefdoms: Query<&Fiefdom>,
) {
    for event in events.read() {
        // Fiefdoms autonomously refuse to share resources without override
        // No resources are moved.
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Sectors**: Fiefdom IDs need to map to physical `TerrainGrid` coordinates so that Pops walking across boundaries physically trigger the tax logic.
- **Riot Integration**: If a Fiefdom refuses aid, the starved Pops in the neighboring sector should gain massive unrest, potentially forming a task to forcefully steal the resources from the hoarding sector.
- **Overriding**: Implement a Layer 3 "Political Coercion" edict that forces a Fiefdom to share, at the cost of the Mayor's loyalty.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Resources transferred between Fiefdoms are taxed appropriately
- [ ] Fiefdoms by default refuse aid requests from other sectors

## 7. Technical Guidance
- Integrate with `MovementSystem` so that hauling tasks recalculate their net value based on Fiefdom border taxes. Utility AI might prefer long routes within one Fiefdom over short routes crossing borders.
- Keep the `SectorInventories` cleanly separated from global `ColonyResources` to reflect the fractured economy.

## 8. Questions
*Builder: add questions here if spec is unclear.*
