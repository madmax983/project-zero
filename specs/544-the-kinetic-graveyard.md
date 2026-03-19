# 544 - The Kinetic Graveyard

## 1. Overview
**Layer:** 2 -> 1
**Fantasy:** Living under a sky made of shrapnel.
**Mechanic:** Over decades, a massive, un-minable debris field forms in Layer 2 orbit from destroyed ships. Periodically, "Debris Showers" rain down on Layer 1. The showers ignore basic roofs and require thick "Blast Shielding" or subterranean construction. However, the impacts scatter rare "Scrap Tech" across the map.

## 2. Dependencies
- `184` Orbital Debris
- `153` Geological Instability (for map impact/damage events)
- `018` Mining and Resources (for Scrap Tech harvesting)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_debris_shower_causes_damage_and_spawns_scrap() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_debris_shower_system);

        let building_id = app.world_mut().spawn((
            Building { tile_x: 10, tile_y: 10 },
            Health { current: 100.0, max: 100.0 },
            RoofType::Basic, // Vulnerable to debris
        )).id();

        // Act
        app.world_mut().send_event(DebrisShowerEvent { target_x: 10, target_y: 10, damage: 50.0 });
        app.update();

        // Assert
        let health = app.world().get::<Health>(building_id).unwrap();
        assert_eq!(health.current, 50.0, "Building should take damage from debris shower");

        // Check for scrap spawn
        let mut scrap_spawned = false;
        for scrap in app.world_mut().query::<&ScrapTech>().iter(app.world()) {
            scrap_spawned = true;
        }
        assert!(scrap_spawned, "ScrapTech should spawn at the impact site");
    }

    #[test]
    fn test_blast_shielding_prevents_debris_damage() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_debris_shower_system);

        let building_id = app.world_mut().spawn((
            Building { tile_x: 15, tile_y: 15 },
            Health { current: 100.0, max: 100.0 },
            RoofType::BlastShielding, // Immune/resistant to debris
        )).id();

        // Act
        app.world_mut().send_event(DebrisShowerEvent { target_x: 15, target_y: 15, damage: 50.0 });
        app.update();

        // Assert
        let health = app.world().get::<Health>(building_id).unwrap();
        assert_eq!(health.current, 100.0, "Building with Blast Shielding should not take damage");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Building { pub tile_x: i32, pub tile_y: i32 }

#[derive(Component)]
pub struct Health { pub current: f32, pub max: f32 }

#[derive(Component, PartialEq, Eq)]
pub enum RoofType { Basic, BlastShielding }

#[derive(Event)]
pub struct DebrisShowerEvent { pub target_x: i32, pub target_y: i32, pub damage: f32 }

#[derive(Component)]
pub struct ScrapTech { pub tile_x: i32, pub tile_y: i32 }

pub fn process_debris_shower_system(
    mut commands: Commands,
    mut events: EventReader<DebrisShowerEvent>,
    mut query: Query<(&Building, &mut Health, &RoofType)>,
) {
    for event in events.read() {
        // Spawn scrap at impact site
        commands.spawn(ScrapTech {
            tile_x: event.target_x,
            tile_y: event.target_y,
        });

        // Apply damage to buildings at the impact site
        for (building, mut health, roof_type) in query.iter_mut() {
            if building.tile_x == event.target_x && building.tile_y == event.target_y {
                if *roof_type != RoofType::BlastShielding {
                    health.current -= event.damage;
                    if health.current < 0.0 {
                        health.current = 0.0;
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Refactor:** `DebrisShowerEvent` currently targets a single specific tile. Debris showers should probably affect an area/radius (`radius: i32`) to simulate the scatter of debris.
- **Improvement:** Introduce a mechanism for triggering `DebrisShowerEvent` from Layer 2 (e.g., when a ship is destroyed in orbit or when orbital debris density crosses a threshold).
- **Code Smell:** `ScrapTech` spawning unconditionally over existing structures could cause pathfinding or building placement issues. Ensure scrap spawns cleanly on the terrain grid layer or drops as haulable items.
- **Integration:** Integrate with `ChronicleSystem` to log a "Debris Shower" narrative event when it happens.

## 6. Acceptance Criteria
- [ ] `cargo test` passes.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Debris shower applies damage to buildings with `RoofType::Basic`.
- [ ] `RoofType::BlastShielding` fully negates or heavily reduces debris damage.
- [ ] `ScrapTech` resources spawn at impact sites for subsequent harvesting.
- [ ] Test coverage exceeds 85%.

## 7. Technical Guidance
- If `RoofType` is not an existing component, you may need to attach it to existing `Building` definitions or replace it with whatever structural integrity mechanics exist for roofs/floors.
- For area-of-effect damage, you can compute Manhattan or Euclidean distance from the event's `target_x`/`target_y` coordinates to the building tiles.

## 8. Questions
*Builder: add questions here if spec is unclear.*
