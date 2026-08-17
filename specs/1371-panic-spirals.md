# 1371: Panic Spirals

## 1. Overview
**Layer:** 1

**Fantasy:** Fear is contagious.

**Mechanic:** Pops witnessing a "Terrifying Event" (Monster, Fire, Corpse) flee in a random direction. If they collide with other Pops, they pass on the "Panic" state. Panicked pops drop items and cannot work.

**Emergence:** A single rat scares a cook. The cook runs into the hallway, scaring the haulers. The haulers drop their explosive payload, causing a fire, which scares the soldiers. The base collapses from a rat.

**Tension:** Crowd control (wide halls) vs. Efficiency (tight spaces).

## 2. Dependencies
- Base ECS system
- Spatial/Grid positioning system
- Pop state machine / task system
- Event emission for "Terrifying Events"

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            trigger_initial_panic_system,
            spread_panic_on_collision_system,
        ));
        app
    }

    #[derive(Component)]
    struct MockTerrifyingEvent;

    #[test]
    fn test_pop_panics_near_terrifying_event() {
        let mut app = setup_app();

        // Spawn a terrifying event
        app.world_mut().spawn((
            MockTerrifyingEvent,
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn a pop nearby
        let pop = app.world_mut().spawn((
            Pop,
            PopState::Working,
            GridPosition { x: 5, y: 6 }, // Adjacent
        )).id();

        app.update();

        let state = app.world().get::<PopState>(pop).unwrap();

        // Pop should be panicked
        assert_eq!(*state, PopState::Panicking);
    }

    #[test]
    fn test_panic_spreads_on_collision() {
        let mut app = setup_app();

        // Spawn a panicked pop
        let panicked_pop = app.world_mut().spawn((
            Pop,
            PopState::Panicking,
            GridPosition { x: 10, y: 10 },
        )).id();

        // Spawn a normal pop in the same tile
        let normal_pop = app.world_mut().spawn((
            Pop,
            PopState::Working,
            GridPosition { x: 10, y: 10 },
        )).id();

        app.update();

        let state = app.world().get::<PopState>(normal_pop).unwrap();

        // Normal pop should now be panicked
        assert_eq!(*state, PopState::Panicking);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum PopState {
    Idle,
    Working,
    Panicking,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component, PartialEq, Eq, Hash, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

// Marker component for anything that causes panic
#[derive(Component)]
pub struct Terrifying;

pub fn trigger_initial_panic_system(
    mut pops: Query<(&mut PopState, &GridPosition), With<Pop>>,
    terrors: Query<&GridPosition, With<Terrifying>>,
) {
    for (mut state, pop_pos) in pops.iter_mut() {
        if *state == PopState::Panicking { continue; }

        for terror_pos in terrors.iter() {
            // Simple adjacency check
            let dx = (pop_pos.x - terror_pos.x).abs();
            let dy = (pop_pos.y - terror_pos.y).abs();

            if dx <= 1 && dy <= 1 {
                *state = PopState::Panicking;
                // Note: dropping items would go here
            }
        }
    }
}

pub fn spread_panic_on_collision_system(
    mut pops: Query<(Entity, &mut PopState, &GridPosition), With<Pop>>,
) {
    // Collect all positions of currently panicking pops
    let mut panic_positions = Vec::new();
    for (_, state, pos) in pops.iter() {
        if *state == PopState::Panicking {
            panic_positions.push(*pos);
        }
    }

    // Apply panic to any non-panicking pop sharing a position
    for (_, mut state, pos) in pops.iter_mut() {
        if *state != PopState::Panicking && panic_positions.contains(pos) {
            *state = PopState::Panicking;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Grid:** Use a spatial hash map or the central `TerrainGrid` to look up adjacent entities instead of O(N^2) iterations.
- **Panic Duration:** Add a `PanicTimer` component so pops eventually calm down.
- **Courage Trait:** Implement a `Courage` stat or trait that allows some pops (e.g. militia) to resist the panic cascade.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] `Pop` entities adjacent to `Terrifying` entities enter `PopState::Panicking`.
- [ ] `Pop` entities sharing a `GridPosition` with a panicking `Pop` also enter `PopState::Panicking`.

## 7. Technical Guidance
- The "dropping items" mechanic mentioned in the fantasy should be implemented if an inventory system exists (e.g., clearing the `HeldItem` component).
- Movement logic for panicked pops (fleeing randomly) should be handled in a separate movement/AI system that checks for `PopState::Panicking`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
