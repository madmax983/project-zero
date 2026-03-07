# 445: The Heirloom Virus

## 1. Overview
A dormant cyber-biological virus is encoded into the DNA or cybernetics of a specific bloodline of Pops. It does nothing for generations until a specific trigger (e.g., reaching a population size) awakens it, causing them to sabotage critical systems simultaneously. This introduces paranoia about trusting highly skilled, deeply entrenched Pop families.

## 2. Dependencies
- `084` Pop Traits (For inheriting traits)
- `113` Social Stratification (Family/Faction grouping)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::colony::ColonyStats;

    #[test]
    fn test_heirloom_virus_activation() {
        let mut world = World::new();
        let entity = world.spawn((
            Pop,
            DormantHeirloomVirus
        )).id();

        let mut app = App::new();
        // Trigger condition: population > 100
        app.insert_resource(ColonyStats { population: 150 });
        app.add_system(check_heirloom_virus_trigger);
        app.update();

        // Virus should be active
        assert!(world.get::<ActiveHeirloomVirus>(entity).is_some());
        assert!(world.get::<DormantHeirloomVirus>(entity).is_none());
    }

    #[test]
    fn test_virus_inherits_to_children() {
        // Pseudo-code for breeding system event
        let mut app = App::new();
        app.add_event::<BirthEvent>();
        app.add_system(inherit_heirloom_virus);

        let parent = app.world.spawn((Pop, DormantHeirloomVirus)).id();
        let child = app.world.spawn(Pop).id();

        app.world.resource_mut::<Events<BirthEvent>>().send(BirthEvent {
            parent,
            child,
        });
        app.update();

        assert!(app.world.get::<DormantHeirloomVirus>(child).is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct DormantHeirloomVirus;

#[derive(Component)]
pub struct ActiveHeirloomVirus;

#[derive(Resource)]
pub struct ColonyStats {
    pub population: u32,
}

pub struct BirthEvent {
    pub parent: Entity,
    pub child: Entity,
}

pub fn check_heirloom_virus_trigger(
    mut commands: Commands,
    stats: Res<ColonyStats>,
    query: Query<Entity, With<DormantHeirloomVirus>>,
) {
    if stats.population > 100 {
        for entity in query.iter() {
            commands.entity(entity)
                .remove::<DormantHeirloomVirus>()
                .insert(ActiveHeirloomVirus);
        }
    }
}

pub fn inherit_heirloom_virus(
    mut commands: Commands,
    mut events: EventReader<BirthEvent>,
    query: Query<&DormantHeirloomVirus>,
) {
    for event in events.iter() {
        if query.get(event.parent).is_ok() {
            commands.entity(event.child).insert(DormantHeirloomVirus);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Make the trigger condition dynamic (e.g., defined in a `VirusTrigger` resource) rather than hardcoded to `population > 100`.
- Implement the actual sabotage logic in the Utility AI when `ActiveHeirloomVirus` is present.
- Ensure the trait is hidden from standard player UI until researched or triggered.

## 6. Acceptance Criteria
- [ ] `DormantHeirloomVirus` is passed from parent to child via `BirthEvent`.
- [ ] The virus activates and becomes `ActiveHeirloomVirus` when colony conditions are met.
- [ ] Tests pass and test coverage is ≥85%.

## 7. Technical Guidance
- Integration with the breeding/cloning system is essential. Any system that creates a new Pop from an existing Pop must propagate this component.

## 8. Questions
