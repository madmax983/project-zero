# 453 - Nanite Fabrication

## 1. Overview
This specification details the implementation of the "Nanite Fabrication" feature (Layer 1). It introduces "Nanoforges", which are advanced buildings capable of producing goods instantly without requiring labor time, converting raw energy/mass directly into items. However, there is a "Containment Breach" risk associated with them. When a breach occurs, it spawns "Grey Goo" tiles that consume adjacent matter (buildings, resources) to replicate. The feature provides an incredible speed advantage for production at the cost of catastrophic, existential risk to the colony.

**Fantasy:** Matter programming. The ultimate convenience, until it eats you.
**Emergence:** You rely on Nanoforges to build your defense fleet. A breach occurs during the launch. The fleet is eaten by its own shipyard.
**Tension:** Instant production (Speed) vs. Existential threat (Safety).

## 2. Dependencies
- `004` Basic Building (Building component)
- `018` Mining and Resources (Resource/Item systems)
- `023` Refining Industry (Crafting logic)
- `013` Schedule and System Execution Ordering (Tick/event updates)

## 3. RED Phase: Tests First

```rust
// tests/integration/nanite_fabrication_tests.rs

use bevy::prelude::*;
use scale::layer1::buildings::{Building, BuildingType};
use scale::layer1::items::{ItemType, Inventory};
use scale::layer1::grid::GridPosition;
use scale::layer1::nanite_fabrication::{
    Nanoforge, GreyGoo, ContainmentBreachEvent,
    nanite_fabrication_system, grey_goo_replication_system
};

#[test]
fn test_nanoforge_instant_production() {
    let mut app = App::new();
    app.add_systems(Update, nanite_fabrication_system);

    // Arrange: Create a nanoforge with input materials
    let mut inventory = Inventory::new();
    inventory.add(ItemType::Energy, 100);
    inventory.add(ItemType::RawMass, 50);

    let forge_entity = app.world_mut().spawn((
        Building { building_type: BuildingType::Nanoforge },
        Nanoforge {
            active_recipe: Some(ItemType::AdvancedAlloy),
            breach_risk: 0.0,
            ..Default::default()
        },
        inventory,
    )).id();

    // Act: Run the system
    app.update();

    // Assert: Check that production happened instantly without labor
    let inventory_after = app.world().get::<Inventory>(forge_entity).unwrap();
    assert!(inventory_after.has_item(ItemType::AdvancedAlloy));
    assert!(inventory_after.get_count(ItemType::Energy) < 100); // Resources consumed
    assert!(inventory_after.get_count(ItemType::RawMass) < 50);
}

#[test]
fn test_nanoforge_containment_breach_event() {
    let mut app = App::new();
    app.add_event::<ContainmentBreachEvent>();
    app.add_systems(Update, nanite_fabrication_system);

    // Arrange: Create a nanoforge with 100% breach risk
    let forge_entity = app.world_mut().spawn((
        Building { building_type: BuildingType::Nanoforge },
        Nanoforge {
            active_recipe: Some(ItemType::AdvancedAlloy),
            breach_risk: 1.0, // Guaranteed breach
            ..Default::default()
        },
        Inventory::new(), // Even with empty inventory, let's say risk check happens
    )).id();

    // Act
    app.update();

    // Assert: Event should be fired
    let events = app.world().resource::<Events<ContainmentBreachEvent>>();
    let mut cursor = events.get_cursor();
    let breach_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(breach_events.len(), 1);
    assert_eq!(breach_events[0].source_entity, forge_entity);
}

#[test]
fn test_grey_goo_replication() {
    let mut app = App::new();
    app.add_systems(Update, grey_goo_replication_system);

    // Arrange: Spawn Grey Goo adjacent to a consumable building
    let pos_goo = GridPosition { x: 10, y: 10, z: 0 };
    let pos_target = GridPosition { x: 10, y: 11, z: 0 };

    let target_entity = app.world_mut().spawn((
        Building { building_type: BuildingType::Storage },
        pos_target,
    )).id();

    app.world_mut().spawn((
        GreyGoo { replication_progress: 1.0 }, // Ready to replicate
        pos_goo,
    ));

    // Act: Run the replication system
    app.update();

    // Assert: The target building should be destroyed and replaced/spawned as Grey Goo
    assert!(app.world().get_entity(target_entity).is_err() || app.world().get::<Building>(target_entity).is_none());

    // Check if new Grey Goo exists at the target position
    let mut new_goo_found = false;
    for (goo, pos) in app.world_mut().query::<(&GreyGoo, &GridPosition)>().iter(app.world()) {
        if pos.x == 10 && pos.y == 11 {
            new_goo_found = true;
            break;
        }
    }
    assert!(new_goo_found, "Grey Goo failed to replicate to adjacent tile.");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/nanite_fabrication.rs

use bevy::prelude::*;
use crate::layer1::buildings::{Building, BuildingType};
use crate::layer1::items::{ItemType, Inventory};
use crate::layer1::grid::GridPosition;
use rand::Rng;

#[derive(Component, Default)]
pub struct Nanoforge {
    pub active_recipe: Option<ItemType>,
    pub breach_risk: f32, // 0.0 to 1.0
}

#[derive(Component)]
pub struct GreyGoo {
    pub replication_progress: f32, // 0.0 to 1.0, 1.0 triggers spread
}

#[derive(Event)]
pub struct ContainmentBreachEvent {
    pub source_entity: Entity,
    pub position: GridPosition,
}

pub fn nanite_fabrication_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Nanoforge, &mut Inventory, Option<&GridPosition>)>,
    mut breach_events: EventWriter<ContainmentBreachEvent>,
) {
    let mut rng = rand::thread_rng();

    for (entity, mut forge, mut inventory, pos_opt) in query.iter_mut() {
        // Roll for breach
        if rng.gen::<f32>() < forge.breach_risk {
            let pos = pos_opt.copied().unwrap_unwrap_or(GridPosition { x: 0, y: 0, z: 0 }); // Fallback for tests
            breach_events.send(ContainmentBreachEvent {
                source_entity: entity,
                position: pos,
            });

            // Optionally, transform forge itself into Grey Goo here or let another system handle it
            continue;
        }

        // Instant Production Logic
        if let Some(recipe) = forge.active_recipe {
            // Simplified recipe check: 10 Energy + 5 RawMass = 1 item
            // In full implementation, link to a proper Recipe system
            let energy_needed = 10;
            let mass_needed = 5;

            if inventory.get_count(ItemType::Energy) >= energy_needed &&
               inventory.get_count(ItemType::RawMass) >= mass_needed {

                inventory.remove(ItemType::Energy, energy_needed);
                inventory.remove(ItemType::RawMass, mass_needed);
                inventory.add(recipe, 1);
            }
        }
    }
}

pub fn grey_goo_replication_system(
    mut commands: Commands,
    mut goo_query: Query<(Entity, &mut GreyGoo, &GridPosition)>,
    target_query: Query<(Entity, &GridPosition, Option<&Building>)>, // Simplified target finding
) {
    for (goo_entity, mut goo, goo_pos) in goo_query.iter_mut() {
        if goo.replication_progress >= 1.0 {
            goo.replication_progress = 0.0; // Reset progress after replication

            // Find an adjacent target to consume
            for (target_entity, target_pos, building_opt) in target_query.iter() {
                // Check adjacency (simplistic orthogonal check)
                if (goo_pos.x - target_pos.x).abs() + (goo_pos.y - target_pos.y).abs() == 1 && goo_pos.z == target_pos.z {
                    if building_opt.is_some() {
                        // Consume!
                        commands.entity(target_entity).despawn_recursive();
                        commands.spawn((
                            GreyGoo { replication_progress: 0.0 },
                            *target_pos,
                        ));
                        break; // Only consume one per tick per goo tile
                    }
                }
            }
        }
    }
}

// Add these systems to a schedule in a setup plugin.
```

## 5. REFACTOR Phase: Quality & Design
- **Integration with Recipe System:** Replace the hardcoded `10 Energy + 5 RawMass` check with the actual `RefiningRecipe` system used by standard crafters.
- **Breach Risk Accumulation:** Add a system where `breach_risk` naturally increases over time or with each production cycle, requiring maintenance actions to reset it.
- **Spatial Queries:** The `grey_goo_replication_system` iterates over all possible targets. This should use the `TerrainGrid` resource to directly look up adjacent cells O(1) instead of an O(N) entity scan.
- **Lore/Chronicle:** Emit a `Major` chronicle event when a `ContainmentBreachEvent` fires, as it is an existential threat.
- **Grey Goo Visuals/UI:** Ensure Grey Goo has distinct visual representations in the rendering layer and raises a severe global alert.

## 6. Acceptance Criteria
- [ ] `cargo test` passes, including the new Nanoforge and Grey Goo tests.
- [ ] Test coverage ≥85% for `nanite_fabrication.rs`.
- [ ] `cargo clippy -- -D warnings` runs cleanly.
- [ ] Nanoforges correctly produce items without requiring an assigned Pop or labor ticks, given input resources.
- [ ] A `ContainmentBreachEvent` triggers based on probability, which leads to the creation of `GreyGoo` entities.
- [ ] `GreyGoo` entities successfully despawn adjacent structures/matter and spawn copies of themselves in those positions over time.

## 7. Technical Guidance
- Create `src/layer1/nanite_fabrication.rs`.
- Add `BuildingType::Nanoforge` to the enum in `src/layer1/buildings.rs` (or similar file defining types).
- Register the systems in `src/layer1/systems/execution.rs` (likely in a `Update` schedule).
- The `ContainmentBreachEvent` should probably be handled by a dedicated system that spawns the initial `GreyGoo` at the forge's location and destroys the forge.
- Ensure the Grey Goo pathfinding/adjacency check prevents OOM/infinite loops by safely capping replication rate or checking grid bounds.

## 8. Questions
*Builder: add questions here if spec is unclear.*
