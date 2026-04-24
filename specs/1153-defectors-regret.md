# 1153: Defector's Regret

## 1. Overview
The awkward diplomacy of realizing your utopia is someone else's dystopia. A high-level leader from a rival Layer 3 empire defects to your civilization, bringing critical technologies or intel. However, after living in your empire for a few years, their hidden "Values" metrics clash violently with your actual policies. They experience "Defector's Regret" and begin secretly broadcasting anti-government propaganda, sabotaging local infrastructure, and demanding exorbitant luxury resources to keep quiet.

## 2. Dependencies
- VIP/Leader system (`src/layer2/leaders.rs` or `src/shared/vip.rs`)
- Faction Values/Ideology system (`src/shared/ideology.rs`)
- Sabotage/Unrest events (`src/layer1/unrest.rs`)

## 3. RED Phase: Tests First

```rust
// src/layer2/defector_tests.rs
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[test]
    fn test_defector_arrival_grants_tech_boost() {
        let mut app = App::new();
        app.add_event::<DefectorArrivalEvent>();
        app.add_systems(Update, process_defector_arrival_system);

        // Setup initial tech level
        let colony = app.world_mut().spawn(TechProgress { points: 100.0 }).id();

        app.world_mut().send_event(DefectorArrivalEvent {
            leader_name: "Admiral Thrawn".to_string(),
            tech_bonus_value: 500.0,
            colony_id: colony,
        });

        app.update();

        // Check if tech points increased
        let progress = app.world().get::<TechProgress>(colony).unwrap();
        assert_eq!(progress.points, 600.0, "Defector arrival should grant an immediate tech bonus.");
    }

    #[test]
    fn test_defector_accumulates_values_clash() {
        let mut app = App::new();
        app.add_systems(Update, defector_values_clash_system);

        // Spawn a defector with incompatible values
        let defector = app.world_mut().spawn(DefectorState {
            regret_level: 0.0,
            ideology: Ideology::Egalitarian,
        }).id();

        // Spawn a local colony with conflicting values
        app.world_mut().spawn(ColonyIdeology {
            ideology: Ideology::Authoritarian,
        });

        app.update();

        // Regret level should increase due to the clash
        let state = app.world().get::<DefectorState>(defector).unwrap();
        assert!(state.regret_level > 0.0, "Regret level should increase when values clash.");
    }

    #[test]
    fn test_high_regret_triggers_sabotage() {
        let mut app = App::new();
        app.add_event::<SabotageEvent>();
        app.add_systems(Update, defector_sabotage_system);

        // Spawn a defector with max regret
        app.world_mut().spawn(DefectorState {
            regret_level: 100.0,
            ideology: Ideology::Egalitarian,
        });

        app.update();

        // Sabotage event should be triggered
        let sabotage_events = app.world().resource::<Events<SabotageEvent>>();
        assert!(sabotage_events.get_reader().len(&sabotage_events) > 0, "High regret should trigger sabotage.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The sabotage events should feed into the existing Layer 1 unrest systems, potentially destroying specific infrastructure nodes.
- **Narrative Event**: When the defector first arrives, and when they begin their sabotage, it should trigger specific narrative templates in the Chronicle.
- **Player Agency**: Allow the player to mitigate regret by providing exorbitant luxury resources (which act as a drain on the economy) via a specific "Appease Defector" diplomatic action.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer2/defector.rs`.
- [ ] Defector arrivals grant the intended immediate bonuses.
- [ ] `DefectorState` regret accumulates over time based on ideological mismatch.
- [ ] High regret triggers sabotage events that impact the colony.

## 7. Technical Guidance
- `Ideology` enum might need to be expanded if the current game state only supports basic values.
- Make the regret accumulation gradual so the player has time to react and try to manage the situation before outright sabotage begins.
- The `SabotageEvent` should have a severity tied to the `regret_level`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
