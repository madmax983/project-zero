# 746 The Cargo Cult

## 1. Overview
**Layer:** 1
**Fantasy:** The devastating consequences of trying to help a primitive society without explaining how things work.
**Mechanic:** When interacting with pre-spaceflight civilizations or long-isolated splinter colonies, dropping advanced resources (like high-tech food or medicine) temporarily boosts their development. However, if done too frequently, they form a "Cargo Cult," completely abandoning their own agriculture and industry to build useless mock-ups of your dropships, waiting for the next delivery.

## 2. Dependencies
- `src/layer2/diplomacy.rs` (Interactions with primitives/splinter colonies)
- `src/layer1/resources.rs` (Resource dropping logic)
- `src/layer1/population.rs` (Primitive colony behavior/tags)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct PrimitiveColony {
        recent_resource_drops: u32,
    }

    #[derive(Component)]
    struct CargoCult;

    #[derive(Component, Default)]
    struct AgriculturalOutput(f32);

    #[derive(Event)]
    struct ResourceDropEvent {
        target: Entity,
        advanced: bool,
    }

    fn process_resource_drops(
        mut events: EventReader<ResourceDropEvent>,
        mut query: Query<(Entity, &mut PrimitiveColony), Without<CargoCult>>,
        mut commands: Commands,
    ) {
        for ev in events.read() {
            if ev.advanced {
                if let Ok((entity, mut colony)) = query.get_mut(ev.target) {
                    colony.recent_resource_drops += 1;
                    if colony.recent_resource_drops >= 3 {
                        commands.entity(entity).insert(CargoCult);
                    }
                }
            }
        }
    }

    fn cargo_cult_starvation_system(
        mut query: Query<&mut AgriculturalOutput, With<CargoCult>>,
    ) {
        for mut output in query.iter_mut() {
            output.0 = 0.0;
        }
    }

    #[test]
    fn test_frequent_drops_cause_cargo_cult() {
        let mut app = App::new();
        app.add_event::<ResourceDropEvent>();
        app.add_systems(Update, process_resource_drops);

        let primitive_colony = app
            .world_mut()
            .spawn(PrimitiveColony {
                recent_resource_drops: 0,
            })
            .id();

        // Send 3 advanced drops
        for _ in 0..3 {
            app.world_mut().send_event(ResourceDropEvent {
                target: primitive_colony,
                advanced: true,
            });
            app.update();
        }

        assert!(app.world().entity(primitive_colony).contains::<CargoCult>());
    }

    #[test]
    fn test_cargo_cult_ruins_agriculture() {
        let mut app = App::new();
        app.add_systems(Update, cargo_cult_starvation_system);

        let cult_colony = app
            .world_mut()
            .spawn((
                PrimitiveColony {
                    recent_resource_drops: 3,
                },
                CargoCult,
                AgriculturalOutput(100.0),
            ))
            .id();

        app.update();

        assert_eq!(
            app.world()
                .entity(cult_colony)
                .get::<AgriculturalOutput>()
                .unwrap()
                .0,
            0.0
        );
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Stub:
// pub struct CargoCult;

// In `src/layer2/diplomacy.rs` (or equivalent system processing drops):
// Track `ResourceDropEvent` or the action representing advanced supply drops.
// Increment a counter `recent_resource_drops` on the target `PrimitiveColony`.
// If `recent_resource_drops >= THRESHOLD`, insert `CargoCult` component.

// In systems managing output for primitive colonies:
// Force `AgriculturalOutput` (and industrial output equivalent) to `0.0` for entities `With<CargoCult>`.
```

## 5. REFACTOR Phase: Quality & Design
- Create a visual representation (e.g. Chronicle events) when the Cargo Cult forms so the player understands *why* the colony is suddenly starving.
- Ensure the `recent_resource_drops` counter decays slowly over time to allow players to safely provide infrequent aid.
- Add a diplomatic action to "educate" the cult, slowly removing the trait at the cost of high influence/resources.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Dropping advanced resources multiple times triggers the `CargoCult` state.
- [ ] A colony with `CargoCult` produces 0 local food/industry.

## 7. Technical Guidance
- The counter should decay gracefully (e.g., `-1` drop count per season or year) so that infrequent aid works as intended.
- `AgriculturalOutput` might be represented differently in the current codebase; hook into whatever drives local food generation for NPC colonies.

## 8. Questions
*Builder: add questions here if spec is unclear.*
