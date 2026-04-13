# 988 Teleporter Psychosis

## 1. Overview
The convenience of instant travel comes at the cost of the soul.

**Mechanic:** "Teleporter Pads" allow instant travel between distant points. However, usage adds a hidden "Dissociation" counter. High counters lead to "Phantom" traits (ignoring hunger, walking through walls) or eventual permadeath (fading away).
**Emergence:** You build a teleporter network to maximize efficiency. Your best workers slowly turn into ghosts who refuse to acknowledge the physical world, eventually vanishing mid-shift.
**Tension:** Speed (Teleport) vs. Safety (Walk).

## 2. Dependencies
- Pathfinding / Navigation nodes
- Pop `Trait` and `Needs` systems
- Buildings and interaction points

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use shared::pop::{Pop, Needs, Traits};

    #[test]
    fn test_teleporter_use_adds_dissociation() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, handle_teleport_system);

        let pop_id = app.world_mut().spawn((
            Pop,
            Dissociation { level: 0.0 },
        )).id();

        // Act: Pop uses teleporter
        app.world_mut().send_event(TeleportEvent { entity: pop_id });
        app.update();

        // Assert
        let dissoc = app.world().get::<Dissociation>(pop_id).unwrap();
        assert!(dissoc.level > 0.0);
    }

    #[test]
    fn test_high_dissociation_grants_phantom_trait() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_psychosis_system);

        let pop_id = app.world_mut().spawn((
            Pop,
            Traits::default(),
            Dissociation { level: 90.0 }, // Above threshold
        )).id();

        // Act
        app.update();

        // Assert
        let traits = app.world().get::<Traits>(pop_id).unwrap();
        assert!(traits.contains(&Trait::Phantom));
    }

    #[test]
    fn test_phantom_trait_ignores_hunger() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, hunger_decay_system);

        let normal_pop = app.world_mut().spawn((
            Pop,
            Needs { hunger: 100.0 },
            Traits::default(),
        )).id();

        let mut ghost_traits = Traits::default();
        ghost_traits.insert(Trait::Phantom);

        let ghost_pop = app.world_mut().spawn((
            Pop,
            Needs { hunger: 100.0 },
            ghost_traits,
        )).id();

        // Act
        app.update();

        // Assert: Normal pop gets hungry, Ghost pop doesn't
        assert!(app.world().get::<Needs>(normal_pop).unwrap().hunger < 100.0);
        assert_eq!(app.world().get::<Needs>(ghost_pop).unwrap().hunger, 100.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use shared::pop::{Pop, Needs, Traits};

#[derive(Component)]
pub struct Dissociation {
    pub level: f32,
}

#[derive(Event, Debug)]
pub struct TeleportEvent {
    pub entity: Entity,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Trait {
    Phantom,
}

pub fn handle_teleport_system(
    mut events: EventReader<TeleportEvent>,
    mut query: Query<&mut Dissociation>,
) {
    for event in events.read() {
        if let Ok(mut dissoc) = query.get_mut(event.entity) {
            dissoc.level += 5.0; // Magic number for minimal passing test
        }
    }
}

pub fn process_psychosis_system(
    mut query: Query<(&Dissociation, &mut Traits)>,
) {
    for (dissoc, mut traits) in query.iter_mut() {
        if dissoc.level >= 80.0 {
            traits.insert(Trait::Phantom);
        }
    }
}

pub fn hunger_decay_system(
    mut query: Query<(&mut Needs, &Traits), With<Pop>>,
) {
    for (mut needs, traits) in query.iter_mut() {
        if !traits.contains(&Trait::Phantom) {
            needs.hunger -= 1.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create constants for thresholds (`DISSOCIATION_PHANTOM_THRESHOLD = 80.0`, `TELEPORT_DISSOCIATION_COST = 5.0`).
- Hook up the `TeleportEvent` to the actual pathfinding completion node or movement system when entering a teleporter building.
- Add an ultimate permadeath state if `Dissociation` reaches 100.0, utilizing `DespawnRecursive`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops correctly gain `Dissociation` and `Phantom` trait.

## 7. Technical Guidance
- `Dissociation` should be an ECS Component added dynamically to `Pop` entities.
- Integrating with the navigation mesh requires marking teleporters as linked nodes with `0.0` path cost, but emitting the `TeleportEvent` upon traversal.

## 8. Questions
*Builder: add questions here if spec is unclear.*
