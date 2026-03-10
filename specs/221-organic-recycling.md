# 221 - Organic Recycling

> "In the deep void, carbon is carbon. You don't waste atoms."

## 1. Overview

This feature introduces the **Recycler** building, allowing the colony to convert biological waste (`Corpse` entities and `ResourceType::Waste`) into `ResourceType::Rations` (Nutrient Paste). This creates a grim but necessary survival loop during famines.

### User Story
As a Colony Commander, I want to recycle dead bodies and organic waste into food so that my colony can survive when conventional farming fails, even at the cost of morale.

## 2. Dependencies

*   `005` Pop Needs (Hunger) - Rations satisfy hunger.
*   `034` Pop Health (Corpses) - Corpses are inputs.
*   `032` Entropy & Spoilage (Waste) - Waste is an input.
*   `031` Pop Morale - Eating recycled food causes morale penalties.

## 3. RED Phase: Tests First

These tests define the behavior of the Recycler and the consequences of using it.

```rust
// tests/layer1/recycling_tests.rs

use scale::layer1::resources::{ColonyResources, ResourceType};
use scale::layer1::funeral::Corpse;
use scale::layer1::building::Recycler;
use scale::layer1::morale::Morale;
use scale::layer1::traits::{Trait, Traits};

#[test]
fn test_recycler_processes_corpse() {
    // Arrange
    let mut world = World::new();
    let recycler = world.spawn(Recycler::default()).id();
    let corpse = world.spawn(Corpse::default()).id();

    // Simulate Corpse being hauled to Recycler
    // (In integration, this would use HaulAction, but here we test the processing logic directly)
    world.entity_mut(recycler).insert(Inventory {
        items: vec![corpse] // Conceptual: Recycler holding the corpse
    });

    // Act
    run_recycling_system(&mut world);

    // Assert
    // Corpse should be despawned
    assert!(world.get_entity(corpse).is_err());

    // Rations should be added to output or colony resources
    let resources = world.resource::<ColonyResources>();
    assert!(resources.rations > 0.0);
}

#[test]
fn test_recycler_processes_waste() {
    // Arrange
    let mut world = World::new();
    let recycler = world.spawn((Recycler::default(), Inventory::with_resource(ResourceType::Waste, 10.0))).id();

    // Act
    run_recycling_system(&mut world);

    // Assert
    // Waste consumed
    let inventory = world.get::<Inventory>(recycler).unwrap();
    assert_eq!(inventory.get_amount(ResourceType::Waste), 0.0);

    // Rations produced (Ratio e.g., 2 Waste -> 1 Ration)
    let resources = world.resource::<ColonyResources>();
    assert_eq!(resources.rations, 5.0);
}

#[test]
fn test_eating_rations_causes_gloom() {
    // Arrange
    let mut world = World::new();
    let pop = world.spawn((
        Pop::default(),
        Morale::default(),
        Needs { hunger: 0.1, ..Default::default() }
    )).id();

    // Act
    // Simulate eating Rations
    eat_food(&mut world, pop, ResourceType::Rations);

    // Assert
    let morale = world.get::<Morale>(pop).unwrap();
    assert!(morale.has_modifier("Ate Slop")); // Negative modifier
}

#[test]
fn test_cannibal_trait_ignores_gloom() {
    // Arrange
    let mut world = World::new();
    let pop = world.spawn((
        Pop::default(),
        Morale::default(),
        Traits::new(vec![Trait::Cannibal])
    )).id();

    // Act
    eat_food(&mut world, pop, ResourceType::Rations);

    // Assert
    let morale = world.get::<Morale>(pop).unwrap();
    assert!(!morale.has_modifier("Ate Slop"));
    // Optionally: Might even get a positive modifier "Good Eating"
}
```

## 4. GREEN Phase: Minimal Implementation

1.  **Define Building**: Add `Recycler` variant to `BuildingType`.
2.  **Define Conversion Logic**:
    *   `recycle_processing_system`: Query `Recycler` buildings with `Inventory`.
    *   If `Inventory` contains `Corpse`, despawn it and add `ResourceType::Rations` (e.g., 50.0).
    *   If `Inventory` contains `ResourceType::Waste`, consume it and add `ResourceType::Rations` (Ratio 2:1).
3.  **Define Consumption Logic**:
    *   Modify `eat_action` or `satisfy_hunger_system`.
    *   If `FoodSource` was `Rations`:
        *   Check Pop Traits.
        *   If not `Cannibal` or `Pragmatist`, apply `MoodModifier` ("Ate Slop", -10, 24h).

## 5. REFACTOR Phase: Quality & Design

*   **Job Integration**: Ensure `Haul` jobs correctly target `Recycler` for Corpses (currently they target Graves). Priority issue?
    *   *Solution*: `Recycler` should have higher priority than `Grave` if active, or player sets policy.
*   **Disease Risk**: Eating Waste-based rations should probably carry a disease risk (check `034 Pop Health`).
*   **Balance**: Tune the Waste->Ration ratio. Infinite food glitch if Pop -> Waste -> Ration -> Pop is > 100% efficient. Thermodynamics says no.

## 6. Acceptance Criteria

- [ ] `Recycler` building can be placed and built.
- [ ] Corpses can be hauled to `Recycler`.
- [ ] Waste can be hauled to `Recycler`.
- [ ] Processing generates `Rations` in `ColonyResources`.
- [ ] Pops eating `Rations` suffer Morale penalty (unless Trait prevents it).
- [ ] Test coverage > 85%.

## 7. Technical Guidance

*   **Corpse Handling**: `Corpse` is an Entity, not a `ResourceItem`. The Inventory system needs to handle Entities (or `Recycler` has a special "Occupant" slot like a bed).
    *   *Guidance*: Reuse `Carrying` logic. If `Inventory` only supports `ResourceType`, you might need to convert Corpse to `ResourceType::OrganicMatter` first, OR make `Recycler` act like a `Grave` that processes instantly.
*   **Food Source Tracking**: The `SatisfyHunger` action needs to know *what* was eaten. Currently, it might just decrement `ColonyResources.food`.
    *   *Guidance*: `ColonyResources.rations` is separate from `.food`. Ensure the AI explicitly chooses to eat Rations (perhaps only when `.food` is empty, or if Rations are closer?).

## 8. Questions

*   *Builder*: Should `Recycler` require power?
    *Architect:* Yes, assumed standard building.
*   *Builder*: Does `Waste` come from `ColonyResources.waste` or items on the ground?
    *Architect:* Both. Haulers pick up items, Industry outputs to `ColonyResources.waste`. `Recycler` should probably pull from `ColonyResources.waste` via a worker job, or have haulers bring items.
