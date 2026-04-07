# 841: Radioactive Hearth

## 1. Overview
In extreme cold biomes, players face a dire choice: freeze to death or use radioactive items (Waste, Ore) as makeshift heaters. Radioactive items emit heat, which can save Pops from freezing but causes slow, cumulative radiation sickness. This creates a desperate tension where the warmest place in the colony is also the deadliest.

## 2. Dependencies
- Temperature and Thermal systems (`Temperature`, `ThermalSource`).
- Inventory and Item types (`Inventory`, `ItemType`).
- Pop health and condition tracking (`Health`, `RadiationSickness`, `Freezing`).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_radioactive_items_emit_heat() {
        let mut app = App::new();
        app.add_systems(Update, radioactive_heat_emission_system);

        // Arrange
        let item_entity = app.world_mut().spawn((
            Position { x: 0, y: 0 },
            Item { item_type: ItemType::RadioactiveWaste },
            ThermalSource { output: 0 }, // Should be updated
        )).id();

        // Act
        app.update();

        // Assert
        let source = app.world().get::<ThermalSource>(item_entity).unwrap();
        assert!(source.output > 0, "Radioactive waste should emit heat");
    }

    #[test]
    fn test_pops_near_radiation_get_sick() {
        let mut app = App::new();
        app.add_systems(Update, radiation_exposure_system);

        // Arrange
        app.world_mut().spawn((
            Position { x: 1, y: 1 },
            Item { item_type: ItemType::RadioactiveOre },
        ));

        let pop_entity = app.world_mut().spawn((
            Position { x: 1, y: 1 },
            Pop,
            RadiationSickness { level: 0.0 },
        )).id();

        // Act
        app.update();

        // Assert
        let sickness = app.world().get::<RadiationSickness>(pop_entity).unwrap();
        assert!(sickness.level > 0.0, "Pop near radioactive material should gain radiation sickness");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, Clone, PartialEq)]
pub enum ItemType {
    RadioactiveWaste,
    RadioactiveOre,
    StandardRation,
}

#[derive(Component)]
pub struct Item {
    pub item_type: ItemType,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct ThermalSource {
    pub output: i32,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct RadiationSickness {
    pub level: f32,
}

pub fn radioactive_heat_emission_system(
    mut query: Query<(&Item, &mut ThermalSource)>,
) {
    for (item, mut source) in query.iter_mut() {
        if matches!(item.item_type, ItemType::RadioactiveWaste | ItemType::RadioactiveOre) {
            source.output = 50; // Arbitrary heat value
        }
    }
}

pub fn radiation_exposure_system(
    radioactive_items: Query<&Position, With<Item>>,
    mut pops: Query<(&Position, &mut RadiationSickness), With<Pop>>,
) {
    let mut radioactive_positions = Vec::new();
    for pos in radioactive_items.iter() {
        // In a real system we'd filter for actual radioactive item types
        radioactive_positions.push(*pos);
    }

    for (pop_pos, mut sickness) in pops.iter_mut() {
        if radioactive_positions.contains(pop_pos) {
            sickness.level += 1.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Grid:** Use a spatial grid to calculate distance-based radiation decay, rather than just exact coordinate matches.
- **Component Tags:** Introduce a `Radioactive` tag component for items instead of matching on `ItemType` variants. This is cleaner and more extensible.
- **Dose Rates:** `RadiationSickness` should increase based on proximity and the "strength" of the radioactive source.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] Radioactive items correctly emit thermal output.
- [ ] Pops adjacent or on top of radioactive items accrue radiation sickness.

## 7. Technical Guidance
- **Integration with Thermal Map:** Ensure the `ThermalSource` integrates with the existing `temperature.rs` spatial grids, likely utilizing the `ahash` maps Bolt optimized.
- **Balancing:** The heat output needs to be sufficient to stave off freezing, otherwise the player has no reason to employ this desperate tactic.

## 8. Questions
*Builder: add questions here if spec is unclear.*
