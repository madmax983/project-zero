# Generational Vengeance

## 1. Overview
The Generational Vengeance feature tracks unresolved grievances between Pops and allows those grievances to be inherited by their descendants. If a Pop dies with a "Grudge" against another Pop, the grudge is passed down to their children. Over generations, what started as a petty feud over a tool or ration can escalate. If a descendant harboring an ancient grudge rises to power (e.g., becoming a Faction Leader or Fleet Commander), that personal grudge becomes a faction-level penalty or even a war goal. This applies across layers (Layer 1 -> Layer 3).

## 2. Dependencies
- `src/layer1/pop.rs`: Birth and death lifecycle systems for inheriting traits.
- `src/layer1/relationships.rs`: Managing affinity and grudges between entities.
- `src/layer3/faction.rs`: Translating personal grudges into faction-level diplomatic penalties or war goals.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Mock components for testing
    #[derive(Component, Debug, Clone, PartialEq)]
    struct Pop {
        id: u32,
    }

    #[derive(Component, Debug, Clone)]
    struct Grudge {
        target_id: u32,
        intensity: f32,
        origin_reason: String,
    }

    #[derive(Component)]
    struct GrudgeList(Vec<Grudge>);

    #[derive(Component)]
    struct Lineage {
        parent_id: Option<u32>,
    }

    #[derive(Event)]
    struct PopDiedEvent(Entity);

    #[derive(Event)]
    struct PopBornEvent {
        child: Entity,
        parent: Entity,
    }

    // 1. Test that a child inherits the grudges of its parent upon birth
    #[test]
    fn test_child_inherits_parent_grudges() {
        let mut app = App::new();
        app.add_event::<PopBornEvent>();
        app.add_systems(Update, inherit_grudges_on_birth_system);

        // Spawn a parent with a grudge
        let parent = app.world_mut().spawn((
            Pop { id: 1 },
            GrudgeList(vec![Grudge {
                target_id: 2,
                intensity: 50.0,
                origin_reason: "Stole a ration".to_string(),
            }]),
        )).id();

        // Spawn a child
        let child = app.world_mut().spawn((
            Pop { id: 3 },
            Lineage { parent_id: Some(1) },
            GrudgeList(vec![]),
        )).id();

        app.world_mut().send_event(PopBornEvent { child, parent });
        app.update();

        // Verify child inherited the grudge
        let child_grudges = app.world().get::<GrudgeList>(child).unwrap();
        assert_eq!(child_grudges.0.len(), 1);
        assert_eq!(child_grudges.0[0].target_id, 2);
        assert_eq!(child_grudges.0[0].origin_reason, "Stole a ration");
    }

    // 2. Test that when a Pop dies, their grudge is transferred to their living descendants
    #[test]
    fn test_grudge_transfers_on_death() {
        let mut app = App::new();
        app.add_event::<PopDiedEvent>();
        app.add_systems(Update, transfer_grudges_on_death_system);

        // Spawn a parent with a grudge
        let parent = app.world_mut().spawn((
            Pop { id: 1 },
            GrudgeList(vec![Grudge {
                target_id: 2,
                intensity: 80.0,
                origin_reason: "Killed my kin".to_string(),
            }]),
        )).id();

        // Spawn a child who does NOT yet have the grudge (perhaps born before the grudge started)
        let child = app.world_mut().spawn((
            Pop { id: 3 },
            Lineage { parent_id: Some(1) },
            GrudgeList(vec![]),
        )).id();

        // Parent dies
        app.world_mut().send_event(PopDiedEvent(parent));
        app.update();

        // Verify child now has the grudge
        let child_grudges = app.world().get::<GrudgeList>(child).unwrap();
        assert_eq!(child_grudges.0.len(), 1);
        assert_eq!(child_grudges.0[0].target_id, 2);
        // The intensity might amplify slightly upon death to represent "avenging"
        assert!(child_grudges.0[0].intensity >= 80.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Components
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Pop {
    pub id: u32,
}

#[derive(Component, Debug, Clone)]
pub struct Grudge {
    pub target_id: u32,
    pub intensity: f32,
    pub origin_reason: String,
}

#[derive(Component, Clone)]
pub struct GrudgeList(pub Vec<Grudge>);

#[derive(Component)]
pub struct Lineage {
    pub parent_id: Option<u32>,
}

// Events
#[derive(Event)]
pub struct PopDiedEvent(pub Entity);

#[derive(Event)]
pub struct PopBornEvent {
    pub child: Entity,
    pub parent: Entity,
}

// Systems
pub fn inherit_grudges_on_birth_system(
    mut events: EventReader<PopBornEvent>,
    parent_query: Query<&GrudgeList>,
    mut child_query: Query<&mut GrudgeList>,
) {
    for event in events.read() {
        if let Ok(parent_grudges) = parent_query.get(event.parent) {
            if let Ok(mut child_grudges) = child_query.get_mut(event.child) {
                // Inherit all grudges
                for grudge in &parent_grudges.0 {
                    child_grudges.0.push(grudge.clone());
                }
            }
        }
    }
}

pub fn transfer_grudges_on_death_system(
    mut events: EventReader<PopDiedEvent>,
    dead_query: Query<(&Pop, &GrudgeList)>,
    mut descendant_query: Query<(&Lineage, &mut GrudgeList)>,
) {
    for event in events.read() {
        if let Ok((dead_pop, dead_grudges)) = dead_query.get(event.0) {
            // Find all children and pass on the grudge
            for (lineage, mut child_grudges) in descendant_query.iter_mut() {
                if lineage.parent_id == Some(dead_pop.id) {
                    for grudge in &dead_grudges.0 {
                        // Check if the child already has a grudge against this target
                        let mut found = false;
                        for existing_grudge in &mut child_grudges.0 {
                            if existing_grudge.target_id == grudge.target_id {
                                // Amplify existing grudge
                                existing_grudge.intensity += grudge.intensity * 0.5;
                                found = true;
                                break;
                            }
                        }

                        if !found {
                            // Inherit new grudge, amplifying slightly for "vengeance"
                            let mut inherited_grudge = grudge.clone();
                            inherited_grudge.intensity *= 1.2; // +20% intensity on death inheritance
                            child_grudges.0.push(inherited_grudge);
                        }
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Refactor 1: Target Resolution.** Currently, grudges track the `target_id` (a `u32` representing the Pop's ID). If the *target* of the grudge dies, does the grudge transfer to the target's descendants? The spec implies this ("descendants of miner A form a faction and blockade the surface planet governed by descendants of miner B"). The current code only transfers the *holder* of the grudge. We need a secondary system that transfers the *target* of the grudge to their children upon death.
- **Refactor 2: Faction Integration.** The `GrudgeList` needs to be read by the Layer 3 Faction AI. If a Pop with a high-intensity grudge becomes a Faction Leader, their personal `GrudgeList` should apply a massive negative diplomatic modifier to the faction controlled by their target (or their target's descendants).
- **Refactor 3: Generational Decay.** To prevent infinite grudge stacking over 300 years, there should be a slow, generational decay of intensity, counteracted only if new offenses occur between the families.

## 6. Acceptance Criteria

- [ ] Children correctly inherit all grudges from their parent upon birth.
- [ ] When a Pop dies, their living descendants inherit or amplify their grudges.
- [ ] Grudge intensity amplifies upon death (Vengeance modifier).
- [ ] All RED phase tests pass (`cargo test` returns 0 failures).
- [ ] Clippy warnings resolved.
- [ ] Test coverage ≥ 85% for new implementation.

## 7. Technical Guidance

- Pay close attention to how Bevy handles events and query mutability. `PopDiedEvent` implies the entity is about to be despawned or has just "died" but its components are still accessible. Depending on the architecture, you may need to read the components *before* despawning in a death-handler system.
- Consider utilizing a "Family Tree" or "Dynasty" resource if tracing `parent_id` through queries becomes too expensive for large populations over many generations.

## 8. Questions
*Builder: add questions here if spec is unclear.*
