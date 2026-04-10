# 935 - The Ghost Frequency

## 1. Overview

**Layer:** Cross-layer
**Fantasy:** A mysterious broadcast that only certain pops can hear, driving them to act strangely or unlocking hidden tech, but attracting unwanted attention.
**Mechanic:** A signal from Layer 3 is picked up by the colony. It selectively affects Pops with specific traits (e.g., 'Sensitive', 'Genius'). Affected pops gain massive research or crafting boosts but suffer escalating Stress and occasionally abandon their jobs to construct bizarre, non-functional "Antennas" out of colony resources.
**Emergence:** You rely on the affected pops to push your tech tree. However, during a critical siege, your lead engineer wanders off to build an antenna out of the shield generator's spare parts, leaving the colony defenseless.
**Tension:** Do you shut down the comms array (losing all Layer 3 intel and the tech boost) to cure the afflicted, or let the signal continue and try to manage the creeping madness?

## 2. Dependencies

- Utility AI / Action execution system (for task abandonment and Antenna building)
- Pop Needs / Stress system
- Comms / Layer 3 Event system

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct PopTraits { is_sensitive: bool }

    #[derive(Component)]
    struct GhostFrequencyAffliction { stress_buildup_rate: f32, tech_boost: f32 }

    #[derive(Component)]
    struct CommsArray { is_active: bool, receives_ghost_signal: bool }

    #[derive(Component)]
    struct CurrentAction { action_type: ActionType }

    #[derive(Clone, PartialEq, Eq)]
    enum ActionType { Working, BuildingAntenna }

    #[test]
    fn test_ghost_frequency_afflicts_sensitive_pops() {
        let mut app = App::new();
        app.add_systems(Update, ghost_frequency_exposure_system);

        app.world_mut().spawn(CommsArray { is_active: true, receives_ghost_signal: true });

        let sensitive_pop = app.world_mut().spawn(PopTraits { is_sensitive: true }).id();
        let normal_pop = app.world_mut().spawn(PopTraits { is_sensitive: false }).id();

        app.update();

        // Sensitive pop should get afflicted, normal pop should not
        assert!(app.world().get::<GhostFrequencyAffliction>(sensitive_pop).is_some());
        assert!(app.world().get::<GhostFrequencyAffliction>(normal_pop).is_none());
    }

    #[test]
    fn test_affliction_increases_stress_and_boosts_tech() {
        let mut app = App::new();
        app.add_systems(Update, process_ghost_affliction_system);

        let pop = app.world_mut().spawn((
            PopTraits { is_sensitive: true },
            GhostFrequencyAffliction { stress_buildup_rate: 0.1, tech_boost: 2.0 },
            Needs { stress: 0.0, ..Default::default() },
        )).id();

        app.update();

        // Stress should increase
        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(needs.stress > 0.0);
    }

    #[test]
    fn test_afflicted_pop_abandons_work_to_build_antenna() {
        let mut app = App::new();
        app.add_systems(Update, ghost_frequency_compulsion_system);

        let pop = app.world_mut().spawn((
            GhostFrequencyAffliction { stress_buildup_rate: 0.1, tech_boost: 2.0 },
            CurrentAction { action_type: ActionType::Working },
        )).id();

        app.update();

        // Pop should randomly be compelled to build an antenna
        let action = app.world().get::<CurrentAction>(pop).unwrap();
        assert!(action.action_type == ActionType::BuildingAntenna);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// pub fn ghost_frequency_exposure_system(...) { ... }
// pub fn process_ghost_affliction_system(...) { ... }
// pub fn ghost_frequency_compulsion_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design

- The compulsion to build antennas should be integrated into the existing `UtilityWeights` system rather than forcibly overwriting `CurrentAction` randomly. Give it a very high utility score periodically.
- Building the antenna should actually consume resources from the colony stockpile or nearby buildings.
- Consider adding a UI alert when a pop becomes afflicted.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Sensitive pops receive both a tech boost and a stress penalty when the signal is active, and occasionally attempt to build useless antennas.

## 7. Technical Guidance

- Integrate the compulsion cleanly into `src/layer1/utility_ai.rs` by creating a new `ActionType::BuildBizarreAntenna` and having the affliction significantly spike its utility score.
- Exposing pops should be based on a global event or resource (like `ActiveGhostSignal`).

## 8. Questions
*Builder: add questions here if spec is unclear.*
