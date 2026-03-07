# 409-Shipbreaking

## 1. Overview
**Layer:** 1
**Fantasy:** Scavenging the bones of giants. Living in the wreckage.
**Mechanic:** Crashed ships spawn as multi-tile, indestructible "Hull" terrain. Mining it yields refined alloys or high-tech components but requires high-tier tools. Rooms can be built *inside* the hull, using it as pre-built (but oddly shaped) walls.
**Emergence:** A poor colony survives only because a cruiser crashed nearby. They live inside the engine block for warmth.
**Tension:** Rapid wealth (mining the hull for scrap) vs. Defense/Shelter (keeping the hull for protection).

## 2. Dependencies
- Terrain system (must support indestructible / mineable multi-tile structures).
- Building placement system (must allow building rooms inside 'Hull' terrain).
- Tool economy / Mining system (mining requires high-tier tools).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_shipbreaking_hull_generation() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // ... setup terrain ...

        // Act
        // Spawn a crashed ship
        app.world_mut().send_event(SpawnCrashedShipEvent {
            location: IVec2::new(10, 10),
            ship_type: ShipType::Cruiser,
        });
        app.update();

        // Assert
        // Check if multiple Hull tiles are spawned
        let hull_tiles: Vec<_> = app.world_mut().query::<&HullTile>().iter(&app.world()).collect();
        assert!(!hull_tiles.is_empty(), "Crashed ship should spawn Hull tiles");
    }

    #[test]
    fn test_mining_hull_requires_tool() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let hull_entity = app.world_mut().spawn((
            HullTile,
            Mineable { required_tool_tier: 3 },
            Health::new(100.0)
        )).id();

        let miner_entity = app.world_mut().spawn(Miner { tool_tier: 1 }).id();

        // Act
        // Attempt to mine
        app.world_mut().send_event(MineEvent { miner: miner_entity, target: hull_entity });
        app.update();

        // Assert
        // Health should be unchanged
        let health = app.world().get::<Health>(hull_entity).unwrap();
        assert_eq!(health.current, 100.0, "Miner with low tier tool should not damage Hull tile");
    }

    #[test]
    fn test_mining_hull_yields_alloys() {
         // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let hull_entity = app.world_mut().spawn((
            HullTile,
            Mineable { required_tool_tier: 3 },
            Health::new(10.0), // Low health for easy mining
            YieldsOnMine(vec![Resource::RefinedAlloy]),
        )).id();

        let miner_entity = app.world_mut().spawn(Miner { tool_tier: 3 }).id();

        // Act
        // Mine the tile
        app.world_mut().send_event(MineEvent { miner: miner_entity, target: hull_entity });
        app.update();

        // Assert
        // Check for spawned resources or added to inventory
        // (Assuming a system handles drops)
        let drops: Vec<_> = app.world_mut().query::<&DroppedResource>().iter(&app.world()).collect();
        assert!(!drops.is_empty(), "Mining Hull should yield resources");
    }

    #[test]
    fn test_building_inside_hull() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn hull walls shaping a small room
        app.world_mut().spawn((HullTile, Transform::from_translation(Vec3::new(0., 0., 0.))));
        app.world_mut().spawn((HullTile, Transform::from_translation(Vec3::new(1., 0., 0.))));
        app.world_mut().spawn((HullTile, Transform::from_translation(Vec3::new(2., 0., 0.))));
        // ... imagine a U shape

        // Act
        // Try placing a bed inside
        let placement_result = try_place_building(
            &mut app.world_mut(),
            BuildingType::Bed,
            Vec3::new(1., 1., 0.)
        );

        // Assert
        assert!(placement_result.is_ok(), "Should be able to place buildings inside hull boundaries");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass

#[derive(Component)]
pub struct HullTile;

#[derive(Component)]
pub struct Mineable {
    pub required_tool_tier: u8,
}

#[derive(Component)]
pub struct YieldsOnMine(pub Vec<Resource>);

pub struct MineEvent {
    pub miner: Entity,
    pub target: Entity,
}

pub fn mine_system(
    mut events: EventReader<MineEvent>,
    mut target_query: Query<(&Mineable, &mut Health)>,
    miner_query: Query<&Miner>,
) {
    for event in events.read() {
        if let Ok(miner) = miner_query.get(event.miner) {
            if let Ok((mineable, mut health)) = target_query.get_mut(event.target) {
                if miner.tool_tier >= mineable.required_tool_tier {
                    health.take_damage(10.0);
                }
            }
        }
    }
}

pub fn hull_destroyed_system(
    mut commands: Commands,
    query: Query<(Entity, &Health, &YieldsOnMine, &Transform), With<HullTile>>,
) {
    for (entity, health, yields, transform) in query.iter() {
        if health.current <= 0.0 {
            // Spawn yields
            commands.spawn((DroppedResource, transform.clone()));
            commands.entity(entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:**
  - Extract the `MineEvent` logic into a generic gathering system if not already present.
  - Implement a proper drop table system instead of a simple `YieldsOnMine` component.
  - Integrate `HullTile` seamlessly into the room calculation logic (treating them as indestructible walls).
- **Code Smells:**
  - Hardcoded damage values in the mining system (should depend on tool stats).
- **Performance Considerations:**
  - Generating large crashed ships should not block the main thread; consider staggered spawning or procedural generation during map load.
- **API Improvements:**
  - `try_place_building` should dynamically check collision with `HullTile`s to allow building "inside" but not "on top" of them.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Hull tiles correctly require high-tier tools to mine.
- [ ] Mining a Hull tile drops the correct resources.
- [ ] Players can place buildings within the confines of a crashed ship hull.

## 7. Technical Guidance
- **Code Structure:** Create a new module `src/layer1/shipbreaking.rs` for specific logic, and integrate hull tiles into the existing `terrain` module.
- **Integration Points:**
  - Terrain generation (spawning the crash site).
  - Mining/gathering systems (handling the tool tier requirements).
  - Room/building placement systems (handling hull tiles as valid walls).
- **Gotchas:** Ensure that `HullTile`s block pathfinding but not placement *adjacent* to them (forming rooms).

## 8. Questions
*Builder: add questions here if spec is unclear.*
