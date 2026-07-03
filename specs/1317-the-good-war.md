# The 'Good' War

## 1. Overview
War is good for business. Your economy relies on "War Profiteering". Factories get bonuses when the galaxy is at war. If peace breaks out, your economy crashes ("Recession"). You must conduct "False Flag" operations to keep the war going.

## 2. Dependencies
- Galactic Factions (Layer 3 diplomacy)
- Colony Economy/Production Modifiers
- Covert Operations / Espionage system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_war_profiteering_bonus() {
        let mut app = App::new();
        // Setup world with a factory and a war state
        app.insert_resource(GalacticState { at_war: true });
        let factory = app.world_mut().spawn((Factory { base_output: 10.0 }, WarProfiteer)).id();

        app.add_systems(Update, apply_war_profiteering_system);
        app.update();

        let factory_comp = app.world().get::<Factory>(factory).unwrap();
        // Output should have a bonus
        assert_eq!(factory_comp.current_output, 15.0);
    }

    #[test]
    fn test_peace_recession() {
        let mut app = App::new();
        // Peace breaks out
        app.insert_resource(GalacticState { at_war: false });
        let factory = app.world_mut().spawn((Factory { base_output: 10.0 }, WarProfiteer)).id();

        app.add_systems(Update, apply_war_profiteering_system);
        app.update();

        let factory_comp = app.world().get::<Factory>(factory).unwrap();
        // Output crashes due to recession
        assert_eq!(factory_comp.current_output, 5.0);
    }

    #[test]
    fn test_false_flag_operation() {
        let mut app = App::new();
        app.insert_resource(GalacticState { at_war: false });
        app.add_event::<FalseFlagEvent>();
        app.add_systems(Update, false_flag_system);

        // Trigger a false flag
        app.world_mut().send_event(FalseFlagEvent);
        app.update();

        // The galaxy should be back at war
        let state = app.world().resource::<GalacticState>();
        assert_eq!(state.at_war, true);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Factory {
    pub base_output: f32,
    pub current_output: f32,
}

#[derive(Component)]
pub struct WarProfiteer;

#[derive(Resource)]
pub struct GalacticState {
    pub at_war: bool,
}

#[derive(Event)]
pub struct FalseFlagEvent;

pub fn apply_war_profiteering_system(
    state: Res<GalacticState>,
    mut query: Query<&mut Factory, With<WarProfiteer>>,
) {
    for mut factory in query.iter_mut() {
        if state.at_war {
            factory.current_output = factory.base_output * 1.5;
        } else {
            factory.current_output = factory.base_output * 0.5;
        }
    }
}

pub fn false_flag_system(
    mut events: EventReader<FalseFlagEvent>,
    mut state: ResMut<GalacticState>,
) {
    if !events.is_empty() {
        state.at_war = true;
        events.clear();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Extrapolate the "at war" state into a more complex `DiplomaticTension` system rather than a simple boolean.
- Differentiate output bonuses by resource type (e.g. ammunition gets more boost than food).
- Add success/failure chance and costs to `FalseFlagEvent`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Factory output is modified correctly based on galactic war state

## 7. Technical Guidance
- Ensure `WarProfiteer` component can be added and removed dynamically.
- `FalseFlagEvent` should likely integrate with an espionage module.

## 8. Questions
*Builder: add questions here if spec is unclear.*
