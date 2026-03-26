# Spec 627: The Sabotaged Seed Bank

## 1. Overview
The Sabotaged Seed Bank mechanic allows players to uncover and cultivate a precursor "Golden-Wheat" crop that provides massive food yields and climate resistance. However, this miracle crop has a built-in genetic kill-switch. When a specific "Precursor Signal" is broadcast across the sector (a rare event triggered by neighboring worlds or Layer 2 interactions), all instances of Golden-Wheat instantly turn to toxic ash, destroying the crop and permanently ruining the soil. This forces players to balance the incredible short-term boon against the inevitable, catastrophic collapse of a monoculture they don't control.

## 2. Dependencies
- **Layer 1 Agriculture** (Farms, Crop types, Soil fertility)
- **Layer 1 Environmental Status** (Soil toxicity)
- **Layer 2 / Cross-layer Events** (Precursor Signal trigger)

## 3. RED Phase: Tests First

```rust
// specs/627-sabotaged-seed-bank.md

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Mock components and resources
    #[derive(Component, PartialEq, Debug)]
    enum CropType {
        NormalWheat,
        GoldenWheat,
    }

    #[derive(Component)]
    struct Crop {
        yield_amount: u32,
    }

    #[derive(Component)]
    struct Soil {
        fertility: f32,
        is_toxic: bool,
    }

    #[derive(Event)]
    struct PrecursorSignalEvent;

    #[test]
    fn test_golden_wheat_destroyed_by_signal() {
        // Arrange
        let mut app = App::new();
        app.add_event::<PrecursorSignalEvent>();
        app.add_systems(Update, handle_precursor_signal);

        let soil_entity = app.world.spawn(Soil {
            fertility: 1.0,
            is_toxic: false,
        }).id();

        let crop_entity = app.world.spawn((
            CropType::GoldenWheat,
            Crop { yield_amount: 300 },
            Parent(soil_entity),
        )).id();

        app.world.entity_mut(soil_entity).push_children(&[crop_entity]);

        // Act
        app.world.send_event(PrecursorSignalEvent);
        app.update();

        // Assert
        assert!(app.world.get::<CropType>(crop_entity).is_none(), "Golden Wheat should be destroyed");
        let soil = app.world.get::<Soil>(soil_entity).unwrap();
        assert!(soil.is_toxic, "Soil hosting Golden Wheat should become toxic");
        assert_eq!(soil.fertility, 0.0, "Soil fertility should drop to 0");
    }

    #[test]
    fn test_normal_crops_unaffected_by_signal() {
        // Arrange
        let mut app = App::new();
        app.add_event::<PrecursorSignalEvent>();
        app.add_systems(Update, handle_precursor_signal);

        let soil_entity = app.world.spawn(Soil {
            fertility: 1.0,
            is_toxic: false,
        }).id();

        let crop_entity = app.world.spawn((
            CropType::NormalWheat,
            Crop { yield_amount: 100 },
            Parent(soil_entity),
        )).id();

        app.world.entity_mut(soil_entity).push_children(&[crop_entity]);

        // Act
        app.world.send_event(PrecursorSignalEvent);
        app.update();

        // Assert
        assert!(app.world.get::<CropType>(crop_entity).is_some(), "Normal Wheat should survive");
        let soil = app.world.get::<Soil>(soil_entity).unwrap();
        assert!(!soil.is_toxic, "Soil hosting Normal Wheat should remain non-toxic");
        assert_eq!(soil.fertility, 1.0, "Soil fertility should remain unchanged");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, PartialEq, Debug)]
pub enum CropType {
    NormalWheat,
    GoldenWheat,
}

#[derive(Component)]
pub struct Crop {
    pub yield_amount: u32,
}

#[derive(Component)]
pub struct Soil {
    pub fertility: f32,
    pub is_toxic: bool,
}

#[derive(Event)]
pub struct PrecursorSignalEvent;

pub fn handle_precursor_signal(
    mut commands: Commands,
    mut events: EventReader<PrecursorSignalEvent>,
    crop_query: Query<(Entity, &CropType, &Parent)>,
    mut soil_query: Query<&mut Soil>,
) {
    for _ in events.read() {
        for (crop_entity, crop_type, parent) in crop_query.iter() {
            if *crop_type == CropType::GoldenWheat {
                commands.entity(crop_entity).despawn_recursive();
                if let Ok(mut soil) = soil_query.get_mut(parent.get()) {
                    soil.is_toxic = true;
                    soil.fertility = 0.0;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration Points**: Introduce the "Golden-Wheat" seed discovery as an archaeological dig outcome or a random event. The `PrecursorSignalEvent` should be trigged randomly over time, perhaps driven by a Layer 3 or Layer 2 director system.
- **Toxicity Mechanics**: Ensure that toxic soil in Layer 1 interacts properly with the existing map grid and cannot be simply bypassed by instantly replanting. There should be a "soil remediation" cost or process to recover the land.
- **Event Chronicle**: Add an entry into the Chronicle log when the signal destroys the crops so players have narrative context for the sudden famine.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] `GoldenWheat` crops are immediately destroyed when `PrecursorSignalEvent` fires.
- [ ] `NormalWheat` and other crops are ignored by the kill-switch.
- [ ] Soil that previously hosted `GoldenWheat` becomes toxic and loses its fertility upon trigger.

## 7. Technical Guidance
- Verify that `Parent` relationships between crops and soil tiles map correctly onto the actual Layer 1 `TerrainGrid` implementation.
- You may need to adapt the `Soil` and `Crop` mocked structures to fit into existing `TileMap` or `Farm` structures used in the active project.
- Make sure to register `PrecursorSignalEvent` and the `handle_precursor_signal` system in `simulation.rs`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
