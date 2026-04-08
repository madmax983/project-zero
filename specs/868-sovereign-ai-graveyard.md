# 868: The Sovereign AI Graveyard

## 1. Overview
When you upgrade or scrap planetary AI managers, their core personalities are dumped into an asteroid belt. Eventually, they network together and declare a sovereign, highly-efficient robotic micro-state that starts out-competing your trade routes. Players must decide whether to engage in a costly military strike against unarmed but economically devastating discarded hardware, taking a huge galactic reputation hit, or allow them to bankrupt their empire legally.

## 2. Dependencies
- Layer 2 / Layer 3 AI Cores and Decommissioning logic
- Trade Route economy systems
- Layer 3 Galactic Diplomacy / Reputation components

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_ai_graveyard_formation_threshold() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(SovereignAIGraveyardPlugin);

        // Spawn 4 decommissioned cores (below threshold of 5)
        for _ in 0..4 {
            app.world_mut().spawn(DecommissionedAICore { sentience_level: 5.0 });
        }

        app.update();

        // Assert: Microstate not yet formed
        let state_query = app.world_mut().query::<&SovereignAIMicrostate>().iter(app.world()).count();
        assert_eq!(state_query, 0);

        // Add 5th core
        app.world_mut().spawn(DecommissionedAICore { sentience_level: 5.0 });
        app.update();

        // Assert: Microstate formed
        let state_query = app.world_mut().query::<&SovereignAIMicrostate>().iter(app.world()).count();
        assert_eq!(state_query, 1);
    }

    #[test]
    fn test_ai_microstate_undercuts_trade() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(SovereignAIGraveyardPlugin);

        // Setup trade route
        let route = app.world_mut().spawn(TradeRoute {
            profit_margin: 100.0,
        }).id();

        // Setup microstate
        app.world_mut().spawn(SovereignAIMicrostate {
            economic_power: 50.0,
        });

        app.update();

        // Assert: Trade route profit is reduced
        let current_profit = app.world().get::<TradeRoute>(route).unwrap().profit_margin;
        assert!(current_profit < 100.0);
        assert_eq!(current_profit, 50.0); // 100 - 50 economic power
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct DecommissionedAICore {
    pub sentience_level: f32,
}

#[derive(Component)]
pub struct SovereignAIMicrostate {
    pub economic_power: f32,
}

#[derive(Component)]
pub struct TradeRoute {
    pub profit_margin: f32,
}

pub struct SovereignAIGraveyardPlugin;

impl Plugin for SovereignAIGraveyardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            check_graveyard_formation_system,
            apply_microstate_economic_pressure_system,
        ));
    }
}

fn check_graveyard_formation_system(
    mut commands: Commands,
    cores: Query<&DecommissionedAICore>,
    existing_state: Query<Entity, With<SovereignAIMicrostate>>,
) {
    if existing_state.is_empty() && cores.iter().count() >= 5 {
        // Calculate total economic power based on cores
        let total_power: f32 = cores.iter().map(|c| c.sentience_level * 2.0).sum();

        commands.spawn(SovereignAIMicrostate {
            economic_power: total_power,
        });
    }
}

fn apply_microstate_economic_pressure_system(
    microstates: Query<&SovereignAIMicrostate>,
    mut trade_routes: Query<&mut TradeRoute>,
) {
    let total_pressure: f32 = microstates.iter().map(|m| m.economic_power).sum();

    if total_pressure > 0.0 {
        for mut route in trade_routes.iter_mut() {
            route.profit_margin = (route.profit_margin - total_pressure).max(0.0);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **System Orchestration:** `check_graveyard_formation_system` should probably not run every frame. Run on a `Timer` or when a `DecommissionAICoreEvent` is fired.
- **Microstate Power:** The `economic_power` calculation should scale more dynamically and perhaps decay over time unless the AI state "buys" new resources.
- **Event Driven:** Introduce events for when the microstate forms (e.g., `MicrostateDeclarationEvent`) so the UI and Chronicle layers can announce it to the player.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the microstate logic.
- [ ] Graveyard state forms only when enough AI cores have been decommissioned.
- [ ] Trade routes dynamically lose profit margin due to the AI's efficiency.

## 7. Technical Guidance
- Ensure integration with Layer 3 `Diplomacy` so attacking the microstate correctly triggers negative reputation across the galaxy.
- Consider adding a component like `GalacticReputation` directly to the `PlayerEmpire` resource/entity.

## 8. Questions
*Builder: add questions here if spec is unclear.*
