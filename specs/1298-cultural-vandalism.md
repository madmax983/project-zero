# Specification: Cultural Vandalism

## 1. Overview
The streets speak back to the palace. When Unrest is high, Pops may deface "Official" structures like statues, banners, or propaganda screens. Defaced items invert their buffs, turning symbols of loyalty into focal points for rebellion.

## 2. Dependencies
- Building systems (Layer 1)
- Pop Unrest and Action systems
- Moral/Buff aura systems for structures

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Mock components for test phase
    #[derive(Component, Clone)]
    pub struct MoraleAura {
        pub effect: f32,
    }

    #[derive(Component)]
    pub struct OfficialStructure;

    #[derive(Component)]
    pub struct Defaced;

    #[derive(Component)]
    pub struct Unrest {
        pub value: f32,
    }

    #[derive(Component)]
    pub struct PopAction {
        pub target: Option<Entity>,
        pub action_type: ActionType,
    }

    #[derive(PartialEq)]
    pub enum ActionType {
        Idle,
        Vandalize,
    }

    fn evaluate_vandalism_targets(
        q_structures: Query<Entity, (With<OfficialStructure>, Without<Defaced>)>,
        mut q_pops: Query<(&Unrest, &mut PopAction)>,
    ) {
        // Implementation will go here
    }

    fn process_vandalism(
        mut commands: Commands,
        mut q_pops: Query<&mut PopAction>,
        mut q_structures: Query<(Entity, &mut MoraleAura), With<OfficialStructure>>,
    ) {
        // Implementation will go here
    }

    #[test]
    fn test_high_unrest_triggers_vandalism_action() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_vandalism_targets);

        let structure = app.world_mut().spawn(OfficialStructure).id();
        let pop = app.world_mut().spawn((
            Unrest { value: 80.0 },
            PopAction { target: None, action_type: ActionType::Idle }
        )).id();

        app.update();

        let action = app.world().get::<PopAction>(pop).unwrap();
        assert_eq!(action.action_type, ActionType::Vandalize);
        assert_eq!(action.target, Some(structure));
    }

    #[test]
    fn test_vandalism_inverts_morale_aura() {
        let mut app = App::new();
        app.add_systems(Update, process_vandalism);

        let structure = app.world_mut().spawn((
            OfficialStructure,
            MoraleAura { effect: 10.0 },
        )).id();

        app.world_mut().spawn(PopAction {
            target: Some(structure),
            action_type: ActionType::Vandalize,
        });

        app.update();

        let aura = app.world().get::<MoraleAura>(structure).unwrap();
        assert!(aura.effect < 0.0, "Morale effect should be inverted after vandalism");
        assert!(app.world().get::<Defaced>(structure).is_some(), "Structure should be marked as defaced");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The minimal code required to pass the tests

fn evaluate_vandalism_targets(
    q_structures: Query<Entity, (With<OfficialStructure>, Without<Defaced>)>,
    mut q_pops: Query<(&Unrest, &mut PopAction)>,
) {
    for (unrest, mut action) in q_pops.iter_mut() {
        if unrest.value > 50.0 && action.action_type == ActionType::Idle {
            if let Some(target) = q_structures.iter().next() {
                action.action_type = ActionType::Vandalize;
                action.target = Some(target);
            }
        }
    }
}

fn process_vandalism(
    mut commands: Commands,
    mut q_pops: Query<&mut PopAction>,
    mut q_structures: Query<(Entity, &mut MoraleAura), With<OfficialStructure>>,
) {
    for mut action in q_pops.iter_mut() {
        if action.action_type == ActionType::Vandalize {
            if let Some(target_entity) = action.target {
                if let Ok((entity, mut aura)) = q_structures.get_mut(target_entity) {
                    aura.effect = -aura.effect;
                    commands.entity(entity).insert(Defaced);
                }
            }
            action.action_type = ActionType::Idle;
            action.target = None;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate the `Vandalize` action into the actual Pop Utility AI instead of direct hardcoded transitions.
- Hook into the notification or chronicle system when an official structure is defaced.
- Ensure visual changes occur by swapping sprites or enabling a graffiti visual component when `Defaced` is inserted.
- Implement a repair/cleanup action so loyal Pops or automated drones can restore the building and its aura.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Official structures can be targeted and defaced by Pops with high Unrest.
- [ ] Defaced structures invert their morale buffs.

## 7. Technical Guidance
- `OfficialStructure` and `Defaced` should be marker components.
- Make sure that inverted auras propagate properly into the global unrest/morale calculations for the layer.
- Ensure that the action type fits within the project's enum definitions for `ActionType`, expanding it as necessary.

## 8. Questions
*Builder: add questions here if spec is unclear.*
