# 641 - Aerostat Habitats

## 1. Overview
**Layer:** 2 -> 1
**Fantasy:** Floating cities drifting through the upper atmosphere of gas giants, harvesting rare gases while constantly fighting to stay aloft.
**Mechanic:** On Gas Giant planets, colonies must be built on constructible "Aerostat Platforms." These platforms require constant fuel and maintenance to maintain buoyancy. The planet has no solid ground; if a platform loses buoyancy, it slowly sinks into the crushing depths until it is destroyed.

## 2. Dependencies
- Layer 1 `Terrain` / `Grid` system to handle empty sky/abyss tiles.
- Layer 1 `Maintenance` or `Fuel` consumption systems.
- Layer 2 `PlanetType` traits to enforce gas giant rules.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_aerostat_platform_sinks_without_fuel() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_aerostat_buoyancy_system);

        // Spawn a platform over a Gas Giant abyss tile
        let platform = app.world.spawn((
            AerostatPlatform,
            Buoyancy { fuel: 0, required_fuel: 10, altitude: 100 },
            Position { x: 5, y: 5, z: 10 },
        )).id();

        // Act: One tick without fuel
        app.update();

        // Assert: Altitude should decrease
        let buoyancy = app.world.get::<Buoyancy>(platform).unwrap();
        assert!(buoyancy.altitude < 100);
    }

    #[test]
    fn test_aerostat_platform_destroyed_at_crush_depth() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, check_crush_depth_system);

        let platform = app.world.spawn((
            AerostatPlatform,
            Buoyancy { fuel: 0, required_fuel: 10, altitude: 0 }, // Crush depth is 0
            Position { x: 5, y: 5, z: 10 },
        )).id();

        // Act
        app.update();

        // Assert: The platform is marked for destruction
        assert!(app.world.get::<AerostatPlatform>(platform).is_none());
    }

    #[test]
    fn test_building_on_gas_giant_requires_platform() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, validate_construction_system);
        app.insert_resource(PlanetType::GasGiant);
        app.insert_resource(TerrainGrid::new(10, 10)); // All tiles are 'Abyss' by default on Gas Giant

        let building_intent = app.world.spawn((
            ConstructionIntent { building_type: BuildingType::Hab },
            Position { x: 5, y: 5, z: 10 },
        )).id();

        // Act
        app.update();

        // Assert: The construction should be invalid because there is no AerostatPlatform at this position
        let intent = app.world.get::<ConstructionIntent>(building_intent).unwrap();
        assert_eq!(intent.is_valid, false);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Resource, PartialEq, Eq)]
pub enum PlanetType {
    Terrestrial,
    GasGiant,
}

#[derive(Component)]
pub struct AerostatPlatform;

#[derive(Component)]
pub struct Buoyancy {
    pub fuel: u32,
    pub required_fuel: u32,
    pub altitude: i32,
}

pub fn process_aerostat_buoyancy_system(mut query: Query<&mut Buoyancy, With<AerostatPlatform>>) {
    for mut buoyancy in query.iter_mut() {
        if buoyancy.fuel >= buoyancy.required_fuel {
            buoyancy.fuel -= buoyancy.required_fuel;
            // Optionally increase altitude if they are rising back up, but let's keep it simple
        } else {
            buoyancy.fuel = 0;
            buoyancy.altitude -= 10;
        }
    }
}

pub fn check_crush_depth_system(
    mut commands: Commands,
    query: Query<(Entity, &Buoyancy), With<AerostatPlatform>>
) {
    for (entity, buoyancy) in query.iter() {
        if buoyancy.altitude <= 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// In validate_construction_system, add a check:
// if planet == GasGiant && !has_platform_at(pos) { intent.is_valid = false; }
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells:** Despawning the platform immediately might orphan entities on top of it. We need a clean cascade destruction system.
- **Performance:** `process_aerostat_buoyancy_system` runs every tick for every platform. This is fine since platforms are rare/large.
- **API Improvements:** Send an `AerostatSinkingEvent` and `AerostatCrushedEvent` to integrate with the Chronicle and alert the player before it's too late.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Buildings cannot be placed on Gas Giants without a platform beneath them.
- [ ] Platforms sink without fuel and are destroyed at altitude 0.

## 7. Technical Guidance
- `TerrainGrid` needs a new tile type for Gas Giants, e.g., `TileType::Abyss` or `Sky`.
- Make sure that when a platform is destroyed, everything standing on it (Pops, buildings) is also destroyed or marked as falling.
- Fuel consumption should tie into the main inventory or logistics network so drones can refuel them.

## 8. Questions
*Builder: add questions here if spec is unclear.*
