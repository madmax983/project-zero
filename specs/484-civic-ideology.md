# Civic Ideology

## 1. Overview
What holds the colony together? This specification introduces the "Civic Ideology" system for Layer 1. The player can select a core "Colony Goal" (e.g., "Survival", "Profit", "Knowledge"). Actions and construction aligned with the ideology provide morale bonuses, while opposed actions provide morale penalties. This creates a tension between short-term flexibility (building what you need right now) and long-term unity (adhering to the colony's founding principles).

## 2. Dependencies
- `005-pop-needs` (Morale system)
- `010-chronicle-system` (Logging events)
- `016-utility-ai-system` (For pop decision making and morale impacts)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::{Needs, MoraleModifier};

    #[test]
    fn test_civic_ideology_aligned_action_grants_morale() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.init_resource::<ActiveIdeology>();
        app.add_systems(Update, apply_ideological_modifiers_system);

        app.world_mut().insert_resource(ActiveIdeology(IdeologyType::Survivalist));

        let pop = app.world_mut().spawn((
            Needs { morale: 50.0, ..default() },
            RecentAction { action_type: ActionType::HoardFood, duration: 10.0 }
        )).id();

        // Act
        app.update();

        // Assert
        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(needs.morale > 50.0, "Morale should increase when performing aligned actions");
    }

    #[test]
    fn test_civic_ideology_opposed_action_reduces_morale() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.init_resource::<ActiveIdeology>();
        app.add_systems(Update, apply_ideological_modifiers_system);

        app.world_mut().insert_resource(ActiveIdeology(IdeologyType::Survivalist));

        let pop = app.world_mut().spawn((
            Needs { morale: 50.0, ..default() },
            RecentAction { action_type: ActionType::BuildStatue, duration: 10.0 }
        )).id();

        // Act
        app.update();

        // Assert
        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(needs.morale < 50.0, "Morale should decrease when performing opposed actions");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::needs::Needs;

#[derive(Resource, Default)]
pub struct ActiveIdeology(pub IdeologyType);

#[derive(Default, PartialEq, Eq)]
pub enum IdeologyType {
    #[default]
    None,
    Survivalist,
    Profit,
    Knowledge,
}

#[derive(Component)]
pub struct RecentAction {
    pub action_type: ActionType,
    pub duration: f32,
}

#[derive(PartialEq, Eq)]
pub enum ActionType {
    HoardFood,
    BuildStatue,
    Research,
    Trade,
}

pub fn apply_ideological_modifiers_system(
    ideology: Option<Res<ActiveIdeology>>,
    mut query: Query<(&mut Needs, &RecentAction)>,
) {
    let Some(active_ideology) = ideology else { return };
    if active_ideology.0 == IdeologyType::None { return; }

    for (mut needs, action) in query.iter_mut() {
        match active_ideology.0 {
            IdeologyType::Survivalist => {
                if action.action_type == ActionType::HoardFood {
                    needs.morale += 5.0; // Bonus for aligned action
                } else if action.action_type == ActionType::BuildStatue {
                    needs.morale -= 5.0; // Penalty for waste
                }
            }
            IdeologyType::Profit => {
                // Implement profit modifiers
            }
            IdeologyType::Knowledge => {
                // Implement knowledge modifiers
            }
            IdeologyType::None => {}
        }

        // Clamp morale
        needs.morale = needs.morale.clamp(0.0, 100.0);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Data-Driven Ideologies**: Instead of hardcoding `IdeologyType` and its reactions in a massive `match` statement, consider moving to a data-driven approach where an `Ideology` resource holds a map of `ActionType` to `MoraleModifier`.
- **Decay/Duration**: The `RecentAction` component is currently basic. We should use a timer or hook into the existing event system so pops remember their actions and experience the morale effects fading over time.
- **Tension Management**: Ensure the morale penalties for violating ideology are significant enough to make the player think twice, but not so punishing that they cause a death spiral immediately.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Aligned actions grant a morale bonus.
- [ ] Opposed actions apply a morale penalty.

## 7. Technical Guidance
- Integrate with `Layer1SystemSet::Observation` to evaluate actions and apply morale modifiers periodically, rather than every frame.
- Add `ActiveIdeology` to the core resource initialization in `setup_world()`.
- Expand `ActionType` to cover actual in-game actions like `Construct(BuildingType::Statue)` rather than mock enums.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
