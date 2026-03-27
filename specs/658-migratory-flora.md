# 658: Migratory Flora

## 1. Overview
**Layer:** 1
**Fantasy:** The forest is not a location; it is a slow-moving herd.
**Mechanic:** Certain plant species slowly shift position (1 tile per week) towards preferred conditions (Water, Light) or away from threats (Pollution, Fire).
**Emergence:** You build a lumber mill next to the "Ironwood Grove". Over a year, the grove migrates up the mountain, leaving your mill stranded and useless.
**Tension:** Chase the resources (mobile camps) or try to pen them in (walls/bait)?

## 2. Dependencies
- Layer 1 Map/Tile System (`Tilemap`, `TilePos`)
- Layer 1 Flora/Growth System (`Flora`, `GrowthStage`)
- Layer 1 Environmental Factors (`Pollution`, `Water`, `Light`, `Temperature`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_migratory_flora_moves_towards_water() {
        let mut app = App::new();
        app.add_systems(Update, process_flora_migration);
        app.insert_resource(Time::default());

        let mut tilemap = Tilemap::new(10, 10);
        tilemap.set_water(5, 5, 100.0); // High water at 5,5
        app.insert_resource(tilemap);

        let flora = app.world_mut().spawn((
            Flora { species: "Ironwood".to_string() },
            MigratoryFlora { speed: 1.0, preferred_condition: Condition::Water },
            TilePos { x: 2, y: 2 },
        )).id();

        // Advance time enough to trigger migration (e.g. 1 week in simulation time)
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(604800));
        app.update();

        let pos = app.world().get::<TilePos>(flora).unwrap();
        // It should have moved one step closer to (5,5)
        assert!(pos.x > 2 || pos.y > 2, "Flora should migrate towards higher water concentration");
    }

    #[test]
    fn test_migratory_flora_moves_away_from_pollution() {
        let mut app = App::new();
        app.add_systems(Update, process_flora_migration);
        app.insert_resource(Time::default());

        let mut tilemap = Tilemap::new(10, 10);
        tilemap.set_pollution(5, 5, 100.0); // High pollution at 5,5
        app.insert_resource(tilemap);

        let flora = app.world_mut().spawn((
            Flora { species: "Ironwood".to_string() },
            MigratoryFlora { speed: 1.0, preferred_condition: Condition::LowPollution },
            TilePos { x: 4, y: 4 },
        )).id();

        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(604800));
        app.update();

        let pos = app.world().get::<TilePos>(flora).unwrap();
        // It should have moved one step further away from (5,5)
        assert!(pos.x < 4 || pos.y < 4, "Flora should migrate away from pollution");
    }

    #[test]
    fn test_walls_block_migration() {
        let mut app = App::new();
        app.add_systems(Update, process_flora_migration);
        app.insert_resource(Time::default());

        let mut tilemap = Tilemap::new(10, 10);
        tilemap.set_water(5, 5, 100.0);
        tilemap.set_wall(3, 3, true); // Wall blocks path from 2,2 to 5,5
        app.insert_resource(tilemap);

        let flora = app.world_mut().spawn((
            Flora { species: "Ironwood".to_string() },
            MigratoryFlora { speed: 1.0, preferred_condition: Condition::Water },
            TilePos { x: 2, y: 2 },
        )).id();

        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(604800));
        app.update();

        let pos = app.world().get::<TilePos>(flora).unwrap();
        // Movement should be blocked by the wall
        assert_eq!(*pos, TilePos { x: 2, y: 2 }, "Wall should block migration path");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Flora {
    pub species: String,
}

#[derive(Component)]
pub struct MigratoryFlora {
    pub speed: f32,
    pub preferred_condition: Condition,
}

#[derive(PartialEq)]
pub enum Condition {
    Water,
    LowPollution,
}

#[derive(Component, Debug, PartialEq)]
pub struct TilePos {
    pub x: u32,
    pub y: u32,
}

#[derive(Resource)]
pub struct Tilemap {
    pub width: u32,
    pub height: u32,
    pub water: Vec<f32>,
    pub pollution: Vec<f32>,
    pub walls: Vec<bool>,
}

impl Tilemap {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            width: w,
            height: h,
            water: vec![0.0; (w * h) as usize],
            pollution: vec![0.0; (w * h) as usize],
            walls: vec![false; (w * h) as usize],
        }
    }

    pub fn get_index(&self, x: u32, y: u32) -> usize { (y * self.width + x) as usize }
    pub fn set_water(&mut self, x: u32, y: u32, val: f32) { let idx = self.get_index(x, y); self.water[idx] = val; }
    pub fn set_pollution(&mut self, x: u32, y: u32, val: f32) { let idx = self.get_index(x, y); self.pollution[idx] = val; }
    pub fn set_wall(&mut self, x: u32, y: u32, val: bool) { let idx = self.get_index(x, y); self.walls[idx] = val; }

    pub fn get_water(&self, x: u32, y: u32) -> f32 { self.water[self.get_index(x,y)] }
    pub fn get_pollution(&self, x: u32, y: u32) -> f32 { self.pollution[self.get_index(x,y)] }
    pub fn is_wall(&self, x: u32, y: u32) -> bool { self.walls[self.get_index(x,y)] }
}

pub fn process_flora_migration(
    time: Res<Time>,
    tilemap: Res<Tilemap>,
    mut query: Query<(&mut TilePos, &MigratoryFlora)>,
) {
    if time.delta().as_secs() < 604800 { return; } // Simplified timer

    for (mut pos, flora) in query.iter_mut() {
        let current_x = pos.x;
        let current_y = pos.y;

        let mut best_dx = 0;
        let mut best_dy = 0;
        let mut best_score = -f32::INFINITY;

        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0), (1, 1), (-1, -1), (1, -1), (-1, 1)];

        for (dx, dy) in directions {
            let nx = current_x as i32 + dx;
            let ny = current_y as i32 + dy;

            if nx >= 0 && nx < tilemap.width as i32 && ny >= 0 && ny < tilemap.height as i32 {
                let unx = nx as u32;
                let uny = ny as u32;

                if tilemap.is_wall(unx, uny) { continue; }

                let score = match flora.preferred_condition {
                    Condition::Water => tilemap.get_water(unx, uny),
                    Condition::LowPollution => -tilemap.get_pollution(unx, uny),
                };

                if score > best_score {
                    best_score = score;
                    best_dx = dx;
                    best_dy = dy;
                }
            }
        }

        if best_score > -f32::INFINITY {
             pos.x = (current_x as i32 + best_dx) as u32;
             pos.y = (current_y as i32 + best_dy) as u32;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Time progression is mocked poorly using `time.delta().as_secs()`. This should use a proper Bevy `Timer` resource synchronized to the game's simulation speed (e.g., ticking once per week).
- **Performance**: Pathfinding is naive greedy search (checking 8 neighbors). While fine for 1 step/week, large forests iterating simultaneously might cause lag spikes. Batch updates across frames.
- **Visual Integration**: Plants shouldn't instantly teleport. Use a Tweening system to animate the movement between tiles, making the "migration" visually evident.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Migratory flora moves one tile towards its preferred environmental condition.
- [ ] Walls successfully block flora movement.
- [ ] Flora correctly moves away from aversive conditions (like Pollution).

## 7. Technical Guidance
- Integrate with existing environmental arrays instead of writing duplicate lookup logic.
- Avoid updating all flora on the exact same tick frame to prevent stutter; stagger their migration checks.
- Handle edge cases: what happens when two migratory plants try to enter the same valid destination tile simultaneously?

## 8. Questions
*Builder: add questions here if spec is unclear.*
