# 426: The Archive of Regret

## 1. Overview
"The Archive of Regret" adds a massive, indestructible building to the colony that records all moral transgressions (Atrocities). Starving a Pop, orbital bombardment, executing prisoners, or cannibalism—each is logged into the Archive.

In exchange for this permanent record of sins, the Archive generates massive Research points (learning from past mistakes). However, as the Archive fills up, it exerts a permanent "Guilt" aura over the colony, severely depressing the Pops.

## 2. Dependencies
- `src/layer1/building.rs`: Building components and initialization.
- `src/layer1/chronicle.rs`: Event generation (AtrocityEvent).
- `src/layer1/morale.rs`: Applying global mood modifiers (Guilt).
- `src/layer1/research.rs`: Generating tech points.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::chronicle::AtrocityEvent;
    use crate::layer1::morale::MoodModifier;
    use crate::layer1::research::ResearchPoints;
    use crate::layer1::building::Building;

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<AtrocityEvent>>();
        world.insert_resource(ResearchPoints { total: 0.0 });
        world.spawn((
            Building { name: "Archive of Regret".to_string(), ..Default::default() },
            Archive { stored_atrocities: 0, aura_strength: 0.0 },
        ));
        world
    }

    #[test]
    fn test_archive_records_atrocity() {
        let mut world = setup_world();

        world.resource_mut::<Events<AtrocityEvent>>().send(AtrocityEvent {
            description: "Executed Prisoner 77".to_string(),
            severity: 5.0,
        });

        update_archive_system(&mut world);

        let mut query = world.query::<&Archive>();
        let archive = query.single(&world);
        assert_eq!(archive.stored_atrocities, 1);
        assert_eq!(archive.aura_strength, 5.0);
    }

    #[test]
    fn test_archive_generates_research() {
        let mut world = setup_world();

        // Setup an archive with stored atrocities
        let mut query = world.query::<&mut Archive>();
        for mut archive in query.iter_mut(&mut world) {
            archive.stored_atrocities = 10;
            archive.aura_strength = 50.0;
        }

        // Generate research based on contents
        generate_archive_research_system(&mut world);

        let research = world.resource::<ResearchPoints>();
        assert_eq!(research.total, 50.0, "Research points should scale with aura_strength");
    }

    #[test]
    fn test_archive_applies_guilt_aura() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            MoodModifier { current: 100.0, label: "Fine".to_string() }
        )).id();

        let mut query = world.query::<&mut Archive>();
        for mut archive in query.iter_mut(&mut world) {
            archive.stored_atrocities = 1;
            archive.aura_strength = 20.0;
        }

        apply_guilt_aura_system(&mut world);

        let mood = world.get::<MoodModifier>(pop).unwrap();
        assert_eq!(mood.current, 80.0, "Pop mood should be reduced by Archive aura strength");
        assert_eq!(mood.label, "Guilty Conscience");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::chronicle::AtrocityEvent;
use crate::layer1::morale::MoodModifier;
use crate::layer1::research::ResearchPoints;
use crate::layer1::building::Building;

#[derive(Component)]
pub struct Archive {
    pub stored_atrocities: u32,
    pub aura_strength: f32,
}

#[derive(Component)]
pub struct Pop;

pub fn update_archive_system(world: &mut World) {
    let mut events = world.resource_mut::<Events<AtrocityEvent>>();
    let mut atrocities = Vec::new();
    for event in events.drain() {
        atrocities.push(event);
    }

    let mut query = world.query::<&mut Archive>();
    for mut archive in query.iter_mut(world) {
        for atrocity in &atrocities {
            archive.stored_atrocities += 1;
            archive.aura_strength += atrocity.severity;
        }
    }
}

pub fn generate_archive_research_system(world: &mut World) {
    let mut total_aura = 0.0;

    let mut query = world.query::<&Archive>();
    for archive in query.iter(world) {
        total_aura += archive.aura_strength;
    }

    if total_aura > 0.0 {
        if let Some(mut research) = world.get_resource_mut::<ResearchPoints>() {
            research.total += total_aura;
        }
    }
}

pub fn apply_guilt_aura_system(world: &mut World) {
    let mut total_aura = 0.0;
    let mut query = world.query::<&Archive>();
    for archive in query.iter(world) {
        total_aura += archive.aura_strength;
    }

    if total_aura > 0.0 {
        let mut pop_query = world.query::<&mut MoodModifier>();
        for mut mood in pop_query.iter_mut(world) {
            mood.current -= total_aura;
            mood.label = "Guilty Conscience".to_string();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Reading:** Use an `EventReader<AtrocityEvent>` instead of mutating the global `Events` resource directly, as multiple systems might need to listen to atrocities.
- **Mood Modifier Strategy:** Constantly subtracting from `MoodModifier.current` in `apply_guilt_aura_system` is destructive; instead, register a persistent active modifier (e.g., in a list of active effects) that gets evaluated during the main morale calculation step.
- **Scaling Limits:** Cap `aura_strength` or apply a logarithmic curve to its effects on research and mood so the colony doesn't instantly collapse or instantly max out the tech tree.

## 6. Acceptance Criteria (Testable!)
- [ ] RED phase tests compile and pass.
- [ ] Atrocities sent as events increase `stored_atrocities` and `aura_strength`.
- [ ] The Archive generates research points scaling with `aura_strength`.
- [ ] The Archive reduces global Pop morale proportional to `aura_strength`.
- [ ] Code has >=85% test coverage.
- [ ] `cargo clippy -- -D warnings` passes cleanly.

## 7. Technical Guidance
- Ensure `apply_guilt_aura_system` integrates nicely with `src/layer1/morale.rs`. If the morale system expects passive tags, emit a global `GlobalAura::Guilt` resource instead of mutating all Pops directly.
- The Archive itself should be flagged as un-demolishable in the `Placement` or `Building` validation logic.

## 8. Questions
*Builder: add questions here if spec is unclear.*
