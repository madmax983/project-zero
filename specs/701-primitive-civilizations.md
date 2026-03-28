# 701 Primitive Civilizations

## 1. Overview
When exploring Layer 2 (Planetary/System map), players may encounter planets inhabited by primitive Pops (Stone or Iron Age technology levels). The player is presented with a moral and strategic dilemma: construct an Observation Post to passively gain Science, or launch an Invasion Force to aggressively seize the land and enslave the inhabitants. This bridges Layer 2 exploration with Layer 1 consequences.

## 2. Dependencies
- Layer 2 `Planet` and `Fleet` mechanics.
- `ColonyResources` for Science generation.
- Faction/Pop systems to integrate primitive pops if invaded.
- Chronicle system for recording first contacts and invasions.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_primitive_planet_generates_science_with_observation_post() {
        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(ColonyResources::default());
        let planet = app.world_mut().spawn((
            Planet { name: "Tau Ceti Prime".to_string() },
            PrimitiveCivilization { tech_level: TechLevel::BronzeAge },
            ObservationPost { active: true }
        )).id();

        // Act
        app.add_systems(Update, process_observation_posts);
        app.update();

        // Assert
        let resources = app.world().get_resource::<ColonyResources>().unwrap();
        assert!(resources.science > 0.0, "Observation post should generate science points");
    }

    #[test]
    fn test_invading_primitive_civilization_spawns_slave_pops() {
        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(ColonyResources::default());
        let planet = app.world_mut().spawn((
            Planet { name: "Tau Ceti Prime".to_string() },
            PrimitiveCivilization { tech_level: TechLevel::IronAge },
        )).id();

        // Act
        // Simulate an invasion event
        let mut events = app.world_mut().resource_mut::<Events<InvasionEvent>>();
        events.send(InvasionEvent { target: planet, aggressor: FactionId::Player });

        app.add_systems(Update, resolve_primitive_invasions);
        app.update();

        // Assert
        let planet_state = app.world().get::<PrimitiveCivilization>(planet);
        assert!(planet_state.is_none(), "Civilization should be removed after successful invasion");

        let slave_count = app.world().query_filtered::<&Pop, With<Slave>>().iter(app.world()).count();
        assert!(slave_count > 0, "Invasion should result in new slave pops");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct PrimitiveCivilization {
    pub tech_level: TechLevel,
}

#[derive(Component)]
pub struct ObservationPost {
    pub active: bool,
}

#[derive(Component)]
pub struct Slave;

pub enum TechLevel {
    StoneAge,
    BronzeAge,
    IronAge,
}

pub struct InvasionEvent {
    pub target: Entity,
    pub aggressor: FactionId,
}

pub fn process_observation_posts(
    posts: Query<&ObservationPost, With<PrimitiveCivilization>>,
    mut resources: ResMut<ColonyResources>,
) {
    for post in posts.iter() {
        if post.active {
            resources.science += 5.0; // Minimal science gain
        }
    }
}

pub fn resolve_primitive_invasions(
    mut commands: Commands,
    mut events: EventReader<InvasionEvent>,
    civs: Query<&PrimitiveCivilization>,
) {
    for event in events.read() {
        if civs.get(event.target).is_ok() {
            // Remove the civilization
            commands.entity(event.target).remove::<PrimitiveCivilization>();

            // Spawn some slaves
            for _ in 0..5 {
                commands.spawn((Pop { ..default() }, Slave));
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**: The `resolve_primitive_invasions` should ideally trigger a more complex combat resolution before declaring an automatic victory, depending on the invading fleet's strength versus the primitive tech level.
- **Code Smells**: Magic numbers for Science output and Slave counts. Introduce `PrimitiveConfig` resource to manage rates.
- **Performance Considerations**: Standard system overhead; negligible.
- **API Improvements**: The `InvasionEvent` should return a result (`InvasionOutcome`) so the UI can display a summary.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer2/primitives.rs`.
- [ ] Active observation posts correctly increment the `science` pool in `ColonyResources`.
- [ ] Invading removes the `PrimitiveCivilization` component and spawns Layer 1 `Pop` entities with a `Slave` (or equivalent) tag.

## 7. Technical Guidance
- **Code Structure Suggestions**: Create `src/layer2/primitives.rs`. Ensure Layer 2 and Layer 1 crossover is handled carefully (e.g., how the new slaves are transported to the player's colony).
- **Integration Points**: Connect to the Chronicle via `AddChronicleEvent` to document "The Uplifting of Tau Ceti" or "The Tau Ceti Massacre".
- **Gotchas**: If the game doesn't currently support multi-planetary populations well, spawning the slaves might require an automatic transfer to the "Capital" colony or the nearest player settlement.

## 8. Questions
*Builder: add questions here if spec is unclear.*
