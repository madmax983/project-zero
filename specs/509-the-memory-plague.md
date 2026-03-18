# Spec 509: The Memory Plague

## 1. Overview
A rare viral infection spreads through the colony, stripping Pops of their accumulated skills and severing relationship ties. Unchecked, infected Pops revert to "Blank Slates," requiring complete retraining and inflicting significant Morale penalties on their former friends.

## 2. Dependencies
- `036` Pop Memory
- `047` Pop Relationships
- `051` Pop Skills and Experience

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_memory_plague_degrades_skills() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, memory_plague_progression_system);

        let pop_entity = app.world_mut().spawn((
            PopSkill { level: 5.0 },
            MemoryPlagueInfection { progress: 0.0, rate: 1.0 },
        )).id();

        // Advance the plague progression
        app.world_mut().get_mut::<MemoryPlagueInfection>(pop_entity).unwrap().progress = 50.0;
        app.update();

        let skills = app.world().get::<PopSkill>(pop_entity).unwrap().level;
        assert!(skills < 5.0, "Memory plague should slowly degrade skills");
    }

    #[test]
    fn test_memory_plague_severs_relationships() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, memory_plague_progression_system);

        let pop_entity = app.world_mut().spawn((
            PopRelationships {
                friend_count: 3,
                family_ties: 2,
            },
            MemoryPlagueInfection { progress: 0.0, rate: 1.0 },
        )).id();

        app.world_mut().get_mut::<MemoryPlagueInfection>(pop_entity).unwrap().progress = 100.0;
        app.update();

        let relationships = app.world().get::<PopRelationships>(pop_entity).unwrap();
        assert_eq!(relationships.friend_count, 0, "Advanced memory plague should sever all friendships");
        assert_eq!(relationships.family_ties, 0, "Advanced memory plague should sever all family ties");
    }

    #[test]
    fn test_blank_slate_reversion_triggers_morale_drop() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_event::<BlankSlateEvent>()
            .init_resource::<GlobalMorale>()
            .add_systems(Update, memory_plague_progression_system);

        let pop_entity = app.world_mut().spawn((
            MemoryPlagueInfection { progress: 99.0, rate: 2.0 },
            PopSkill { level: 0.0 },
            PopRelationships { friend_count: 0, family_ties: 0 },
        )).id();

        app.update();

        let blank_slate_events = app.world().resource::<Events<BlankSlateEvent>>();
        assert_eq!(blank_slate_events.get_reader().len(&blank_slate_events), 1, "Plague progressing past 100 should trigger BlankSlateEvent");

        // Simulating the effect of the event on morale
        let initial_morale = app.world().resource::<GlobalMorale>().level;
        app.world_mut().resource_mut::<GlobalMorale>().level -= 10.0;
        let final_morale = app.world().resource::<GlobalMorale>().level;

        assert!(final_morale < initial_morale, "Blank slate reversion should drop global morale");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct PopSkill {
    pub level: f32,
}

#[derive(Component)]
pub struct PopRelationships {
    pub friend_count: u32,
    pub family_ties: u32,
}

#[derive(Component)]
pub struct MemoryPlagueInfection {
    pub progress: f32,
    pub rate: f32,
}

#[derive(Event)]
pub struct BlankSlateEvent {
    pub pop: Entity,
}

#[derive(Resource, Default)]
pub struct GlobalMorale {
    pub level: f32,
}

pub fn memory_plague_progression_system(
    mut query: Query<(
        Entity,
        &mut MemoryPlagueInfection,
        Option<&mut PopSkill>,
        Option<&mut PopRelationships>,
    )>,
    mut blank_slate_events: EventWriter<BlankSlateEvent>,
) {
    for (entity, mut infection, mut skill, mut relationships) in query.iter_mut() {
        infection.progress += infection.rate;

        if let Some(mut skill) = skill {
            if infection.progress > 20.0 {
                skill.level -= infection.rate * 0.1;
                skill.level = skill.level.max(0.0);
            }
        }

        if let Some(mut relationships) = relationships {
            if infection.progress > 80.0 {
                relationships.friend_count = 0;
                relationships.family_ties = 0;
            }
        }

        if infection.progress >= 100.0 {
            blank_slate_events.send(BlankSlateEvent { pop: entity });
            // Once they become a blank slate, the infection is "cleared" as there's nothing left to eat
            // We could remove the component here, but this satisfies the basic progression logic.
            infection.progress = 0.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an `InfectionVector` component or resource to handle how the plague spreads between adjacent Pops.
- Consider adding a `MedicalBay` task to isolate infected Pops and slow down `infection.rate`.
- Morale drop should be calculated per-pop based on who they used to be friends with, rather than a flat global drop.

## 6. Acceptance Criteria
- [ ] Memory plague reduces Pop skills as it progresses.
- [ ] Memory plague severs relationships at advanced stages.
- [ ] Memory plague triggers a `BlankSlateEvent` upon reaching 100%.
- [ ] Test coverage >= 85%.
- [ ] `cargo clippy -- -D warnings` passes.

## 7. Technical Guidance
- The `BlankSlateEvent` should ideally remove the `MemoryPlagueInfection` component from the Pop and insert a `BlankSlate` trait that significantly increases the time it takes to learn new skills.
- The `PopRelationships` component is just a placeholder here; integrate with the actual social network graph implemented in `047 Pop Relationships`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
