# Spec 345: Planetary Quirks

## 1. Overview
Every world is an alien puzzle. Not just generic biome types, but specific "Quirks" that apply global modifiers to Layer 1. High gravity increases movement energy cost. Tidal locking creates permanent day/night zones. These quirks force players to adapt their colonial strategies rather than copying the same build order everywhere.

## 2. Dependencies
- Planet generation/selection mechanics (Layer 2)
- Modifier systems (for global effects like power, movement, temperature)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_high_gravity_increases_movement_cost() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(PlanetaryQuirks {
            active_quirks: vec![Quirk::HighGravity],
        });
        app.add_systems(Update, apply_quirk_modifiers);

        let pop = app.world_mut().spawn(PopMovementCost {
            base_cost: 10.0,
            current_cost: 10.0,
        }).id();

        // Act
        app.update();

        // Assert
        let cost = app.world().get::<PopMovementCost>(pop).unwrap();
        // High gravity should significantly increase the energy cost of moving
        assert!(cost.current_cost > cost.base_cost);
    }

    #[test]
    fn test_tidally_locked_creates_permanent_day() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(PlanetaryQuirks {
            active_quirks: vec![Quirk::TidallyLocked],
        });
        app.add_systems(Update, apply_quirk_lighting_system);

        let solar_panel = app.world_mut().spawn(SolarPanel {
            efficiency: 0.0,
        }).id();

        // Act
        // Simulate a "night" cycle where solar panels normally lose efficiency
        app.insert_resource(GlobalTime { is_night: true });
        app.update();

        // Assert
        let panel = app.world().get::<SolarPanel>(solar_panel).unwrap();
        // If tidally locked, "night" might not apply depending on zone,
        // but for a test, we ensure the efficiency isn't 0.
        assert!(panel.efficiency > 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Quirk {
    HighGravity,
    TidallyLocked,
    CarnivorousFlora,
}

#[derive(Resource, Default)]
pub struct PlanetaryQuirks {
    pub active_quirks: Vec<Quirk>,
}

#[derive(Component)]
pub struct PopMovementCost {
    pub base_cost: f32,
    pub current_cost: f32,
}

#[derive(Resource)]
pub struct GlobalTime {
    pub is_night: bool,
}

#[derive(Component)]
pub struct SolarPanel {
    pub efficiency: f32,
}

pub fn apply_quirk_modifiers(
    quirks: Res<PlanetaryQuirks>,
    mut query: Query<&mut PopMovementCost>,
) {
    if quirks.active_quirks.contains(&Quirk::HighGravity) {
        for mut cost in query.iter_mut() {
            cost.current_cost = cost.base_cost * 1.5; // Example multiplier
        }
    }
}

pub fn apply_quirk_lighting_system(
    quirks: Res<PlanetaryQuirks>,
    time: Res<GlobalTime>,
    mut query: Query<&mut SolarPanel>,
) {
    for mut panel in query.iter_mut() {
        if quirks.active_quirks.contains(&Quirk::TidallyLocked) {
            panel.efficiency = 1.0; // Always 100% on the day side
        } else if time.is_night {
            panel.efficiency = 0.0;
        } else {
            panel.efficiency = 1.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Modifier System**: Hardcoding checks in `apply_quirk_modifiers` is inflexible. The `PlanetaryQuirks` resource should inject global multipliers into a centralized `GlobalModifiers` resource that other systems (like pathfinding) read from.
- **Tidally Locked Zones**: A tidally locked planet isn't just "always day." It has a permanent day side, a permanent night side, and a terminator line. This requires dividing the colony grid into specific lighting zones instead of a global `is_night` boolean.
- **Generation**: Layer 2 (System Generation) needs to procedurally attach these quirks to planets during creation.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for quirk modifier logic.
- [ ] Global modifiers properly applied based on active planet quirks.

## 7. Technical Guidance
- **Layer**: Cross-Layer (Generated in Layer 2, applied in Layer 1).
- Start with 2-3 simple numeric multiplier quirks before attempting complex mechanical ones like `TidallyLocked`. `HighGravity` (movement speed), `DenseAtmosphere` (wind power x2, solar power / 2), and `Volcanic` (geothermal power x2, higher temperature) are good starting points.

## 8. Questions
*Builder: add questions here if spec is unclear.*
