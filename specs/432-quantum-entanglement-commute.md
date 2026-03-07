# Spec 432: Quantum Entanglement Commute

## 1. Overview
"Quantum Pads" allow instant teleportation between any two linked points. However, the process causes severe "Disorientation" stress. If used too frequently, Pops develop "Temporal Dissociation," wandering aimlessly and hallucinating that they are in two places at once.

## 2. Dependencies
- `006-building-placement`
- `016-utility-ai-system`
- `127-stress-breakdowns`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_teleportation_applies_disorientation() {
        let mut app = App::new();
        app.add_event::<TeleportEvent>();
        app.add_systems(Update, quantum_commute_system);

        let entity = app.world_mut().spawn((
            Position { value: Vec3::ZERO },
            Stress { value: 0.0 },
        )).id();

        app.world_mut().send_event(TeleportEvent {
            entity,
            target_pos: Vec3::new(100.0, 0.0, 0.0),
        });

        app.update();

        let pos = app.world().get::<Position>(entity).unwrap();
        assert_eq!(pos.value, Vec3::new(100.0, 0.0, 0.0));

        let stress = app.world().get::<Stress>(entity).unwrap();
        assert_eq!(stress.value, 10.0); // +10 stress per jump
    }

    #[test]
    fn test_temporal_dissociation_trigger() {
        let mut app = App::new();
        app.add_systems(Update, check_dissociation_system);

        let entity = app.world_mut().spawn((
            Stress { value: 100.0 },
            TeleportCount { jumps: 5 },
        )).id();

        app.update();

        assert!(app.world().get::<TemporalDissociation>(entity).is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Position {
    pub value: Vec3,
}

#[derive(Component)]
pub struct Stress {
    pub value: f32,
}

#[derive(Component)]
pub struct TeleportCount {
    pub jumps: u32,
}

#[derive(Component)]
pub struct TemporalDissociation;

#[derive(Event)]
pub struct TeleportEvent {
    pub entity: Entity,
    pub target_pos: Vec3,
}

pub fn quantum_commute_system(
    mut events: EventReader<TeleportEvent>,
    mut q_pop: Query<(&mut Position, &mut Stress, &mut TeleportCount)>,
) {
    for event in events.read() {
        if let Ok((mut pos, mut stress, mut count)) = q_pop.get_mut(event.entity) {
            pos.value = event.target_pos;
            stress.value += 10.0;
            count.jumps += 1;
        }
    }
}

pub fn check_dissociation_system(
    mut commands: Commands,
    q_pop: Query<(Entity, &Stress, &TeleportCount), Without<TemporalDissociation>>,
) {
    for (entity, stress, count) in q_pop.iter() {
        if stress.value >= 100.0 && count.jumps >= 5 {
            commands.entity(entity).insert(TemporalDissociation);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Implement a `WanderAimlesslyAI` behavior that activates when a pop has `TemporalDissociation`.
- Have `TeleportCount` decay slowly over time so infrequent jumps are safe.
- Add an Energy cost to the `QuantumPad` building every time it is used.

## 6. Acceptance Criteria
- [ ] Teleporting a pop moves their position instantly.
- [ ] Teleporting applies 10.0 `Stress` and increments `TeleportCount`.
- [ ] Pops with high `Stress` and `TeleportCount` gain `TemporalDissociation`.
- [ ] Tests pass with >= 85% coverage.

## 7. Technical Guidance
- Integrate with Utility AI so pops will autonomously choose `QuantumPad`s if the pathing distance is far, but avoid them if they have high `TeleportCount`.
- The `TemporalDissociation` component should override normal job AI, similar to `StressBreakdowns`.

## 8. Questions
- How long does `TemporalDissociation` last? Can it be cured by medical care?
