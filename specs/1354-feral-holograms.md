# 1354: Feral Holograms

## 1. Overview
High-tech "Holo-Theaters" provide massive morale boosts but require constant maintenance. If they break down or lose power for an extended period, the holograms "glitch" and wander the colony as hard-light constructs. They cannot do physical damage, but they block pathfinding and cause stress to Pops who encounter them.

## 2. Dependencies
- `GridPosition` component
- `bevy_ecs` setup

## 3. RED Phase: Tests First
```rust
use bevy::prelude::*;

// Mock components to ensure tests run without unverified structures
#[derive(Component)]
pub struct GridPosition { pub x: i32, pub y: i32 }
#[derive(Component)]
pub struct Pop;
#[derive(Component)]
pub struct PopNeeds { pub morale: f32 }

#[test]
fn test_holo_theater_glitch_spawns_feral_hologram() {
    let mut app = App::new();
    app.add_systems(Update, holo_theater_glitch_system);

    app.world_mut().spawn((
        HoloTheater,
        TheaterMaintenance { health: 0.0 },
        GridPosition { x: 5, y: 5 },
    ));

    app.update();

    let mut query = app.world_mut().query::<&FeralHologram>();
    let hologram_count = query.iter(app.world()).count();
    assert!(hologram_count > 0, "A broken holo-theater should spawn feral holograms");
}

#[test]
fn test_feral_hologram_causes_stress_to_nearby_pops() {
    let mut app = App::new();
    app.add_systems(Update, feral_hologram_stress_system);

    let pop = app.world_mut().spawn((
        Pop,
        PopNeeds { morale: 100.0 },
        GridPosition { x: 5, y: 5 }
    )).id();

    app.world_mut().spawn((
        FeralHologram { intensity: 10.0 },
        GridPosition { x: 5, y: 5 }
    ));

    app.update();

    let needs = app.world().get::<PopNeeds>(pop).unwrap();
    assert!(needs.morale < 100.0, "Feral hologram should reduce morale of nearby pops");
}

#[test]
fn test_feral_hologram_blocks_pathfinding() {
    let mut app = App::new();
    app.init_resource::<PassableGridMock>();
    app.add_systems(Update, feral_hologram_pathfinding_block_system);

    app.world_mut().spawn((
        FeralHologram { intensity: 10.0 },
        GridPosition { x: 5, y: 5 }
    ));

    app.update();

    let grid = app.world().resource::<PassableGridMock>();
    assert!(!grid.is_passable(5, 5), "Feral holograms should block pathfinding");
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct HoloTheater;

#[derive(Component)]
pub struct TheaterMaintenance {
    pub health: f32,
}

#[derive(Component)]
pub struct FeralHologram {
    pub intensity: f32,
}

#[derive(Resource, Default)]
pub struct PassableGridMock {
    blocked: Vec<(i32, i32)>
}
impl PassableGridMock {
    pub fn set_passable(&mut self, x: i32, y: i32, passable: bool) {
        if !passable {
            self.blocked.push((x, y));
        } else {
            self.blocked.retain(|&(bx, by)| bx != x || by != y);
        }
    }
    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        !self.blocked.contains(&(x, y))
    }
}

pub fn holo_theater_glitch_system(
    mut commands: Commands,
    query: Query<(&TheaterMaintenance, &GridPosition), With<HoloTheater>>,
) {
    for (maintenance, pos) in query.iter() {
        if maintenance.health <= 0.0 {
            commands.spawn((
                FeralHologram { intensity: 10.0 },
                GridPosition { x: pos.x, y: pos.y },
            ));
        }
    }
}

pub fn feral_hologram_stress_system(
    mut pops: Query<(&GridPosition, &mut PopNeeds), With<Pop>>,
    holograms: Query<(&GridPosition, &FeralHologram)>,
) {
    for (h_pos, hologram) in holograms.iter() {
        for (p_pos, mut needs) in pops.iter_mut() {
            if h_pos.x == p_pos.x && h_pos.y == p_pos.y {
                needs.morale -= hologram.intensity;
            }
        }
    }
}

pub fn feral_hologram_pathfinding_block_system(
    mut terrain: ResMut<PassableGridMock>,
    holograms: Query<&GridPosition, With<FeralHologram>>,
) {
    for pos in holograms.iter() {
        terrain.set_passable(pos.x, pos.y, false);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Instead of directly modifying `PassableGridMock`, integrate with the real pathfinding obstacle system or create a dynamic `Obstacle` component that the pathfinding system respects.
- Holograms shouldn't spawn indefinitely on every tick; add a cooldown or a state flag to `HoloTheater` to prevent infinite entity spawning once broken.
- Provide a way for security forces/militia to "kill" or disperse the `FeralHologram` entities to resolve the blockage.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Feral holograms spawn when HoloTheaters break and reduce Pop morale
- [ ] Feral holograms block pathfinding grid

## 7. Technical Guidance
- Implement this in layer 1.
- For pathfinding blocks, ensure that when the hologram moves or is destroyed, the grid is updated to be passable again.
- Tie the stress impact to the `lore` system if possible (e.g. they see ghosts of past entertainers).

## 8. Questions
*Builder: add questions here if spec is unclear.*
