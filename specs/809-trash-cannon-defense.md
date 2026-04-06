# Specification: Trash-Cannon Defense (Feature 809)

## 1. Overview
The **Trash-Cannon Defense** mechanic allows colonies to repurpose solid waste as makeshift ammunition. Turrets can accept "Waste", "Slag", or "Stone" chunks to fire. While these are cheap and provide short-range defense, their impact creates "Messy" tiles that require post-raid cleaning operations. This creates a resource-management tension between recycling waste for materials and using it as a disposable defense mechanism.

## 2. Dependencies
- `Layer 1 Defense Systems`
- `Resource Inventory Systems` (Waste, Slag, Stone)
- `Tile Properties System` (Messy tiles)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_trash_cannon_fires_using_waste_inventory() {
        let mut app = App::new();
        app.add_systems(Update, fire_trash_cannon_system);

        // Arrange
        let entity = app.world_mut().spawn((
            TrashCannon { range: 5.0, cooldown: 0.0 },
            Inventory { waste_count: 10 }
        )).id();

        let target = app.world_mut().spawn(EnemyTarget).id();

        // Act
        app.update();

        // Assert
        let inventory = app.world().get::<Inventory>(entity).unwrap();
        assert_eq!(inventory.waste_count, 9, "Firing should consume one unit of waste");
    }

    #[test]
    fn test_trash_cannon_impact_creates_messy_tile() {
        let mut app = App::new();
        app.add_systems(Update, resolve_trash_impact_system);

        // Arrange
        let map_tile = app.world_mut().spawn((
            Tile { x: 0, y: 0 },
            CleanStatus::Clean
        )).id();

        app.world_mut().spawn(TrashImpactEvent { target_tile: map_tile });

        // Act
        app.update();

        // Assert
        let tile_status = app.world().get::<CleanStatus>(map_tile).unwrap();
        assert_eq!(*tile_status, CleanStatus::Messy, "Impact should leave the tile in a Messy state");
    }

    #[test]
    fn test_trash_cannon_fails_to_fire_without_ammo() {
        let mut app = App::new();
        app.add_systems(Update, fire_trash_cannon_system);

        // Arrange
        let entity = app.world_mut().spawn((
            TrashCannon { range: 5.0, cooldown: 0.0 },
            Inventory { waste_count: 0 }
        )).id();

        let target = app.world_mut().spawn(EnemyTarget).id();

        // Act
        app.update();

        // Assert
        let inventory = app.world().get::<Inventory>(entity).unwrap();
        assert_eq!(inventory.waste_count, 0, "Inventory should remain unchanged if cannon cannot fire");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct TrashCannon {
    pub range: f32,
    pub cooldown: f32,
}

#[derive(Component)]
pub struct Inventory {
    pub waste_count: u32,
}

#[derive(Component)]
pub struct EnemyTarget;

#[derive(Component)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, PartialEq, Debug)]
pub enum CleanStatus {
    Clean,
    Messy,
}

#[derive(Event)]
pub struct TrashImpactEvent {
    pub target_tile: Entity,
}

pub fn fire_trash_cannon_system(
    mut commands: Commands,
    mut query: Query<(&mut Inventory, &mut TrashCannon)>,
    target_query: Query<Entity, With<EnemyTarget>>
) {
    if target_query.is_empty() { return; }

    for (mut inventory, mut cannon) in query.iter_mut() {
        if inventory.waste_count > 0 && cannon.cooldown <= 0.0 {
            inventory.waste_count -= 1;
            cannon.cooldown = 1.0;
            // Simplified: Fire logic triggers impact event elsewhere
        }
    }
}

pub fn resolve_trash_impact_system(
    mut events: EventReader<TrashImpactEvent>,
    mut tile_query: Query<&mut CleanStatus>
) {
    for event in events.read() {
        if let Ok(mut status) = tile_query.get_mut(event.target_tile) {
            *status = CleanStatus::Messy;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Refactor the inventory system to use a generalized `ResourceBuffer` map to abstract checking for `Waste`, `Slag`, or `Stone` specifically.
- Instead of setting a simple `CleanStatus::Messy`, add a `Pollution` component to the tile that accrues incrementally with each shot.
- Add spatial querying (e.g., using a grid or spatial hash) to ensure the `EnemyTarget` is actually within the `TrashCannon::range` before firing.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] A cannon refuses to fire if out of accepted waste ammo.
- [ ] Cannon shots convert clean tiles into messy tiles.

## 7. Technical Guidance
- Firing the cannon should queue a `TrashImpactEvent` rather than immediately altering tile state, ensuring that the visual projectile system has time to render travel distance before impact resolution.
- Make sure to initialize the `TrashImpactEvent` event queue in the world setup phase to prevent test panics.

## 8. Questions
*Builder: add questions here if spec is unclear.*
