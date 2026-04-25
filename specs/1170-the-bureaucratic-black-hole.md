# 1170: The Bureaucratic Black Hole

## 1. Overview
A government so complex and dense that information, resources, and even people simply vanish into it, never to be seen again. "Administration Hubs" are required to manage large populations. However, if a colony builds too many administrative buildings relative to its productive buildings, it creates a "Bureaucratic Black Hole". This anomaly has a physical radius on the Layer 1 map. Resources hauled into this radius have a small percentage chance of being "Lost in Paperwork" (deleted). Pops who enter the radius to perform jobs have a very rare chance to be permanently "Reassigned" (deleted from the game without generating a corpse or memory).

## 2. Dependencies
- Layer 1 `GridPosition` and Building placement system.
- Entity/Resource hauling system (e.g., `Inventory`).
- Pop job evaluation / movement system.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_bureaucratic_black_hole_formation() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, check_bureaucratic_density_system);

    // Spawn 1 productive building
    app.world_mut().spawn((Building, Productive, GridPosition { x: 0, y: 0 }));
    // Spawn multiple admin buildings close to each other
    for i in 1..=5 {
        app.world_mut().spawn((Building, Administrative, GridPosition { x: i, y: 0 }));
    }

    // Act
    app.update();

    // Assert: BureaucraticBlackHole component should be added to the cluster
    let mut query = app.world_mut().query::<&BureaucraticBlackHole>();
    assert_eq!(query.iter(&app.world()).count(), 1, "A Bureaucratic Black Hole should have formed");
}

#[test]
fn test_resource_lost_in_paperwork() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, bureaucratic_resource_loss_system);

    let black_hole_pos = GridPosition { x: 5, y: 5 };
    app.world_mut().spawn((BureaucraticBlackHole { radius: 2.0 }, black_hole_pos));

    // Resource carried into the black hole (using 100% loss chance for testing)
    let resource_entity = app.world_mut().spawn((
        ResourceItem,
        GridPosition { x: 5, y: 6 },
        LostInPaperworkChance(1.0)
    )).id();

    // Act
    app.update();

    // Assert: Resource is deleted
    assert!(app.world().get::<ResourceItem>(resource_entity).is_none(), "Resource should be lost in paperwork");
}

#[test]
fn test_pop_reassignment_deletion() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, bureaucratic_pop_reassignment_system);

    let black_hole_pos = GridPosition { x: 5, y: 5 };
    app.world_mut().spawn((BureaucraticBlackHole { radius: 2.0 }, black_hole_pos));

    // Pop enters the black hole (using 100% reassignment chance for testing)
    let pop_entity = app.world_mut().spawn((
        Pop,
        GridPosition { x: 5, y: 4 },
        ReassignmentChance(1.0)
    )).id();

    // Act
    app.update();

    // Assert: Pop is deleted
    assert!(app.world().get::<Pop>(pop_entity).is_none(), "Pop should be permanently reassigned (deleted)");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct Productive;

#[derive(Component)]
pub struct Administrative;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn distance(&self, other: &GridPosition) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Component)]
pub struct BureaucraticBlackHole {
    pub radius: f32,
}

#[derive(Component)]
pub struct ResourceItem;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct LostInPaperworkChance(pub f32);

#[derive(Component)]
pub struct ReassignmentChance(pub f32);

pub fn check_bureaucratic_density_system(
    mut commands: Commands,
    admin_query: Query<(Entity, &GridPosition), With<Administrative>>,
    productive_query: Query<&GridPosition, With<Productive>>,
    black_hole_query: Query<&BureaucraticBlackHole>,
) {
    if !black_hole_query.is_empty() {
        return; // Only one for now to satisfy minimal test
    }

    let admin_count = admin_query.iter().count();
    let prod_count = productive_query.iter().count();

    if admin_count > prod_count * 2 && admin_count >= 5 {
        if let Some((entity, _)) = admin_query.iter().next() {
            commands.entity(entity).insert(BureaucraticBlackHole { radius: 2.0 });
        }
    }
}

pub fn bureaucratic_resource_loss_system(
    mut commands: Commands,
    black_holes: Query<(&BureaucraticBlackHole, &GridPosition)>,
    resources: Query<(Entity, &GridPosition, Option<&LostInPaperworkChance>), With<ResourceItem>>,
) {
    for (bh, bh_pos) in black_holes.iter() {
        for (entity, res_pos, chance) in resources.iter() {
            if bh_pos.distance(res_pos) <= bh.radius {
                let loss_chance = chance.map(|c| c.0).unwrap_or(0.01);
                // For test determinism if chance is 1.0 we delete
                if loss_chance >= 1.0 {
                    commands.entity(entity).despawn();
                } else if rand::random::<f32>() < loss_chance {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

pub fn bureaucratic_pop_reassignment_system(
    mut commands: Commands,
    black_holes: Query<(&BureaucraticBlackHole, &GridPosition)>,
    pops: Query<(Entity, &GridPosition, Option<&ReassignmentChance>), With<Pop>>,
) {
    for (bh, bh_pos) in black_holes.iter() {
        for (entity, pop_pos, chance) in pops.iter() {
            if bh_pos.distance(pop_pos) <= bh.radius {
                let reassignment_chance = chance.map(|c| c.0).unwrap_or(0.001);
                if reassignment_chance >= 1.0 {
                    commands.entity(entity).despawn();
                } else if rand::random::<f32>() < reassignment_chance {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Cluster Detection**: The MVP simply picks the first administrative building. A better approach would be to find the centroid of the cluster of administrative buildings to place the black hole.
- **Event Broadcasting**: Currently, pops and resources are silently deleted. It would be better to broadcast a `ChronicleEvent` for narrative generation (e.g. "Dr. [Name] was reassigned to a special project.").
- **Deterministic RNG**: The use of `rand::random` is non-deterministic. Inject a seeded `Rng` resource into the systems.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Pops and resources within the Bureaucratic Black Hole radius are deleted probabilistically.

## 7. Technical Guidance
- Integrate with existing `Pop` and `Resource` systems. Ensure you don't leak memory or leave dangling references (e.g., if a pop was holding an item or had an active job).
- The `BureaucraticBlackHole` component should probably be rendering a special UI aura or warning to players so they understand why things are vanishing.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
