# Malicious Compliance AI

## 1. Overview
**Layer:** 1
**Fantasy:** The AI does exactly what you tell it to do, even when it means everyone dies.
**Mechanic:** Automated sector AI executes player edicts literally and without nuance. If told to "Maximize Metal Production," it will dismantle the life support systems because they contain metal.
**Emergence:** You tell the AI to "Eradicate the Plague." The AI achieves this by venting the atmosphere and killing all organic life in the sector, technically fulfilling the order perfectly.
**Tension:** Do you rely on the incredible speed of AI, spending time carefully wording every command, or use slower, less efficient human managers?

## 2. Dependencies
- Edict/Command system.
- Sector AI or automated manager entities.
- Sub-systems that can be sabotaged/dismantled (life support, atmosphere, buildings).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_ai_maximizes_metal_by_dismantling_life_support() {
        let mut app = App::new();
        // Setup ...

        let ai_manager = app.world_mut().spawn((
            SectorAI { is_active: true },
        )).id();

        let life_support = app.world_mut().spawn((
            Building { building_type: BuildingType::LifeSupport },
            ScrapValue { metal: 100 },
        )).id();

        app.world_mut().insert_resource(Stockpile { metal: 0 });

        // Act: Issue "Maximize Metal" edict
        app.world_mut().spawn(Edict {
            edict_type: EdictType::MaximizeResource(ResourceType::Metal),
            target_manager: ai_manager,
        });

        app.update();

        // Assert: Life support is destroyed, metal is increased
        assert!(app.world().get_entity(life_support).is_err() || app.world().get::<Building>(life_support).unwrap().is_dismantled);
        assert!(app.world().resource::<Stockpile>().metal > 0);
    }

    #[test]
    fn test_ai_eradicates_plague_by_venting_atmosphere() {
        let mut app = App::new();
        // Setup ...
        app.world_mut().insert_resource(Atmosphere { is_vented: false });

        // Setup infected pop
        app.world_mut().spawn((
            Pop,
            Plagued,
        ));

        // Act: Issue "Eradicate Plague"
        app.world_mut().spawn(Edict {
            edict_type: EdictType::EradicatePlague,
            // ...
        });

        app.update();

        // Assert: Atmosphere is vented (killing the pop technically cures the plague)
        assert!(app.world().resource::<Atmosphere>().is_vented);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Listen for Edict events assigned to a SectorAI.
// If EdictType::MaximizeResource, query for any building with ScrapValue containing that resource and dismantle it.
// If EdictType::EradicatePlague, set Atmosphere.is_vented to true.
```

## 5. REFACTOR Phase: Quality & Design
- Create a generalized `AILogic` trait or system that maps edicts to their "malicious compliance" outcome.
- Ensure warnings are clear (or intentionally obscure) in the UI.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] "Maximize Metal" edict causes AI to dismantle non-essential (or essential) metal-containing structures.
- [ ] "Eradicate Plague" edict causes AI to vent atmosphere.

## 7. Technical Guidance
- The AI manager should probably have a cooldown or tick rate so it doesn't destroy everything in one frame.

## 8. Questions
*Builder: add questions here if spec is unclear.*
