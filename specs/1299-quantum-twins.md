# Specification: Quantum Twins

## 1. Overview
A rare trait where two Pops become "Entangled". They share XP gains, Mood trends, and Damage taken. If one twin dies, the other immediately suffers a catastrophic "Severance" state, leaving them Catatonic. This adds tension to managing highly specialized or at-risk pops, encouraging either deep separation of risk or tight localized safety.

## 2. Dependencies
- Pop logic (Layer 1)
- Pop Health, XP, and Mood systems
- Death event propagation

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Mock components for test phase
    #[derive(Component)]
    pub struct Entangled {
        pub twin_entity: Entity,
    }

    #[derive(Component)]
    pub struct Health {
        pub current: f32,
    }

    #[derive(Component)]
    pub struct Mood {
        pub value: f32,
    }

    #[derive(Component)]
    pub struct Catatonic;

    #[derive(Event)]
    pub struct DamageEvent {
        pub target: Entity,
        pub amount: f32,
    }

    #[derive(Event)]
    pub struct DeathEvent {
        pub entity: Entity,
    }

    fn propagate_damage_to_twin(
        mut events: EventReader<DamageEvent>,
        q_entangled: Query<&Entangled>,
        mut q_health: Query<&mut Health>,
    ) {
        // Implementation will go here
    }

    fn process_twin_death_severance(
        mut events: EventReader<DeathEvent>,
        mut commands: Commands,
        q_entangled: Query<(Entity, &Entangled)>,
    ) {
        // Implementation will go here
    }

    #[test]
    fn test_damage_propagates_to_twin() {
        let mut app = App::new();
        app.add_event::<DamageEvent>();
        app.add_systems(Update, propagate_damage_to_twin);

        let twin1 = app.world_mut().spawn(Health { current: 100.0 }).id();
        let twin2 = app.world_mut().spawn((
            Health { current: 100.0 },
            Entangled { twin_entity: twin1 }
        )).id();
        app.world_mut().entity_mut(twin1).insert(Entangled { twin_entity: twin2 });

        app.world_mut().resource_mut::<Events<DamageEvent>>().send(DamageEvent {
            target: twin1,
            amount: 20.0,
        });

        app.update();

        // Assuming a standard damage system reduces twin1, we just test twin2 here
        let health2 = app.world().get::<Health>(twin2).unwrap();
        assert_eq!(health2.current, 80.0, "Twin 2 should receive the same damage as Twin 1");
    }

    #[test]
    fn test_death_causes_severance_in_twin() {
        let mut app = App::new();
        app.add_event::<DeathEvent>();
        app.add_systems(Update, process_twin_death_severance);

        let twin1 = app.world_mut().spawn_empty().id();
        let twin2 = app.world_mut().spawn(Entangled { twin_entity: twin1 }).id();
        app.world_mut().entity_mut(twin1).insert(Entangled { twin_entity: twin2 });

        app.world_mut().resource_mut::<Events<DeathEvent>>().send(DeathEvent { entity: twin1 });

        app.update();

        assert!(app.world().get::<Catatonic>(twin2).is_some(), "Twin 2 should become Catatonic upon Twin 1's death");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The minimal code required to pass the tests

fn propagate_damage_to_twin(
    mut events: EventReader<DamageEvent>,
    q_entangled: Query<&Entangled>,
    mut q_health: Query<&mut Health>,
) {
    for event in events.read() {
        // Apply damage to primary target
        if let Ok(mut health) = q_health.get_mut(event.target) {
            health.current -= event.amount;
        }

        // Propagate to twin
        if let Ok(entangled) = q_entangled.get(event.target) {
            if let Ok(mut twin_health) = q_health.get_mut(entangled.twin_entity) {
                twin_health.current -= event.amount;
            }
        }
    }
}

fn process_twin_death_severance(
    mut events: EventReader<DeathEvent>,
    mut commands: Commands,
    q_entangled: Query<(Entity, &Entangled)>,
) {
    for event in events.read() {
        for (entity, entangled) in q_entangled.iter() {
            if entangled.twin_entity == event.entity {
                commands.entity(entity).insert(Catatonic);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Avoid double-dipping on damage if an AoE attack hits both twins simultaneously. Track recently applied damage via event deduplication or a frame-local damage buffer.
- Expand `propagate_damage_to_twin` to mirror XP and Mood events in a similar pipeline.
- Implement the effects of `Catatonic` on Pop Utility AI (they should drop tasks, stop moving, and require medical/psychiatric rescue).
- Integrate the initial pairing logic into the Pop generation/arrival system to occasionally link twins.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Damage applied to one twin propagates to the other.
- [ ] The death of one twin inserts the `Catatonic` component onto the survivor.

## 7. Technical Guidance
- `Entangled` should reference another Entity, making it a directed graph link (always kept reciprocal).
- If either twin's entity ID becomes invalid, ensure proper cleanup so system panics don't occur when looking up `twin_entity`.
- Consider placing the damage propagation logic early in the `Schedule` to ensure it resolves before health boundary checks (like death threshold checks).

## 8. Questions
*Builder: add questions here if spec is unclear.*
