# Specification: 449 - The Overview Effect

## 1. Overview
The **Overview Effect** feature bridges Layer 1 and Layer 2/3. By building an "Observatory" on the colony surface, Pops can view the system and galaxy map. This interaction grants them immense "Knowledge" (Tech XP) but also inflicts "Existential Dread" (Stress) or "Inspiration" (Morale), depending on their individual traits and what is currently happening in orbit (e.g., a massive alien fleet approaching).

## 2. Dependencies
- `010` Chronicle System
- `094` System View Architecture
- `084` Pop Traits

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use scale::layer1::pop::{Pop, Trait};
    use scale::layer1::mood::{Mood, MoodModifier};
    use scale::layer1::skills::SkillSet;
    use scale::layer2::fleet::{Fleet, Faction};

    #[test]
    fn test_observatory_grants_knowledge() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(OverviewPlugin);

        let pop_entity = app.world.spawn((
            Pop,
            SkillSet::default(),
        )).id();

        // Act: Pop uses observatory
        app.world.send_event(ObserveEvent {
            pop: pop_entity,
        });
        app.update();

        // Assert: Pop gains Science XP
        let skills = app.world.get::<SkillSet>(pop_entity).unwrap();
        assert!(skills.science_xp > 0, "Observing must grant Science XP");
    }

    #[test]
    fn test_existential_dread_trait_reaction() {
        let mut app = App::new();
        app.add_plugins(OverviewPlugin);

        // Spawn pop with 'Anxious' trait
        let mut traits = Vec::new();
        traits.push(Trait::Anxious);

        let pop_entity = app.world.spawn((
            Pop,
            traits,
            Mood::default(),
        )).id();

        // Act
        app.world.send_event(ObserveEvent { pop: pop_entity });
        app.update();

        // Assert: Pop gains Dread (Negative Mood Modifier)
        let mood = app.world.get::<Mood>(pop_entity).unwrap();
        assert!(mood.modifiers.iter().any(|m| matches!(m, MoodModifier::ExistentialDread)),
                "Anxious pop should feel Dread when looking at the stars");
    }

    #[test]
    fn test_orbital_fleet_reaction() {
        let mut app = App::new();
        app.add_plugins(OverviewPlugin);

        // Spawn massive hostile fleet in Layer 2 (resource or global state)
        app.world.insert_resource(Layer2State {
            hostile_fleets_in_orbit: true,
        });

        let pop_entity = app.world.spawn((
            Pop,
            Vec::<Trait>::new(),
            Mood::default(),
        )).id();

        // Act
        app.world.send_event(ObserveEvent { pop: pop_entity });
        app.update();

        // Assert: Even without traits, seeing a hostile fleet causes massive stress/dread
        let mood = app.world.get::<Mood>(pop_entity).unwrap();
        assert!(mood.modifiers.iter().any(|m| matches!(m, MoodModifier::ExistentialDread)),
                "Seeing a hostile fleet must cause Dread");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass

#[derive(Event)]
pub struct ObserveEvent {
    pub pop: Entity,
}

#[derive(Resource, Default)]
pub struct Layer2State {
    pub hostile_fleets_in_orbit: bool,
}

pub fn overview_effect_system(
    mut events: EventReader<ObserveEvent>,
    mut query: Query<(&mut SkillSet, &Vec<Trait>, &mut Mood)>,
    l2_state: Option<Res<Layer2State>>,
) {
    let hostile_orbit = l2_state.map(|s| s.hostile_fleets_in_orbit).unwrap_or(false);

    for ev in events.read() {
        if let Ok((mut skills, traits, mut mood)) = query.get_mut(ev.pop) {
            skills.science_xp += 10;

            if traits.contains(&Trait::Anxious) || hostile_orbit {
                mood.modifiers.push(MoodModifier::ExistentialDread);
            } else if traits.contains(&Trait::Optimist) {
                 mood.modifiers.push(MoodModifier::Inspired);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Integration:** The `Layer2State` should be a bridge system that reads actual `Fleet` entities in the Layer 2 ECS world rather than a mock boolean resource.
- **Cooldowns:** Pops shouldn't be able to spam the telescope. Add an `ObservedRecently` component with a timer to prevent rapid cycling of the event.
- **Knowledge Scaling:** The amount of Science XP gained should scale with the colony's tech level or the specific events happening in the sky (e.g., observing a supernova grants a huge one-time boost).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Using the Observatory grants Science XP to the Pop.
- [ ] Pops with specific traits or seeing hostile fleets gain Existential Dread.

## 7. Technical Guidance
- Link the `ObserveEvent` to the `Utility AI System` (016) so Pops will autonomously choose to use the Observatory building during their `Leisure` time.
- Make sure `ExistentialDread` is registered in the mood calculation system to properly reduce the overall mood score.

## 8. Questions
*Builder: add questions here if spec is unclear.*
