# 280: Archaeological Layers

## 1. Overview
The ground beneath your feet is a graveyard of civilizations. Deep terrain layers contain `Ruins` tiles. Excavating them yields `Artifacts` (lore/resources) but risks unleashing `Old World Maladies` (curses/diseases). This creates tension between digging deep for secrets and staying shallow for safety.

## 2. Dependencies
- `018` Mining and Resources
- `156` Xeno-Artifacts
- `034` Pop Health and Damage

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::inventory::Inventory;
    use crate::layer1::pop::Pop;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, handle_excavation_system);
        app
    }

    #[test]
    fn test_excavating_ruin_yields_artifact() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Pop,
            Inventory::new(10.0),
        )).id();

        let ruin_id = app.world_mut().spawn(RuinTile {
            malady_chance: 0.0, // Force no malady
        }).id();

        app.world_mut().send_event(ExcavateEvent {
            pop: pop_id,
            target: ruin_id,
        });

        app.update();

        let pop_inventory = app.world().get::<Inventory>(pop_id).unwrap();
        assert!(pop_inventory.count(ResourceType::Artifact) > 0);
        assert!(app.world().get_entity(ruin_id).is_none()); // Ruin destroyed
    }

    #[test]
    fn test_excavating_ruin_can_trigger_malady() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Pop,
            Inventory::new(10.0),
        )).id();

        let ruin_id = app.world_mut().spawn(RuinTile {
            malady_chance: 1.0, // Force malady
        }).id();

        app.world_mut().send_event(ExcavateEvent {
            pop: pop_id,
            target: ruin_id,
        });

        app.update();

        // Pop should be afflicted with a malady (e.g., Sickness component added)
        assert!(app.world().get::<Sickness>(pop_id).is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::inventory::Inventory;
use crate::layer1::resources::ResourceType;
use rand::Rng;

#[derive(Component)]
pub struct RuinTile {
    pub malady_chance: f32,
}

#[derive(Component)]
pub struct Sickness; // Assume imported from health system

#[derive(Event)]
pub struct ExcavateEvent {
    pub pop: Entity,
    pub target: Entity,
}

pub fn handle_excavation_system(
    mut events: EventReader<ExcavateEvent>,
    mut pops: Query<&mut Inventory>,
    ruins: Query<&RuinTile>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();

    for event in events.read() {
        if let Ok(ruin) = ruins.get(event.target) {
            if let Ok(mut inventory) = pops.get_mut(event.pop) {
                // Yield Artifact
                inventory.add(ResourceType::Artifact, 1.0);

                // Check for malady
                if rng.gen::<f32>() < ruin.malady_chance {
                    commands.entity(event.pop).insert(Sickness);
                }

                // Destroy the ruin tile
                commands.entity(event.target).despawn();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- Malady generation should be tied to a generic `Disease` or `Curse` event rather than inserting a raw `Sickness` component to allow for varied maladies.
- The `RuinTile` could have a tiered reward system, where deeper ruins yield rarer artifacts but carry higher malady risks.
- Excavation should likely take time (Action/Job) rather than being an instantaneous event, fitting into the existing `Mining` job system.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Excavating ruins provides an artifact to the miner's inventory.
- [ ] Excavating ruins carries a chance to apply a negative status effect to the miner.

## 7. Technical Guidance
- Add `RuinTile` logic to `src/layer1/terrain/ruins.rs`.
- Hook this into the `Mining` job logic, adding a special case for `RuinTile` targets.
- Ensure `ResourceType::Artifact` is registered if not already present from Spec 156.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
