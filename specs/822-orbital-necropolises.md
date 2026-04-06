# 822: Orbital Necropolises

## 1. Overview
Instead of burying the dead on Layer 1 (taking up valuable land), colonies can launch them into "Orbital Necropolis" stations on Layer 2. These stations provide massive cultural and ideological control bonuses to the planet below. However, they are highly fragile. If an enemy destroys one, the planet below suffers catastrophic morale failure and a permanent "Desecrated Skies" penalty.

## 2. Dependencies
- `src/layer1/pop.rs` for Pops and death events.
- `src/layer2/stations.rs` or orbital infrastructure implementations.
- `src/layer1/colony.rs` for colony-wide morale modifiers.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_orbital_necropolis_provides_bonus() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, apply_necropolis_bonus);

        let colony_entity = app.world_mut().spawn(Colony {
            morale: 50.0,
            ..default()
        }).id();

        let necropolis = app.world_mut().spawn((
            OrbitalNecropolis { colony_id: colony_entity },
            Health { current: 100, max: 100 },
        )).id();

        // Act
        app.update();

        // Assert
        // The colony should have a positive morale modifier.
        let colony = app.world().get::<Colony>(colony_entity).unwrap();
        assert!(colony.morale > 50.0);
    }

    #[test]
    fn test_destroyed_necropolis_applies_desecrated_skies() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, handle_necropolis_destruction);
        app.add_event::<EntityDestroyedEvent>();

        let colony_entity = app.world_mut().spawn(Colony {
            morale: 80.0,
            ..default()
        }).id();

        let necropolis = app.world_mut().spawn((
            OrbitalNecropolis { colony_id: colony_entity },
        )).id();

        // Act
        app.world_mut().send_event(EntityDestroyedEvent { entity: necropolis });
        app.update();

        // Assert
        let desecrated = app.world().get::<DesecratedSkies>(colony_entity);
        assert!(desecrated.is_some());
        let colony = app.world().get::<Colony>(colony_entity).unwrap();
        // The penalty should heavily reduce morale.
        assert!(colony.morale < 80.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct OrbitalNecropolis {
    pub colony_id: Entity,
}

#[derive(Component)]
pub struct DesecratedSkies;

pub struct EntityDestroyedEvent {
    pub entity: Entity,
}

pub fn apply_necropolis_bonus(
    query: Query<&OrbitalNecropolis>,
    mut colonies: Query<&mut Colony>,
) {
    for necropolis in query.iter() {
        if let Ok(mut colony) = colonies.get_mut(necropolis.colony_id) {
            // Apply a static bonus each tick, or modify a base stat.
            // Simplified for Green phase:
            colony.morale = (colony.morale + 1.0).min(100.0);
        }
    }
}

pub fn handle_necropolis_destruction(
    mut events: EventReader<EntityDestroyedEvent>,
    necropolis_query: Query<&OrbitalNecropolis>,
    mut commands: Commands,
    mut colonies: Query<&mut Colony>,
) {
    for event in events.read() {
        if let Ok(necropolis) = necropolis_query.get(event.entity) {
            commands.entity(necropolis.colony_id).insert(DesecratedSkies);
            if let Ok(mut colony) = colonies.get_mut(necropolis.colony_id) {
                // Catastrophic penalty
                colony.morale = (colony.morale - 50.0).max(0.0);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Refactor morale application: Instead of modifying `colony.morale` directly every tick, use the existing modifier system (if one exists) so `DesecratedSkies` applies a persistent `-X Morale` component and `OrbitalNecropolis` provides a persistent `+X Morale` component.
- Ensure `DesecratedSkies` hooks into the Chronicle system for storytelling.
- Add an integration test to ensure Pops dying on Layer 1 are properly sent to the Necropolis on Layer 2 (e.g., increasing its capacity or bonus).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Necropolises properly link to their source colony and grant bonuses.
- [ ] Destruction triggers the `DesecratedSkies` catastrophic penalty.

## 7. Technical Guidance
- Be careful with `EntityDestroyedEvent` matching against components; if the entity is despawned before the event is read, `query.get` might fail. You may need to read the `colony_id` *before* the entity is despawned, perhaps storing it in the event payload itself.
- Consider making `OrbitalNecropolis` a station type or building type in Layer 2.

## 8. Questions
*Builder: add questions here if spec is unclear.*
