# Feature Specification: Hull Geometry (845)

## 1. Overview
**Layer**: 2 (System Layer)
**Fantasy**: Fitting a square peg in a round hull. Ship design is a puzzle.
**Mechanic**: Ship modules have adjacency effects. Heat-generating engines next to Ammo Magazines = Explosion risk. Shield generators need direct Power adjacency.
**Emergence**: You build a "Glass Cannon" ship. One hit to the engine chain-reacts through the ammo and splits the ship in half.
**Tension**: Compact design (Small target/cheap) vs. Safe design (Spaced out/expensive).

## 2. Dependencies
- `Ship Building System` (for modular ships)
- `Damage System` (for cascading explosions)
- `Grid Adjacency System` (for spatial module logic)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Dummy structs for compilation
    #[derive(Component, Clone, Copy, PartialEq)]
    enum ModuleType {
        Engine,
        AmmoMagazine,
        ShieldGenerator,
        PowerCore,
        Empty,
    }

    #[derive(Component)]
    struct ShipGrid {
        modules: Vec<Vec<ModuleType>>,
    }

    #[derive(Component)]
    struct HeatRisk {
        level: f32,
    }

    #[derive(Component)]
    struct ExplosionRisk {
        probability: f32,
    }

    #[derive(Component)]
    struct PowerEfficiency {
        modifier: f32,
    }

    #[test]
    fn test_engine_adjacent_to_ammo_increases_explosion_risk() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, calculate_adjacency_risks);

        let grid = vec![
            vec![ModuleType::Engine, ModuleType::AmmoMagazine],
            vec![ModuleType::Empty, ModuleType::Empty]
        ];

        let entity = app.world_mut().spawn((
            ShipGrid { modules: grid },
            ExplosionRisk { probability: 0.0 }
        )).id();

        // Act
        app.update();

        // Assert
        let risk = app.world().get::<ExplosionRisk>(entity).unwrap();
        assert!(risk.probability > 0.5); // High risk!
    }

    #[test]
    fn test_shield_adjacent_to_power_increases_efficiency() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, calculate_adjacency_efficiency);

        let grid = vec![
            vec![ModuleType::ShieldGenerator, ModuleType::PowerCore],
            vec![ModuleType::Empty, ModuleType::Empty]
        ];

        let entity = app.world_mut().spawn((
            ShipGrid { modules: grid },
            PowerEfficiency { modifier: 1.0 }
        )).id();

        // Act
        app.update();

        // Assert
        let efficiency = app.world().get::<PowerEfficiency>(entity).unwrap();
        assert!(efficiency.modifier > 1.0); // Boosted efficiency
    }

    #[test]
    fn test_cascading_damage_triggers() {
        // Arrange
        let mut app = App::new();
        app.add_event::<DamageEvent>();
        app.add_event::<ExplosionEvent>();
        app.add_systems(Update, process_module_damage);

        let grid = vec![
            vec![ModuleType::AmmoMagazine, ModuleType::AmmoMagazine]
        ];

        let entity = app.world_mut().spawn((
            ShipGrid { modules: grid },
        )).id();

        app.world_mut().send_event(DamageEvent { target: entity, x: 0, y: 0, amount: 100.0 });

        // Act
        app.update();

        // Assert
        let explosion_events = app.world().resource::<Events<ExplosionEvent>>();
        let mut reader = explosion_events.get_cursor();
        let events: Vec<_> = reader.read(explosion_events).collect();
        assert!(!events.is_empty());
        // Should cascade to (0,1)
    }
}

// Dummy events for compilation
struct DamageEvent {
    target: Entity,
    x: usize,
    y: usize,
    amount: f32,
}

struct ExplosionEvent {
    target: Entity,
    x: usize,
    y: usize,
    radius: f32,
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq)]
pub enum ModuleType {
    Engine,
    AmmoMagazine,
    ShieldGenerator,
    PowerCore,
    Empty,
}

#[derive(Component)]
pub struct ShipGrid {
    pub modules: Vec<Vec<ModuleType>>,
}

#[derive(Component)]
pub struct ExplosionRisk {
    pub probability: f32,
}

#[derive(Component)]
pub struct PowerEfficiency {
    pub modifier: f32,
}

pub fn calculate_adjacency_risks(
    mut query: Query<(&ShipGrid, &mut ExplosionRisk)>
) {
    for (grid, mut risk) in query.iter_mut() {
        let mut current_risk = 0.0;
        let height = grid.modules.len();
        let width = if height > 0 { grid.modules[0].len() } else { 0 };

        for y in 0..height {
            for x in 0..width {
                if grid.modules[y][x] == ModuleType::Engine {
                    // Check adjacencies (simple right/down for minimal implementation)
                    if x + 1 < width && grid.modules[y][x + 1] == ModuleType::AmmoMagazine {
                        current_risk += 0.6;
                    }
                    if y + 1 < height && grid.modules[y + 1][x] == ModuleType::AmmoMagazine {
                        current_risk += 0.6;
                    }
                }
            }
        }
        risk.probability = current_risk;
    }
}

pub fn calculate_adjacency_efficiency(
    mut query: Query<(&ShipGrid, &mut PowerEfficiency)>
) {
    for (grid, mut efficiency) in query.iter_mut() {
        let mut modifier = 1.0;
        let height = grid.modules.len();
        let width = if height > 0 { grid.modules[0].len() } else { 0 };

        for y in 0..height {
            for x in 0..width {
                if grid.modules[y][x] == ModuleType::ShieldGenerator {
                    if x + 1 < width && grid.modules[y][x + 1] == ModuleType::PowerCore {
                        modifier += 0.5;
                    }
                }
            }
        }
        efficiency.modifier = modifier;
    }
}

#[derive(Event)]
pub struct DamageEvent {
    pub target: Entity,
    pub x: usize,
    pub y: usize,
    pub amount: f32,
}

#[derive(Event)]
pub struct ExplosionEvent {
    pub target: Entity,
    pub x: usize,
    pub y: usize,
    pub radius: f32,
}

pub fn process_module_damage(
    mut damage_events: EventReader<DamageEvent>,
    mut explosion_events: EventWriter<ExplosionEvent>,
    query: Query<&ShipGrid>
) {
    for event in damage_events.read() {
        if let Ok(grid) = query.get(event.target) {
            if event.y < grid.modules.len() && event.x < grid.modules[event.y].len() {
                if grid.modules[event.y][event.x] == ModuleType::AmmoMagazine && event.amount >= 100.0 {
                    explosion_events.send(ExplosionEvent {
                        target: event.target,
                        x: event.x,
                        y: event.y,
                        radius: 2.0,
                    });
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smell**: Hardcoded `Vec<Vec<ModuleType>>` iteration.
- **Improvement**: Create an iterator for grid adjacencies (up, down, left, right) to reuse the logic.
- **API Change**: `ExplosionRisk` should probably be calculated per-module, not globally for the whole ship, so we know *where* the explosion is likely to happen.
- **Optimization**: Only recalculate adjacencies when the ship's module configuration changes, rather than every frame.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Engine + Ammo adjacency drastically increases explosion risk
- [ ] Shield + Power adjacency increases shield efficiency
- [ ] Destroying ammo modules cascades damage to adjacent modules

## 7. Technical Guidance
- Integrate with the existing `Ship` structure in `src/layer2/ship.rs`.
- Look at `src/layer1/nature/temperature.rs` grid systems for inspiration on grid iteration.
- Consider utilizing a sparse spatial hashmap if ships get large.

## 8. Questions
*Builder: add questions here if spec is unclear.*
