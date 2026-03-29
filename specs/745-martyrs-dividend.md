# 745 The Martyr's Dividend

## 1. Overview
**Layer:** Cross-layer (1 -> 3)
**Fantasy:** Weaponizing tragedy to fuel an empire's expansion.
**Mechanic:** When a Pop dies under highly specific, dramatic circumstances (e.g., starvation during a siege, crushed by a failing planetary engine), they become a "Martyr." This generates a massive, temporary spike in a unique Layer 3 resource: "Zeal." Zeal can be used to instantly complete massive megaprojects or force through wildly unpopular edicts without Unrest.

## 2. Dependencies
- `src/layer1/population.rs` (Pop death events)
- `src/layer3/resources.rs` (Layer 3 resources/empire state)
- `src/layer1/edicts.rs` (Applying Edicts/Unrest)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct Pop;

    #[derive(Event)]
    struct PopDeathEvent {
        entity: Entity,
        cause: DeathCause,
    }

    #[derive(PartialEq, Eq)]
    enum DeathCause {
        OldAge,
        StarvationSiege, // Qualifies for Martyrdom
        CrushedByEngine, // Qualifies for Martyrdom
    }

    #[derive(Resource, Default)]
    struct Zeal(u32);

    #[derive(Event)]
    struct EdictEnactedEvent {
        edict_type: EdictType,
        used_zeal: bool,
    }

    #[derive(PartialEq, Eq)]
    enum EdictType {
        ForcedLabor,
    }

    #[derive(Resource, Default)]
    struct Unrest(u32);

    fn process_martyrdom_system(
        mut events: EventReader<PopDeathEvent>,
        mut zeal: ResMut<Zeal>,
    ) {
        for ev in events.read() {
            if ev.cause == DeathCause::StarvationSiege || ev.cause == DeathCause::CrushedByEngine {
                zeal.0 += 100;
            }
        }
    }

    fn apply_edict_system(
        mut events: EventReader<EdictEnactedEvent>,
        mut unrest: ResMut<Unrest>,
        mut zeal: ResMut<Zeal>,
    ) {
        for ev in events.read() {
            if ev.used_zeal {
                if zeal.0 >= 50 {
                    zeal.0 -= 50;
                    // No unrest added!
                } else {
                    // Fallback to normal behavior if zeal is requested but missing
                    unrest.0 += 50;
                }
            } else {
                unrest.0 += 50;
            }
        }
    }

    #[test]
    fn test_martyrdom_generates_zeal() {
        let mut app = App::new();
        app.add_event::<PopDeathEvent>();
        app.init_resource::<Zeal>();
        app.add_systems(Update, process_martyrdom_system);

        let entity = app.world_mut().spawn(Pop).id();
        app.world_mut().send_event(PopDeathEvent {
            entity,
            cause: DeathCause::StarvationSiege,
        });

        app.update();

        assert_eq!(app.world().resource::<Zeal>().0, 100);
    }

    #[test]
    fn test_zeal_bypasses_unrest_for_edicts() {
        let mut app = App::new();
        app.add_event::<EdictEnactedEvent>();
        app.insert_resource(Zeal(100));
        app.init_resource::<Unrest>();
        app.add_systems(Update, apply_edict_system);

        app.world_mut().send_event(EdictEnactedEvent {
            edict_type: EdictType::ForcedLabor,
            used_zeal: true, // Use Zeal!
        });

        app.update();

        assert_eq!(app.world().resource::<Zeal>().0, 50); // Consumed zeal
        assert_eq!(app.world().resource::<Unrest>().0, 0); // No unrest!
    }

    #[test]
    fn test_edict_without_zeal_causes_unrest() {
        let mut app = App::new();
        app.add_event::<EdictEnactedEvent>();
        app.init_resource::<Zeal>();
        app.init_resource::<Unrest>();
        app.add_systems(Update, apply_edict_system);

        app.world_mut().send_event(EdictEnactedEvent {
            edict_type: EdictType::ForcedLabor,
            used_zeal: false,
        });

        app.update();

        assert_eq!(app.world().resource::<Unrest>().0, 50); // Unrest generated
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// In `src/layer3/resources.rs` (or equivalent):
// pub struct Zeal(pub u32);

// In `src/layer1/population.rs` or `src/simulation.rs`:
// Hook into `PopDeathEvent`. Map specific dramatic causes (Needs.hunger == 0 AND UnderSiege, or structural collapse) to generate `Zeal`.

// In `src/layer1/edicts.rs`:
// Modify edict application functions/systems to accept an optional `consume_zeal` parameter.
// If consumed, skip adding Unrest/Negative Morale.
```

## 5. REFACTOR Phase: Quality & Design
- Zeal should probably decay over time so players can't hoard it from an early crisis to use endlessly in the late game.
- Ensure the UI clearly shows the player the tradeoff: a tragic death vs. the sudden influx of Zeal.
- Integrate with the Chronicle: Log the martyrdom specifically.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] "Dramatic" deaths generate Zeal. Normal deaths (old age, simple accident) do not.
- [ ] Edicts can consume Zeal to prevent Unrest generation.

## 7. Technical Guidance
- Be careful with `DeathCause` enum or equivalent representation. You may need to inspect the *context* of the death at the time of the event (e.g., was the colony under siege?) rather than relying solely on the cause of death being `Starvation`.
- Zeal is a global resource (Layer 3), so ensure the system applying it has access to the appropriate resource context.

## 8. Questions
*Builder: add questions here if spec is unclear.*
