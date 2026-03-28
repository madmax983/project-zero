# Specification: The Xenofloral Architect

## 1. Overview
**Layer:** 1
**Title:** The Xenofloral Architect
**Description:** Building your base out of living, growing plants instead of dead metal. "Iron-Vine" can be planted instead of constructed, filling designated tiles for free and self-repairing. However, if not pruned by a Farmer pop, it overgrows into empty adjacent tiles, crushing machinery and suffocating pops trapped inside.

## 2. Dependencies
- Tile/Grid system (`Position`, `Building`)
- Resource system (`IronVine` vs `Steel`)
- Jobs/Pops system (`Farmer` and pruning jobs)
- Health/Damage system (crushing pops/buildings)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_iron_vine_grows_over_time() {
        let mut app = App::new();
        app.add_systems(Update, grow_iron_vine_system);

        let start_pos = Position::new(5, 5);
        let vine_entity = app.world_mut().spawn((
            Position::new(start_pos.x, start_pos.y),
            IronVine { growth_progress: 0.0, overgrowth_threshold: 100.0 },
        )).id();

        // Act: Simulate time passage for growth
        app.world_mut().insert_resource(Time::new(std::time::Duration::from_secs(10)));
        app.update();

        // Assert
        let vine = app.world().get::<IronVine>(vine_entity).unwrap();
        assert!(vine.growth_progress > 0.0, "Iron-Vine should grow over time");
    }

    #[test]
    fn test_unpruned_iron_vine_spreads_and_crushes() {
        let mut app = App::new();
        app.add_event::<CrushEvent>();
        app.add_systems(Update, (grow_iron_vine_system, resolve_crush_events));

        let center = Position::new(5, 5);
        let adjacent = Position::new(6, 5);

        // Spawn a fully overgrown vine
        app.world_mut().spawn((
            Position::new(center.x, center.y),
            IronVine { growth_progress: 105.0, overgrowth_threshold: 100.0 },
        ));

        // Spawn a fragile building adjacent to it
        let building_entity = app.world_mut().spawn((
            Position::new(adjacent.x, adjacent.y),
            Building { integrity: 50 },
        )).id();

        // Act: Simulate overgrowth spread
        app.update();

        // Assert
        let events = app.world().resource::<Events<CrushEvent>>();
        let mut reader = events.get_cursor();
        let crushes: Vec<_> = reader.read(events).collect();

        assert_eq!(crushes.len(), 1, "An adjacent building should be crushed");
        assert_eq!(crushes[0].target, building_entity);

        // Verify building took damage
        let building = app.world().get::<Building>(building_entity).unwrap();
        assert!(building.integrity < 50, "Building integrity should be reduced by crushing vine");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self { Self { x, y } }
    pub fn adjacent(&self) -> Vec<Position> {
        vec![
            Position::new(self.x + 1, self.y),
            Position::new(self.x - 1, self.y),
            Position::new(self.x, self.y + 1),
            Position::new(self.x, self.y - 1),
        ]
    }
}

#[derive(Component)]
pub struct IronVine {
    pub growth_progress: f32,
    pub overgrowth_threshold: f32,
}

#[derive(Component)]
pub struct Building {
    pub integrity: i32,
}

#[derive(Event)]
pub struct CrushEvent {
    pub target: Entity,
    pub damage: i32,
}

pub fn grow_iron_vine_system(
    time: Option<Res<Time>>,
    mut query: Query<(&Position, &mut IronVine)>,
    buildings: Query<(Entity, &Position, &Building)>,
    mut crush_events: EventWriter<CrushEvent>,
) {
    let delta = time.map(|t| t.delta_seconds()).unwrap_or(10.0);

    // Build spatial map of buildings
    let mut grid: HashMap<(i32, i32), Entity> = HashMap::new();
    for (entity, pos, _) in buildings.iter() {
        grid.insert((pos.x, pos.y), entity);
    }

    for (pos, mut vine) in query.iter_mut() {
        vine.growth_progress += delta * 2.0; // Growth rate

        if vine.growth_progress > vine.overgrowth_threshold {
            // Overgrowth! Look for adjacent tiles to crush/spread into
            for adj in pos.adjacent() {
                if let Some(&target_entity) = grid.get(&(adj.x, adj.y)) {
                    crush_events.send(CrushEvent {
                        target: target_entity,
                        damage: 25,
                    });
                }
            }
            // Reset growth after spreading
            vine.growth_progress = 0.0;
        }
    }
}

pub fn resolve_crush_events(
    mut events: EventReader<CrushEvent>,
    mut buildings: Query<&mut Building>,
) {
    for event in events.read() {
        if let Ok(mut building) = buildings.get_mut(event.target) {
            building.integrity -= event.damage;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Queries:** `grow_iron_vine_system` shouldn't rebuild the entire `HashMap` every frame. Use a spatial index resource or a grid layer to find adjacent targets efficiently.
- **Jobs:** Pops with the `Farmer` or `Botanist` role should generate "Prune" jobs targeting `IronVine` entities approaching the `overgrowth_threshold`. Completing a prune job should reset `growth_progress` to 0.0 and yield a small amount of bio-mass.
- **Visuals:** Add visual scaling or sprite changes to the `IronVine` entity as `growth_progress` increases.

## 6. Acceptance Criteria (Testable!)
- [ ] `test_iron_vine_grows_over_time` passes.
- [ ] `test_unpruned_iron_vine_spreads_and_crushes` passes.
- [ ] Pruning job system effectively reduces vine growth before it overflows.
- [ ] Pops trapped in overgrown tiles take damage or suffocate.
- [ ] Test coverage ≥85%.

## 7. Technical Guidance
- Implement `IronVine` logic in `src/layer1/nature/flora.rs`.
- `CrushEvent` should be part of the global simulation loop and handle both buildings and pops.
- Consider utilizing the `Job` system to schedule `PruneJob` dynamically based on vine growth.
- **Warning:** Be cautious with recursive spread; ensure vines only spread once per threshold tick to prevent exponential growth cascades in a single frame.

## 8. Questions
*Builder: add questions here if spec is unclear.*
