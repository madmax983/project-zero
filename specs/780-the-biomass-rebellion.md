# 780: The Biomass Rebellion

## 1. Overview
The Biomass Rebellion introduces a terrifying consequence to the highly efficient "Vat-Meat" facilities in the late game. When these facilities suffer from neglect, power failures, or corrupted containment, the synthetic biomass inside mutates. It breaks out of the vats and forms a massive, slow-moving "Flesh Blob" entity. While it doesn't attack Pops directly, it consumes any organic matter in its path—such as crops, corpses, and wooden buildings—growing larger with each consumption. This mechanic balances the incredible space efficiency of vat-meat with the existential risk of a biological breakout.

## 2. Dependencies
- `008-building-farm.md` (for food production and organic matter concepts)
- `138-the-blob.md` (for the base concept of a slow-moving, consuming entity, if applicable)
- `042-energy-system.md` (for power failure triggers)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Setup helper for the test environment
    fn setup_test_app() -> App {
        let mut app = App::new();
        // Register necessary components and events
        app.add_event::<PowerFailureEvent>();
        app.add_event::<BiomassBreakoutEvent>();

        // Add minimal systems needed for the tests
        app.add_systems(Update, (
            trigger_biomass_breakout_system,
            spawn_flesh_blob_system,
            flesh_blob_consumption_system,
            flesh_blob_growth_system,
        ));

        app
    }

    #[test]
    fn test_vat_meat_breakout_on_power_failure() {
        // Arrange
        let mut app = setup_test_app();

        let facility_id = app.world.spawn((
            VatMeatFacility { integrity: 100.0, biomass_level: 50.0 },
            PowerReceiver { is_powered: true },
            GridPosition { x: 10, y: 10 },
        )).id();

        // Act - Simulate a power failure
        app.world.send_event(PowerFailureEvent { entity: facility_id });
        app.update(); // Process the power failure and trigger the breakout event
        app.update(); // Process the breakout event to spawn the Flesh Blob

        // Assert - Verify that a Flesh Blob was spawned at the facility's location
        let mut blob_query = app.world.query::<(&FleshBlob, &GridPosition)>();
        let mut blobs_found = 0;
        for (blob, pos) in blob_query.iter(&app.world) {
            assert_eq!(pos.x, 10);
            assert_eq!(pos.y, 10);
            assert_eq!(blob.size, 1); // Initial size
            blobs_found += 1;
        }
        assert_eq!(blobs_found, 1, "A Flesh Blob should have spawned.");

        // Ensure the facility's biomass level was reset or integrity compromised
        let facility = app.world.get::<VatMeatFacility>(facility_id).unwrap();
        assert_eq!(facility.biomass_level, 0.0, "Biomass should be consumed by the breakout.");
    }

    #[test]
    fn test_flesh_blob_consumes_organic_matter() {
        // Arrange
        let mut app = setup_test_app();

        // Spawn a Flesh Blob
        let blob_id = app.world.spawn((
            FleshBlob { size: 1, consumed_biomass: 0.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Spawn organic matter nearby (e.g., a crop)
        let crop_id = app.world.spawn((
            Crop { yield_amount: 10.0 },
            OrganicMatter, // Marker component
            GridPosition { x: 5, y: 5 },
        )).id();

        // Act - Run the consumption system
        app.update();

        // Assert - Verify the crop was destroyed and the blob gained biomass
        assert!(app.world.get_entity(crop_id).is_none(), "The organic crop should be consumed and despawned.");

        let blob = app.world.get::<FleshBlob>(blob_id).unwrap();
        assert!(blob.consumed_biomass > 0.0, "The Flesh Blob should have gained consumed biomass.");
    }

    #[test]
    fn test_flesh_blob_growth() {
        // Arrange
        let mut app = setup_test_app();

        // Spawn a Flesh Blob with enough consumed biomass to grow
        let blob_id = app.world.spawn((
            FleshBlob { size: 1, consumed_biomass: 100.0 }, // Threshold is let's say 50.0
            GridPosition { x: 5, y: 5 },
        )).id();

        // Act - Run the growth system
        app.update();

        // Assert - Verify the blob's size increased and consumed biomass decreased
        let blob = app.world.get::<FleshBlob>(blob_id).unwrap();
        assert_eq!(blob.size, 2, "The Flesh Blob should have grown to size 2.");
        assert!(blob.consumed_biomass < 100.0, "Consumed biomass should decrease after growth.");
    }

    #[test]
    fn test_flesh_blob_ignores_inorganic_matter() {
        // Arrange
        let mut app = setup_test_app();

        // Spawn a Flesh Blob
        let _blob_id = app.world.spawn((
            FleshBlob { size: 1, consumed_biomass: 0.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Spawn an inorganic building (e.g., a steel wall)
        let wall_id = app.world.spawn((
            Building { is_organic: false },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Act - Run the consumption system
        app.update();

        // Assert - Verify the wall still exists
        assert!(app.world.get_entity(wall_id).is_some(), "The inorganic wall should not be consumed.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Components
#[derive(Component)]
pub struct VatMeatFacility {
    pub integrity: f32,
    pub biomass_level: f32,
}

#[derive(Component)]
pub struct FleshBlob {
    pub size: u32,
    pub consumed_biomass: f32,
}

#[derive(Component)]
pub struct OrganicMatter; // Marker for consumable entities

#[derive(Component)]
pub struct PowerReceiver {
    pub is_powered: bool,
}

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

// Dummy Crop component for test integration
#[derive(Component)]
pub struct Crop {
    pub yield_amount: f32,
}

// Dummy Building component
#[derive(Component)]
pub struct Building {
    pub is_organic: bool,
}

// Events
pub struct PowerFailureEvent {
    pub entity: Entity,
}

pub struct BiomassBreakoutEvent {
    pub facility_entity: Entity,
    pub position: GridPosition,
    pub initial_biomass: f32,
}

// Systems
pub fn trigger_biomass_breakout_system(
    mut power_failure_events: EventReader<PowerFailureEvent>,
    mut facility_query: Query<(&mut VatMeatFacility, &GridPosition)>,
    mut breakout_events: EventWriter<BiomassBreakoutEvent>,
) {
    for event in power_failure_events.read() {
        if let Ok((mut facility, pos)) = facility_query.get_mut(event.entity) {
            // Minimal condition: any power failure triggers breakout if biomass > 0
            if facility.biomass_level > 0.0 {
                breakout_events.send(BiomassBreakoutEvent {
                    facility_entity: event.entity,
                    position: *pos,
                    initial_biomass: facility.biomass_level,
                });
                facility.biomass_level = 0.0; // Reset biomass as it escapes
            }
        }
    }
}

pub fn spawn_flesh_blob_system(
    mut commands: Commands,
    mut breakout_events: EventReader<BiomassBreakoutEvent>,
) {
    for event in breakout_events.read() {
        commands.spawn((
            FleshBlob {
                size: 1,
                consumed_biomass: event.initial_biomass,
            },
            event.position,
        ));
    }
}

pub fn flesh_blob_consumption_system(
    mut commands: Commands,
    mut blob_query: Query<(&mut FleshBlob, &GridPosition)>,
    organic_query: Query<(Entity, &GridPosition), With<OrganicMatter>>,
    building_query: Query<(Entity, &GridPosition, &Building)>,
) {
    for (mut blob, blob_pos) in blob_query.iter_mut() {
        // Consume standard organic matter
        for (organic_entity, organic_pos) in organic_query.iter() {
            if blob_pos == organic_pos {
                commands.entity(organic_entity).despawn();
                blob.consumed_biomass += 10.0; // Arbitrary gain
            }
        }

        // Consume organic buildings (e.g., wooden structures)
        for (building_entity, building_pos, building) in building_query.iter() {
            if building.is_organic && blob_pos == building_pos {
                commands.entity(building_entity).despawn();
                blob.consumed_biomass += 25.0; // Buildings yield more
            }
        }
    }
}

pub fn flesh_blob_growth_system(
    mut blob_query: Query<&mut FleshBlob>,
) {
    let growth_threshold = 50.0;
    for mut blob in blob_query.iter_mut() {
        if blob.consumed_biomass >= growth_threshold {
            blob.size += 1;
            blob.consumed_biomass -= growth_threshold;
            // Note: Increasing `size` might later translate to occupying multiple tiles
            // or having a larger consumption radius/visual representation.
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding & Movement**: The Blob currently doesn't move. In a real scenario, it should slowly wander or expand toward nearby dense organic matter clusters. Integrate with a diffusion or slow-tick pathfinding system.
- **Tile Occupancy**: As `size` increases, the Flesh Blob should occupy more tiles on the `TerrainGrid` (multi-tile entity representation).
- **Vulnerability**: Introduce specific weakness components (e.g., `Flammable` or `VulnerableToIncendiary`) so Pops can fight back effectively using specific weapons.
- **Narrative Integration**: The `BiomassBreakoutEvent` should dispatch a Chronicle event (e.g., `AddChronicleEvent` with the `BLOB_SIGHTING` or `BLOB_CONSUMPTION` template) to alert the player and record the disaster.
- **Refining 'Organic'**: Ensure a consistent `OrganicMatter` trait/component is applied globally to crops, corpses, and wooden materials so the Blob consumes the right targets without excessive ad-hoc `Query` filters.

## 6. Acceptance Criteria
- [ ] All tests in the RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage is ≥85% for the new components and systems.
- [ ] Power failure on a Vat-Meat facility triggers the spawning of a Flesh Blob.
- [ ] The Flesh Blob successfully destroys overlapping organic entities and increases its stored biomass.
- [ ] The Flesh Blob grows in size when its stored biomass exceeds the defined threshold.
- [ ] The Flesh Blob strictly ignores inorganic entities.

## 7. Technical Guidance
- **Integration with Physics/Fire**: Since the lore mentions destroying it with incendiary weapons, ensure the `FleshBlob` has a high flammability multiplier or takes bonus damage from fire-based attacks.
- **System Ordering**: Ensure the consumption system runs *after* movement systems but *before* growth, so it can reliably process overlapping positions.
- **Performance**: If the Blob grows to encompass many tiles, checking collisions via simple coordinate equality might become expensive. Consider representing large Blobs as a distinct area/grid overlay (similar to `PressureGrid`) or using spatial hashing if performance degrades.

## 8. Questions
*Builder: Add any questions here.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
