# 542 - The Architectural Palimpsest

## 1. Overview
**Layer:** 1
**Fantasy:** You can tear down the old walls, but the ghosts of the old city still dictate how the new one feels.
**Mechanic:** When a building is demolished, it leaves an invisible "Shadow" tag on those tiles. New buildings constructed over these shadows inherit a fraction of the old building's aura (Beauty, Squalor, or History). If you build a pristine hospital over the site of an old, blood-soaked arena, the hospital will naturally generate a low-level "Dread" aura for patients.

## 2. Dependencies
- `004` Basic Building
- `064` Room Quality

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_demolition_leaves_shadow() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_demolition_system);

        let tile_entity = app.world_mut().spawn(TilePosition { x: 5, y: 5 }).id();
        let building_id = app.world_mut().spawn((
            Building { tile: tile_entity },
            AuraEmitter { aura_type: AuraType::Dread, intensity: 10.0 },
            DemolishMarker,
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get_entity(building_id).is_err(), "Building should be despawned");
        let shadow = app.world().get::<ArchitecturalShadow>(tile_entity).expect("Tile should have a shadow component");
        assert_eq!(shadow.aura_type, AuraType::Dread);
        assert_eq!(shadow.intensity, 5.0, "Shadow intensity should be a fraction of original (e.g., half)");
    }

    #[test]
    fn test_new_building_inherits_shadow() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_new_building_shadow_inheritance_system);

        let tile_entity = app.world_mut().spawn((
            TilePosition { x: 5, y: 5 },
            ArchitecturalShadow { aura_type: AuraType::Squalor, intensity: 4.0 },
        )).id();

        let new_building_id = app.world_mut().spawn((
            Building { tile: tile_entity },
            JustBuiltMarker,
        )).id();

        // Act
        app.update();

        // Assert
        let aura = app.world().get::<AuraEmitter>(new_building_id).unwrap();
        assert_eq!(aura.aura_type, AuraType::Squalor);
        assert_eq!(aura.intensity, 4.0, "Building inherits the shadow aura");
        assert!(app.world().get::<JustBuiltMarker>(new_building_id).is_none(), "Marker should be removed");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct TilePosition { pub x: i32, pub y: i32 }
#[derive(Component)]
pub struct Building { pub tile: Entity }
#[derive(Component)]
pub struct DemolishMarker;
#[derive(Component)]
pub struct JustBuiltMarker;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AuraType { Dread, Beauty, Squalor, History }

#[derive(Component, Clone, Copy)]
pub struct AuraEmitter { pub aura_type: AuraType, pub intensity: f32 }

#[derive(Component, Clone, Copy)]
pub struct ArchitecturalShadow { pub aura_type: AuraType, pub intensity: f32 }

pub fn process_demolition_system(
    mut commands: Commands,
    query: Query<(Entity, &Building, Option<&AuraEmitter>), With<DemolishMarker>>,
) {
    for (entity, building, aura_opt) in query.iter() {
        if let Some(aura) = aura_opt {
            commands.entity(building.tile).insert(ArchitecturalShadow {
                aura_type: aura.aura_type,
                intensity: aura.intensity * 0.5,
            });
        }
        commands.entity(entity).despawn();
    }
}

pub fn process_new_building_shadow_inheritance_system(
    mut commands: Commands,
    query_buildings: Query<(Entity, &Building), With<JustBuiltMarker>>,
    query_tiles: Query<&ArchitecturalShadow>,
) {
    for (entity, building) in query_buildings.iter() {
        if let Ok(shadow) = query_tiles.get(building.tile) {
            commands.entity(entity).insert(AuraEmitter {
                aura_type: shadow.aura_type,
                intensity: shadow.intensity,
            });
        }
        commands.entity(entity).remove::<JustBuiltMarker>();
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Refactor:** The `process_demolition_system` assumes one building per tile. Handle multi-tile buildings gracefully by applying the shadow to all underlying tiles.
- **Improvement:** Combine inherited shadow auras with the new building's base aura if it already has one, rather than just blindly inserting `AuraEmitter`.
- **Code Smell:** `JustBuiltMarker` pattern could be improved by using standard Bevy `Added<Building>` query filters instead of an explicit marker component.
- **Integration:** Integrate the `AuraEmitter` effects to actually influence Room Quality and Pop Morale.

## 6. Acceptance Criteria
- [ ] `cargo test` passes.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Demolishing a building with an aura creates an `ArchitecturalShadow` on its tile(s) with partial intensity.
- [ ] Newly built structures on tiles with a shadow acquire or combine the shadow's aura.
- [ ] Test coverage exceeds 85%.

## 7. Technical Guidance
- Bevy's `Added<T>` query filter is generally preferred over a custom `JustBuiltMarker`. It detects components added during the previous tick.
- If a building spans multiple tiles, iterate through its footprint and read the shadows. You may want to average the shadow intensity or take the highest one.

## 8. Questions
*Builder: add questions here if spec is unclear.*
