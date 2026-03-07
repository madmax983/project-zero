# Specification: 447 - Orbital Drop Logistics

## 1. Overview
The **Orbital Drop Logistics** feature introduces a high-risk, high-speed delivery method from Layer 2 ships to Layer 1 colonies. Instead of slow, pinpoint shuttle landings, supplies are fired from orbit. They land fast but scatter, requiring haulers to retrieve them from the wilderness before they degrade or are destroyed by the environment or fauna.

## 2. Dependencies
- `010` Chronicle System
- `025` Hauling Logistics
- `032` Entropy/Spoilage
- `099` Fleet Movement

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use scale::layer1::grid::{GridPos, GridMap};
    use scale::layer1::inventory::Item;
    use scale::layer1::entropy::Spoilage;

    #[test]
    fn test_orbital_drop_spawns_scattered_items() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(OrbitalDropPlugin);
        app.world.insert_resource(GridMap::new(50, 50));

        let target_pos = GridPos::new(25, 25);
        let drop_event = OrbitalDropEvent {
            target: target_pos,
            items: vec![Item::Food(10), Item::Metal(5)],
            scatter_radius: 5,
        };

        // Act
        app.world.send_event(drop_event);
        app.update();

        // Assert: Items are spawned
        let dropped_items = app.world.query::<(&Item, &GridPos)>().iter(&app.world).count();
        assert_eq!(dropped_items, 2, "Both items should spawn");

        // Assert: Items are within scatter radius
        for (item, pos) in app.world.query::<(&Item, &GridPos)>().iter(&app.world) {
            let distance = target_pos.distance(*pos);
            assert!(distance <= 5.0, "Items must land within scatter radius");
        }
    }

    #[test]
    fn test_dropped_items_have_entropy() {
        let mut app = App::new();
        app.add_plugins(OrbitalDropPlugin);
        app.world.insert_resource(GridMap::new(10, 10));

        let drop_event = OrbitalDropEvent {
            target: GridPos::new(5, 5),
            items: vec![Item::Food(5)],
            scatter_radius: 1,
        };

        app.world.send_event(drop_event);
        app.update();

        // Assert: Dropped food has a Spoilage component applied immediately
        let has_spoilage = app.world.query::<&Spoilage>().iter(&app.world).count() > 0;
        assert!(has_spoilage, "Dropped perishable items must start degrading");
    }

    #[test]
    fn test_drop_damages_terrain() {
        let mut app = App::new();
        app.add_plugins(OrbitalDropPlugin);
        let mut grid = GridMap::new(10, 10);
        let target_pos = GridPos::new(5, 5);
        grid.set_terrain(target_pos, TerrainType::Grass);
        app.world.insert_resource(grid);

        let drop_event = OrbitalDropEvent {
            target: target_pos,
            items: vec![Item::Metal(50)], // Heavy drop
            scatter_radius: 0, // Direct hit
        };

        app.world.send_event(drop_event);
        app.update();

        // Assert: Heavy drop turns grass into Crater
        let grid = app.world.resource::<GridMap>();
        assert_eq!(grid.get_terrain(target_pos), TerrainType::Crater, "Heavy drop should crater the terrain");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass

#[derive(Event)]
pub struct OrbitalDropEvent {
    pub target: GridPos,
    pub items: Vec<Item>,
    pub scatter_radius: i32,
}

#[derive(Component)]
pub struct DroppedCrate {
    pub item: Item,
}

pub fn process_orbital_drops(
    mut events: EventReader<OrbitalDropEvent>,
    mut commands: Commands,
    mut grid: ResMut<GridMap>,
) {
    for drop in events.read() {
        for item in &drop.items {
            // Apply simple random scatter
            let dx = (rand::random::<i32>() % (drop.scatter_radius * 2 + 1)) - drop.scatter_radius;
            let dy = (rand::random::<i32>() % (drop.scatter_radius * 2 + 1)) - drop.scatter_radius;
            let final_pos = GridPos::new(drop.target.x + dx, drop.target.y + dy);

            // Spawn crate entity
            let mut entity = commands.spawn((
                DroppedCrate { item: item.clone() },
                final_pos,
                Item::clone(item),
            ));

            // Apply spoilage if perishable
            if matches!(item, Item::Food(_)) {
                entity.insert(Spoilage { rate: 1.0, current: 100.0 });
            }

            // Damage terrain if item is heavy
            if matches!(item, Item::Metal(amt) if *amt > 10) {
                 grid.set_terrain(final_pos, TerrainType::Crater);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Randomness:** Use Bevy's deterministic random number generation (e.g., `EntropyComponent`) rather than standard `rand::random` to ensure tests are repeatable.
- **Pathfinding Checks:** Ensure items do not drop onto impenetrable terrain (like solid rock walls) or off the edge of the map. Clamp the `final_pos` to valid grid bounds.
- **Visuals:** The drop should ideally play an animation or effect before spawning the item to signify the drop happening, rather than items just instantly appearing.
- **Hauling Designation:** Automatically flag dropped crates for hauling so idle pops go get them immediately.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Items land within the specified scatter radius.
- [ ] Terrain damage occurs on direct hits from heavy drops.

## 7. Technical Guidance
- When applying the scatter logic, ensure you don't overwrite existing critical structures without triggering appropriate damage events.
- To make haulers pick up the items, you may need to add a `HaulTarget` or `Designation::Haul` component to the spawned `DroppedCrate`.
- Make sure to bounds-check the scattered `GridPos` against the `GridMap` dimensions.

## 8. Questions
*Builder: add questions here if spec is unclear.*
