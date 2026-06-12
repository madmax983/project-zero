# 1303: The Edible Architecture

## 1. Overview
A colony so desperate it begins consuming the very structures keeping it alive. Certain advanced or organic building materials (like bioplastics or mycelial scaffolding) can be repurposed as low-quality food during a famine. Players can manually designate non-essential buildings to be "consumed," destroying the building but providing temporary sustenance for the Pops.

## 2. Dependencies
- `layer1::architecture::building::Building`
- `layer1::economy::inventory::Inventory`
- `layer1::psychology::needs::Needs`
- `layer1::administration::designation::Designation`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::architecture::building::{Building, BuildingType};
    use crate::layer1::administration::designation::{Designation, DesignationType};
    use crate::layer1::economy::inventory::Inventory;
    use crate::layer1::economy::items::{Item, ItemType};
    use crate::layer1::core::map::GridPosition;

    #[test]
    fn test_consume_building_yields_food() {
        let mut app = App::new();
        app.add_systems(Update, consume_edible_architecture_system);

        let building = app.world_mut().spawn((
            Building { building_type: BuildingType::Housing },
            EdibleArchitecture { food_yield: 50 },
            Designation { designation_type: DesignationType::Demolish }, // Using Demolish or a new Consume
            GridPosition { x: 5, y: 5 },
        )).id();

        app.update();

        assert!(app.world().get_entity(building).is_err());

        let mut found_food = false;
        for item in app.world_mut().query::<&Item>().iter(app.world()) {
            if item.item_type == ItemType::Potato {
                found_food = true;
                break;
            }
        }
        assert!(found_food, "Consuming the building should spawn food items (Potato for MVP)");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::architecture::building::Building;
use crate::layer1::administration::designation::{Designation, DesignationType};
use crate::layer1::economy::items::{Item, ItemType};
use crate::layer1::core::map::GridPosition;

#[derive(Component)]
pub struct EdibleArchitecture {
    pub food_yield: u32,
}

pub fn consume_edible_architecture_system(
    mut commands: Commands,
    query: Query<(Entity, &EdibleArchitecture, &Designation, Option<&GridPosition>), With<Building>>,
) {
    for (entity, edible, designation, pos) in query.iter() {
        // Ideally this should use a specific Consume designation
        if designation.designation_type == DesignationType::Demolish {
            commands.entity(entity).despawn_recursive();

            if let Some(pos) = pos {
                commands.spawn((
                    Item { item_type: ItemType::Potato }, // Placeholder for edible architecture rations
                    GridPosition { x: pos.x, y: pos.y },
                ));
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Connect `DesignationType::Consume` to the UI so the player can trigger it.
- Pops should ideally walk to the building and perform a "Deconstruct/Eat" job rather than it instantly turning into food items.
- Consuming architecture should trigger a massive negative morale event/memory ("We ate the walls").

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test --lib scale` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Buildings with `EdibleArchitecture` and the appropriate designation are despawned and yield food items.

## 7. Technical Guidance
- Add `DesignationType::Consume` to the existing `DesignationType` enum in `src/layer1/administration/designation.rs`.
- Ensure the food item spawned is recognizable by Pops' hunger systems. Currently `ItemType::Potato` is used as a stand-in, but a dedicated low-morale synthetic food item could be better.

## 8. Questions
*Builder: add questions here if spec is unclear.*
