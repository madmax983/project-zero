# 620: Atmospheric Strata

## 1. Overview
The air is different up here. The rich live in the clouds; the poor breathe the smog. Building height matters. "High Altitude" floors have higher Wind/Solar efficiency but exposure to Radiation/Storms. "Low Altitude" floors trap Pollution (Smog) but are shielded from weather. Vertical expansion creates a prestige and efficiency advantage at the cost of vulnerability, while lower levels suffer from accumulated toxicity.

## 2. Dependencies
- Building/Grid coordinate system (needs a Z/Y axis for altitude).
- Pollution/Smog tracking system.
- Weather/Disaster system.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_high_altitude_increases_efficiency_and_storm_risk() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, altitude_efficiency_system);

        let building = app.world_mut().spawn(Building {
            altitude: 10,
            base_efficiency: 100.0,
            current_efficiency: 100.0,
            storm_damage_multiplier: 1.0,
        }).id();

        // Act
        app.update();

        // Assert
        let b = app.world().get::<Building>(building).unwrap();
        assert!(b.current_efficiency > b.base_efficiency, "High altitude should boost efficiency (solar/wind)");
        assert!(b.storm_damage_multiplier > 1.0, "High altitude should increase storm vulnerability");
    }

    #[test]
    fn test_low_altitude_traps_pollution() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, pollution_settling_system);

        let high_tile = app.world_mut().spawn(Tile { altitude: 10, pollution: 50.0 }).id();
        let low_tile = app.world_mut().spawn(Tile { altitude: 0, pollution: 50.0 }).id();

        // Act
        app.update();

        // Assert
        let high_p = app.world().get::<Tile>(high_tile).unwrap().pollution;
        let low_p = app.world().get::<Tile>(low_tile).unwrap().pollution;

        assert!(high_p < 50.0, "Pollution should flow away from high altitudes");
        assert!(low_p > 50.0, "Pollution should settle and accumulate in low altitudes");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Building {
    pub altitude: i32,
    pub base_efficiency: f32,
    pub current_efficiency: f32,
    pub storm_damage_multiplier: f32,
}

#[derive(Component)]
pub struct Tile {
    pub altitude: i32,
    pub pollution: f32,
}

pub fn altitude_efficiency_system(
    mut query: Query<&mut Building>,
) {
    for mut building in query.iter_mut() {
        if building.altitude > 5 {
            let bonus = (building.altitude - 5) as f32 * 0.1; // 10% bonus per level above 5
            building.current_efficiency = building.base_efficiency * (1.0 + bonus);
            building.storm_damage_multiplier = 1.0 + bonus; // Same scaling for vulnerability
        } else {
            building.current_efficiency = building.base_efficiency;
            building.storm_damage_multiplier = 1.0;
        }
    }
}

pub fn pollution_settling_system(
    mut query: Query<&mut Tile>,
) {
    let mut total_pollution_transferred = 0.0;

    // Pass 1: Remove from high altitudes
    for mut tile in query.iter_mut() {
        if tile.altitude > 5 && tile.pollution > 0.0 {
            let flow = tile.pollution * 0.1; // 10% flows down
            tile.pollution -= flow;
            total_pollution_transferred += flow;
        }
    }

    // Pass 2: Add to low altitudes
    let mut low_tiles: Vec<Mut<Tile>> = query.iter_mut().filter(|t| t.altitude <= 5).collect();
    if !low_tiles.is_empty() {
        let split = total_pollution_transferred / low_tiles.len() as f32;
        for tile in low_tiles.iter_mut() {
            tile.pollution += split;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Efficiency Logic:** Differentiate building types. Solar panels get an altitude bonus, but a deep-core mine should not. Check `ItemType` or `BuildingType` before applying the buff.
- **Pollution Flow:** The current `pollution_settling_system` globally redistributes pollution. Refactor this to use a cellular automata model where pollution flows physically from a high tile to adjacent lower tiles.
- **Performance:** Two passes over tiles can be slow on large maps.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Buildings at high altitude gain an efficiency boost and a storm damage debuff.
- [ ] Pollution systematically moves from high-altitude tiles to lower ones.

## 7. Technical Guidance
- **Coordinate System:** Make sure `altitude` ties into the existing 3D grid or visual Z-index so players intuitively see the difference.
- **Visuals:** Add conditional visual effects (smog particles on low tiles, sunshine/glare on high tiles) to communicate the mechanic without relying entirely on UI tooltips.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
