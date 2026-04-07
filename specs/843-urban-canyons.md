# 843: Urban Canyons

## 1. Overview
The density of a colony shapes its micro-climate. Tall buildings constructed adjacent to each other create "Urban Canyons" that channel and amplify wind speeds in the streets between them. While this high wind hampers Pop movement (blowing them backward or slowing commutes), it significantly boosts the efficiency of Wind Turbines placed within the canyon. This forces players to weigh space-efficient density against walkability and environmental hostility.

## 2. Dependencies
- Building components (`Structure`, `Height`, `Position`).
- Weather/Atmosphere system (`WindSpeed`, `AtmosphereGrid`).
- Movement systems (`MovementSpeed`, `Pop`).
- Power generation (`WindTurbine`, `PowerOutput`).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_urban_canyon_increases_wind_speed() {
        let mut app = App::new();
        app.add_systems(Update, urban_canyon_wind_system);

        // Arrange: Three tiles, two tall buildings with a gap
        app.world_mut().spawn((Position { x: 0, y: 0 }, Height(5)));
        let street_pos = Position { x: 1, y: 0 };
        app.world_mut().spawn((Position { x: 2, y: 0 }, Height(5)));

        let mut wind_grid = WindGrid::default();
        wind_grid.set_base_wind(street_pos, 10.0);
        app.insert_resource(wind_grid);

        // Act
        app.update();

        // Assert: Wind in the street should be amplified
        let amplified_wind = app.world().resource::<WindGrid>().get_wind(street_pos);
        assert!(amplified_wind > 10.0, "Wind speed should increase between tall buildings");
    }

    #[test]
    fn test_high_wind_slows_pop_movement() {
        let mut app = App::new();
        app.add_systems(Update, wind_movement_penalty_system);

        // Arrange
        let mut wind_grid = WindGrid::default();
        wind_grid.set_base_wind(Position { x: 1, y: 0 }, 50.0); // High wind
        app.insert_resource(wind_grid);

        let pop_entity = app.world_mut().spawn((
            Position { x: 1, y: 0 },
            MovementSpeed(10.0),
            Pop,
        )).id();

        // Act
        app.update();

        // Assert: Pop is slowed
        let speed = app.world().get::<MovementSpeed>(pop_entity).unwrap();
        assert!(speed.0 < 10.0, "Pops should move slower in high wind urban canyons");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Height(pub u32);

#[derive(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component)]
pub struct Pop;

#[derive(Resource, Default)]
pub struct WindGrid {
    pub winds: HashMap<Position, f32>,
}

impl WindGrid {
    pub fn set_base_wind(&mut self, pos: Position, speed: f32) {
        self.winds.insert(pos, speed);
    }
    pub fn get_wind(&self, pos: Position) -> f32 {
        *self.winds.get(&pos).unwrap_or(&0.0)
    }
}

pub fn urban_canyon_wind_system(
    buildings: Query<(&Position, &Height)>,
    mut wind_grid: ResMut<WindGrid>,
) {
    let mut heights = HashMap::new();
    for (pos, height) in buildings.iter() {
        heights.insert(*pos, height.0);
    }

    // Clone keys to avoid mutating while iterating
    let keys: Vec<Position> = wind_grid.winds.keys().cloned().collect();

    for pos in keys {
        // Check for adjacent buildings on X axis
        let left = Position { x: pos.x - 1, y: pos.y };
        let right = Position { x: pos.x + 1, y: pos.y };

        if let (Some(h1), Some(h2)) = (heights.get(&left), heights.get(&right)) {
            if *h1 >= 5 && *h2 >= 5 {
                let current = wind_grid.get_wind(pos);
                wind_grid.winds.insert(pos, current * 2.0); // Amplify
            }
        }
    }
}

pub fn wind_movement_penalty_system(
    wind_grid: Res<WindGrid>,
    mut pops: Query<(&Position, &mut MovementSpeed), With<Pop>>,
) {
    for (pos, mut speed) in pops.iter_mut() {
        let wind = wind_grid.get_wind(*pos);
        if wind > 30.0 {
            speed.0 *= 0.5; // Halve speed in high winds
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **System Ordering:** Ensure `urban_canyon_wind_system` runs *after* base weather generation but *before* `wind_movement_penalty_system`.
- **Directional Wind:** Real wind has a vector. The canyon effect should only amplify wind if the canyon aligns with the wind direction, or it should redirect the wind. Add a `WindDirection` concept.
- **Turbine Integration:** Explicitly spec the interaction with `WindTurbine` components so they scale output based on the grid's local wind speed.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] Wind speed multipliers apply correctly between tall structures.
- [ ] Pops traversing high-wind tiles suffer a quantifiable movement speed penalty.

## 7. Technical Guidance
- **Spatial HashMaps:** Like Bolt's optimizations in `atmosphere.rs`, use `bevy::utils::HashMap` with integer tuple keys for grid storage rather than `std::collections`.
- **Height Abstraction:** Ensure "tall" is defined programmatically (e.g., `Height > 3`) so new building types automatically participate.

## 8. Questions
*Builder: add questions here if spec is unclear.*
