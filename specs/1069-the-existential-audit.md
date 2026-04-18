# 1069: The Existential Audit

## 1. Overview
The Existential Audit introduces a mechanism where a super-advanced precursor AI occasionally evaluates the "meaningfulness" of a colony. High material efficiency without corresponding cultural/philosophical output results in a global `Existential Crisis` debuff, forcing players to balance industrial optimization with artistic and cultural endeavors.

## 2. Dependencies
- Core tick/time system (`SimulationTime`).
- Pop components (specifically `UtilityAI` and `Needs`).
- Architecture/buildings (to evaluate cultural vs. industrial output).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_audit_failure_triggers_existential_crisis() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<SimulationTime>();
        app.insert_resource(PrecursorAI { next_audit_tick: 100 });

        let pop_entity = app.world_mut().spawn((Pop::new(), UtilityAI::default())).id();

        // Add highly industrial, low cultural score building
        app.world_mut().spawn(IndustrialBuilding { efficiency: 100.0, cultural_value: 0.0 });

        app.add_systems(Update, existential_audit_system);

        // Act: Advance time to trigger audit
        let mut time = app.world_mut().resource_mut::<SimulationTime>();
        time.tick = 100;
        app.update();

        // Assert
        let pop = app.world().entity(pop_entity);
        assert!(pop.contains::<ExistentialCrisis>(), "Pop should have an Existential Crisis due to failed audit");
    }

    #[test]
    fn test_existential_crisis_halts_work() {
        // Arrange
        let mut app = App::new();

        // Pop has ExistentialCrisis modifier
        let pop_entity = app.world_mut().spawn((
            Pop::new(),
            UtilityAI::default(),
            ExistentialCrisis { severity: 1.0, duration: 100 }
        )).id();

        app.add_systems(Update, evaluate_actions_system);

        // Act
        app.update();

        // Assert
        let ai = app.world().get::<UtilityAI>(pop_entity).unwrap();
        // The weight for standard work should be drastically reduced or overridden
        assert!(ai.get_weight(ActionType::Work) < 0.1, "Work utility weight should be very low during an Existential Crisis");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct PrecursorAI {
    pub next_audit_tick: u64,
}

#[derive(Component)]
pub struct IndustrialBuilding {
    pub efficiency: f32,
    pub cultural_value: f32,
}

#[derive(Component)]
pub struct ExistentialCrisis {
    pub severity: f32,
    pub duration: u64,
}

// Minimal implementation to turn RED -> GREEN
pub fn existential_audit_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    mut ai: ResMut<PrecursorAI>,
    pops: Query<Entity, With<Pop>>,
    buildings: Query<&IndustrialBuilding>,
) {
    if time.tick >= ai.next_audit_tick {
        let mut total_efficiency = 0.0;
        let mut total_culture = 0.0;

        for b in buildings.iter() {
            total_efficiency += b.efficiency;
            total_culture += b.cultural_value;
        }

        // Failed audit if efficiency vastly outweighs culture
        if total_efficiency > total_culture * 10.0 {
            for pop_entity in pops.iter() {
                commands.entity(pop_entity).insert(ExistentialCrisis {
                    severity: 1.0,
                    duration: 500,
                });
            }
        }

        // Schedule next audit
        ai.next_audit_tick = time.tick + 10000;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Scoring Logic**: The current simple sum of `efficiency` vs `cultural_value` is naive. A true audit should evaluate the ratio of `Meaning` needs satisfied across the colony instead of just reading building stats.
- **Crisis Decay**: The `ExistentialCrisis` should decay over time rather than abruptly disappearing.
- **Notification**: Add narrative popups (`ChronicleEvent`) when an audit occurs, succeeds, or fails, so the player understands why their economy suddenly halted.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Pops correctly stop working when experiencing an `ExistentialCrisis`.

## 7. Technical Guidance
- Integrate with `evaluate_actions_system` to ensure `ActionType::Philosophize` or similar receives a massive weight boost during a crisis, while `ActionType::Work` plummets.
- Ensure the Precursor AI's timing isn't totally predictable; add some randomness to `next_audit_tick`.

## 8. Questions
*Builder: Add questions here if the specification is unclear about how "Cultural Value" is produced or calculated.*
- *Architect:* Cultural Value is produced by Pops with the `Artist` job role when they successfully complete a `CreateArt` task. It is a simple integer component accumulating globally.
