# 572 The Pyrophilic Flora

## 1. Overview
A Layer 1 agricultural mechanic featuring a highly valuable alien crop that only drops its seeds and edible fruit when exposed to open flame. Players must intentionally set fire to the crop field to harvest it, creating extreme tension between economic/nutritional value and the terrifying risk of a controlled burn spreading and destroying the colony.

## 2. Dependencies
- Layer 1 `GridPosition` and `Terrain` (for fields)
- `Fire` system (spreading, ignition, extinction)
- `Plant` lifecycle and `Harvestable` component
- Hauler utility AI for rescuing resources from active fires

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_pyrophilic_crop_does_not_drop_yield_on_standard_harvest() {
        let mut app = App::new();
        // Setup ...
        let plant_entity = app.world.spawn((
            Plant::PyrophilicFlora,
            GrowthStage::Mature,
            Harvestable { default_yield: 0 },
        )).id();

        app.world.send_event(HarvestEvent { target: plant_entity });
        app.update();

        // Assert no items spawned
        assert_eq!(app.world.query::<&Item>().iter(&app.world).count(), 0);
    }

    #[test]
    fn test_pyrophilic_crop_drops_yield_when_on_fire() {
        let mut app = App::new();
        // Setup ...
        let pos = GridPosition { x: 10, y: 10 };
        let plant_entity = app.world.spawn((
            Plant::PyrophilicFlora,
            GrowthStage::Mature,
            pos,
        )).id();

        app.world.spawn((Fire, pos)); // Ignite the tile

        app.update();

        // Assert plant is destroyed/consumed and yield items (BurntSeeds, PyrophilicFruit) are spawned at pos
        let mut query = app.world.query::<(&ItemType, &GridPosition)>();
        let has_yield = query.iter(&app.world).any(|(item, p)| *p == pos && *item == ItemType::PyrophilicFruit);
        assert!(has_yield);
        assert!(app.world.get::<Plant>(plant_entity).is_none()); // Plant destroyed
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub enum Plant {
    PyrophilicFlora,
    // ...
}

pub fn pyrophilic_ignition_harvest_system(
    mut commands: Commands,
    plants: Query<(Entity, &Plant, &GrowthStage, &GridPosition)>,
    fires: Query<&GridPosition, With<Fire>>,
) {
    for (entity, plant, stage, plant_pos) in plants.iter() {
        if matches!(plant, Plant::PyrophilicFlora) && *stage == GrowthStage::Mature {
            // If there's a fire on the exact same tile
            if fires.iter().any(|fire_pos| fire_pos == plant_pos) {
                // Destroy plant
                commands.entity(entity).despawn();
                // Spawn yield
                commands.spawn((
                    ItemType::PyrophilicFruit,
                    *plant_pos,
                ));
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Checking `fires.iter().any()` for every plant is O(N*M). Use a grid lookup (`HashMap<GridPosition, Entity>`) for fires to make it O(1) per plant.
- **Performance**: Fire spread is already computationally heavy; ensure `pyrophilic_ignition_harvest_system` runs *after* fire spread to accurately catch newly ignited tiles.
- **Integration**: The items spawned (`ItemType::PyrophilicFruit`) need a `Flammable` component so they can burn up if haulers don't grab them fast enough.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pyrophilic Flora yields nothing on standard harvest.
- [ ] Pyrophilic Flora yields its fruit/seeds immediately upon sharing a tile with a `Fire` entity.

## 7. Technical Guidance
- The items spawned by the fire harvest must have a short grace period before the fire consumes them, giving Pops a chance to haul them.
- Consider adding a `ControlledBurnOrder` utility task to let Pops safely ignite specific tiles.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
