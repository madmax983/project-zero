# 087: Omens & Taboos

## Overview

The colony is not just a machine; it is a community of people who try to make sense of a hostile world. When tragedies occur, colonists develop superstitions ("Omens") that evolve into "Taboos" against specific actions.

If a mine collapses, mining becomes "cursed". If a farmer dies of starvation, farming becomes "fearful". Pops performing these Taboo actions suffer significant stress (Leisure decay), creating a tension between survival needs and psychological safety.

## Dependencies

- `010` — Chronicle System (Event logging)
- `021` — Utility AI (Action Types)
- `031` — Pop Morale (Leisure/Stress mechanics)
- `071` — Structural Integrity (Cave-in logic)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/taboo.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::actions::{ActionType, PopAction};
    use crate::layer1::needs::Needs;
    use crate::layer1::structural_integrity::StructureCollapsed;
    use crate::layer1::pop::PopDied;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_taboo_resource_default() {
        let active = ActiveTaboos::default();
        assert!(active.entries.is_empty());
    }

    #[test]
    fn test_trigger_taboo_on_collapse() {
        let mut world = World::new();
        world.insert_resource(ActiveTaboos::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());

        // Send event
        world.send_event(StructureCollapsed {
            position: crate::layer1::map::GridPosition { x: 5, y: 5 },
            caused_by_entity: None,
        });

        // Run system
        let mut schedule = Schedule::new();
        schedule.add_systems(trigger_taboos_system);
        schedule.run(&mut world);

        let active = world.resource::<ActiveTaboos>();
        assert_eq!(active.entries.len(), 1);
        assert_eq!(active.entries[0].action, ActionType::Work); // Mining/Building is Work
        assert!(active.entries[0].description.contains("collapsed"));
    }

    #[test]
    fn test_trigger_taboo_on_starvation_death() {
        let mut world = World::new();
        world.insert_resource(ActiveTaboos::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());

        world.send_event(PopDied {
            entity: Entity::PLACEHOLDER,
            name: "Bob".to_string(),
            tick: 0,
            reason: "Starvation".to_string(),
        });

        let mut schedule = Schedule::new();
        schedule.add_systems(trigger_taboos_system);
        schedule.run(&mut world);

        let active = world.resource::<ActiveTaboos>();
        assert_eq!(active.entries.len(), 1);
        assert_eq!(active.entries[0].action, ActionType::Farm);
    }

    #[test]
    fn test_apply_taboo_stress() {
        let mut world = World::new();
        // Set up active taboo against Work
        let mut taboos = ActiveTaboos::default();
        taboos.add(Taboo {
            action: ActionType::Work,
            expiration: 1000,
            description: "Cursed Mines".to_string(),
        });
        world.insert_resource(taboos);
        world.insert_resource(crate::shared::time::SimulationTime { tick: 500 }); // Active

        // Spawn pop doing Work
        let pop = world.spawn((
            PopAction {
                current: ActionType::Work,
                ..Default::default()
            },
            Needs {
                leisure: 1.0,
                ..Default::default()
            },
        )).id();

        // Run effect system
        let mut schedule = Schedule::new();
        schedule.add_systems(apply_taboo_effects_system);
        schedule.run(&mut world);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure < 1.0, "Leisure should decay due to stress");
    }

    #[test]
    fn test_taboo_expiration() {
        let mut world = World::new();
        let mut taboos = ActiveTaboos::default();
        taboos.add(Taboo {
            action: ActionType::Work,
            expiration: 100,
            description: "Old news".to_string(),
        });
        world.insert_resource(taboos);
        world.insert_resource(crate::shared::time::SimulationTime { tick: 200 }); // Expired

        // Run cleanup/trigger system (assuming trigger cleans up or separate system)
        let mut schedule = Schedule::new();
        schedule.add_systems(cleanup_expired_taboos_system);
        schedule.run(&mut world);

        let active = world.resource::<ActiveTaboos>();
        assert!(active.entries.is_empty());
    }

    #[test]
    fn test_structure_collapsed_event_exists() {
        // Just verify struct exists (compiler check)
        let _ = StructureCollapsed {
            position: crate::layer1::map::GridPosition { x: 0, y: 0 },
            caused_by_entity: None,
        };
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Structure Collapsed Event

Update `src/layer1/structural_integrity.rs`:

```rust
// Add event definition
#[derive(Event, Debug, Clone)]
pub struct StructureCollapsed {
    pub position: crate::layer1::map::GridPosition,
    pub caused_by_entity: Option<Entity>,
}

// Update apply_collapse signature to include EventWriter
pub fn apply_collapse(
    world: &mut World,
    pos: GridPosition,
) {
    // ... existing logic ...

    // Emit event
    world.send_event(StructureCollapsed {
        position: pos,
        caused_by_entity: None, // Hard to track for now
    });
}
```

### 2. Taboo System

Create `src/layer1/taboo.rs`:

```rust
use bevy_ecs::prelude::*;
use crate::layer1::actions::{ActionType, PopAction};
use crate::layer1::needs::Needs;
use crate::layer1::structural_integrity::StructureCollapsed;
use crate::layer1::pop::PopDied;
use crate::shared::time::SimulationTime;
use crate::shared::log::MessageLog;

#[derive(Clone, Debug)]
pub struct Taboo {
    pub action: ActionType,
    pub expiration: u64,
    pub description: String,
}

#[derive(Resource, Default)]
pub struct ActiveTaboos {
    pub entries: Vec<Taboo>,
}

impl ActiveTaboos {
    pub fn add(&mut self, taboo: Taboo) {
        // Prevent duplicate types to avoid stacking punishment too hard
        if !self.entries.iter().any(|t| t.action == taboo.action) {
            self.entries.push(taboo);
        }
    }
}

pub fn trigger_taboos_system(
    mut taboos: ResMut<ActiveTaboos>,
    mut events_collapse: EventReader<StructureCollapsed>,
    mut events_death: EventReader<PopDied>,
    time: Res<SimulationTime>,
    mut log: Option<ResMut<MessageLog>>,
) {
    let current_tick = time.tick;
    let duration = 500; // Taboo lasts 500 ticks (~half a day)

    for _evt in events_collapse.read() {
        taboos.add(Taboo {
            action: ActionType::Work,
            expiration: current_tick + duration,
            description: "The earth is angry (Cave-in)".to_string(),
        });
        if let Some(l) = log.as_mut() {
            l.add("OMEN: A cave-in has made Work fearful!".to_string());
        }
    }

    for evt in events_death.read() {
        if evt.reason == "Starvation" {
            taboos.add(Taboo {
                action: ActionType::Farm,
                expiration: current_tick + duration,
                description: "The harvest is cursed (Starvation)".to_string(),
            });
            if let Some(l) = log.as_mut() {
                l.add("OMEN: Starvation has made Farming fearful!".to_string());
            }
        }
        // Future: Handle other death types (Combat -> Fight taboo?)
    }
}

pub fn cleanup_expired_taboos_system(
    mut taboos: ResMut<ActiveTaboos>,
    time: Res<SimulationTime>,
) {
    let current_tick = time.tick;
    taboos.entries.retain(|t| t.expiration > current_tick);
}

pub fn apply_taboo_effects_system(
    taboos: Res<ActiveTaboos>,
    mut query: Query<(&PopAction, &mut Needs)>,
) {
    if taboos.entries.is_empty() {
        return;
    }

    for (action, mut needs) in &mut query {
        for taboo in &taboos.entries {
            if action.current == taboo.action {
                // Apply stress (reduce leisure)
                // 0.005 per tick is significant (5x normal decay)
                needs.leisure = (needs.leisure - 0.005).max(0.0);
            }
        }
    }
}
```

### 3. Registration

Update `src/layer1/mod.rs` and `src/main.rs` (or simulation setup) to register the new module, events, resource, and systems.

## REFACTOR Phase: Quality & Design

- **Visual Feedback**: Pops working under a Taboo should have a specific icon or particle effect (e.g., a "Sweat" drop or "Skull").
- **Notifications**: Use the Notification system (Spec 046) instead of just MessageLog.
- **Duration Scaling**: Taboo duration could scale with event severity (e.g., multiple deaths = longer taboo).
- **Trait Interaction**: "Brave" pops might ignore taboos; "Superstitious" pops might take double stress.

## Acceptance Criteria

- [ ] `StructureCollapsed` event is defined and emitted on collapse.
- [ ] `ActiveTaboos` resource tracks current taboos.
- [ ] Collapse event triggers `ActionType::Work` taboo.
- [ ] Starvation death triggers `ActionType::Farm` taboo.
- [ ] Pops performing taboo actions lose `Leisure` (Stress) significantly faster.
- [ ] Taboos expire after a set duration.
- [ ] All tests pass.

## Technical Guidance

- **Event Reader Consumption**: Remember that `EventReader` consumes events. If other systems need these events, ensure ordering or use manual iteration if Bevy's event clearing behavior is tricky (usually fine within same frame).
- **Performance**: `apply_taboo_effects_system` runs every tick on all pops. Keep it lightweight (nested loop is small since taboos.len() is usually 0-2).
- **Module Structure**: Place `taboo.rs` in `src/layer1/`.

## Questions

*Builder: add questions here if spec is unclear.*
