# 222 - Paperwork Physicality

> "The permit A38 is not just a piece of paper. It is the physical manifestation of the Colony's will."

## 1. Overview

This feature introduces a bureaucratic bottleneck to advanced construction. Complex buildings (Tier 2 and above) are no longer "instantly active" upon construction. Instead, they spawn in a **Permit Required** state. They will not function until a physical `BuildingPermit` item is crafted at an `Office` and hauled to the building.

This transforms late-game expansion from a simple resource dump into a logistical challenge, requiring a functioning administrative supply chain.

### User Story
As a Colony Governor, I want advanced buildings to require physical permits so that my administrative infrastructure (Offices) becomes a critical bottleneck for expansion, preventing me from expanding too fast without bureaucracy.

## 2. Dependencies

*   `004` Building System - Core building spawning logic.
*   `025` Hauling Logistics - Moving the permit from Office to Building.
*   `009` Job System - Assigning haulers and preventing work on unpermitted buildings.

## 3. RED Phase: Tests First

These tests define the behavior of the `PermitRequired` component and its interaction with the simulation.

```rust
// tests/layer1/paperwork_tests.rs

use scale::layer1::building::{Building, BuildingType, Tier};
use scale::layer1::permit::{PermitRequired, BuildingPermit, permit_activation_system};
use scale::layer1::resources::{Inventory, ResourceType};
use scale::layer1::power::PowerConsumer;

#[test]
fn test_advanced_building_spawns_with_permit_requirement() {
    // Arrange
    let mut world = World::new();
    // Setup resources/terrain etc (omitted for brevity)

    // Act
    // Smelter is Tier 2 (Advanced), should require permit
    let id = spawn_building(&mut world, 0, 0, BuildingType::Smelter, MaterialType::default());

    // Assert
    assert!(world.get::<PermitRequired>(id).is_some(), "Smelter should require a permit");
}

#[test]
fn test_basic_building_spawns_active() {
    // Arrange
    let mut world = World::new();

    // Act
    // Farm is Tier 1 (Basic), should NOT require permit
    let id = spawn_building(&mut world, 0, 0, BuildingType::Farm, MaterialType::default());

    // Assert
    assert!(world.get::<PermitRequired>(id).is_none(), "Farm should not require a permit");
}

#[test]
fn test_permit_blocks_functionality() {
    // Arrange
    let mut world = World::new();
    let id = world.spawn((
        Building { building_type: BuildingType::Smelter },
        PermitRequired,
        PowerConsumer { active: true, ..Default::default() } // Normally active
    )).id();

    // Act
    // Run a system that enforces permit restrictions (e.g. disables power/work)
    enforce_permit_restrictions_system(&mut world);

    // Assert
    let consumer = world.get::<PowerConsumer>(id).unwrap();
    assert!(!consumer.active, "PowerConsumer should be disabled by PermitRequired");
}

#[test]
fn test_delivering_permit_activates_building() {
    // Arrange
    let mut world = World::new();
    let id = world.spawn((
        Building { building_type: BuildingType::Smelter },
        PermitRequired,
        Inventory::default() // Inventory to receive the permit
    )).id();

    // Simulate Hauler delivering the permit
    let mut inventory = world.get_mut::<Inventory>(id).unwrap();
    inventory.add(ResourceType::BuildingPermit, 1.0);

    // Act
    permit_activation_system(&mut world);

    // Assert
    assert!(world.get::<PermitRequired>(id).is_none(), "PermitRequired should be removed");

    let inventory = world.get::<Inventory>(id).unwrap();
    assert_eq!(inventory.get_amount(ResourceType::BuildingPermit), 0.0, "Permit should be consumed");
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Define Resource
*   Add `BuildingPermit` to `ResourceType` enum in `src/layer1/resources.rs`.
*   Update `ColonyResources` to track it (though it's mostly an item, not a fluid).

### 2. Define Component
*   Create `src/layer1/permit.rs`.
*   Define `#[derive(Component)] pub struct PermitRequired;`.

### 3. Update Building Spawning
*   In `src/layer1/building.rs`, modify `spawn_building`:
    ```rust
    if let Some((_, tier)) = building_type.tier_info() {
        if tier >= Tier::Advanced {
            entity.insert(PermitRequired);
            // Also ensure it has an Inventory to accept the item
            if !entity.contains::<Inventory>() {
                entity.insert(Inventory::default()); // Or specialized InputInventory
            }
        }
    }
    ```

### 4. Implement Activation System
*   Create `permit_activation_system` in `src/layer1/permit.rs`.
    *   Query entities with `PermitRequired` and `Inventory`.
    *   If `Inventory` has `ResourceType::BuildingPermit >= 1.0`:
        *   Consume Permit.
        *   Remove `PermitRequired`.
        *   Log event: "Building authorized: [Name]".

### 5. Enforce Restrictions
*   Create `enforce_permit_restrictions_system`.
    *   Query `PermitRequired`.
    *   Set `PowerConsumer.active = false`.
    *   Disable `WorkAssignment` (this might require modifying `job_assignment_system` to filter out `PermitRequired` workplaces).

### 6. Crafting Recipe
*   Add a recipe to `Office` to craft `BuildingPermit`.
    *   Cost: Time (Work) + Paper (if exists) or just generic "Admin Effort".
    *   Output: `ResourceType::BuildingPermit`.

## 5. REFACTOR Phase: Quality & Design

*   **UI Feedback**: The building should show a "Waiting for Permit" icon or status in the Inspector.
*   **Hauling Priority**: Permits should be High Priority for haulers.
*   **Corruption**: In the future, allow `Bribe` action to bypass permit requirement for `Credits`.
*   **Tech**: "Automated Bureaucracy" tech could remove the requirement for Tier 2 buildings.

## 6. Acceptance Criteria

- [ ] `BuildingPermit` resource exists.
- [ ] Tier 2+ buildings spawn with `PermitRequired`.
- [ ] `PermitRequired` buildings are inactive (no power draw, no work assignments).
- [ ] `Office` can produce `BuildingPermit`.
- [ ] Haulers carry `BuildingPermit` to the target building.
- [ ] Delivery consumes the permit and activates the building.
- [ ] Test coverage > 85%.

## 7. Technical Guidance

*   **Inventory Issue**: Not all buildings have an `Inventory`. Construction sites usually accept items, but instant-built buildings might not. You might need to add a temporary `Inventory` component or use `Stockpile` logic just for the permit.
    *   *Decision*: Add `Inventory` to `PermitRequired` entities if they don't have it.
*   **Job Filtering**: `009-job-system` iterates available jobs. You must ensure jobs generated by a `PermitRequired` building are either *not generated* or *filtered out*.
    *   *Guidance*: If the building generates dynamic jobs (like "Smelt Ore"), the system generating those jobs should check `!PermitRequired`.

## 8. Questions

*   *Builder*: Should permits stack?
    *Architect:* Yes, they are items.
*   *Builder*: What if I deconstruct the building? Do I get the permit back?
    *Architect:* No, bureaucracy is a sunk cost.
