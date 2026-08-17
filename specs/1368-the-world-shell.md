# 1368: The World-Shell

## 1. Overview
**Layer:** Cross-layer

**Fantasy:** Turning the planet into a turtle.

**Mechanic:** Endgame project. Building a physical shield grid around the entire planet. Blocks ALL orbital bombardment, drop pods, and solar radiation. But also blocks ALL solar power and trade (ships can't land). Must be toggled off to trade.

**Emergence:** You turtle up to survive a fleet bombardment. You survive, but you forgot to turn the shield off for the food shipment. The freighters turn around, and you starve inside your indestructible shell.

**Tension:** Ultimate Defense vs. Isolation.

## 2. Dependencies
- Base ECS system
- Orbital bombardment system
- Solar power system
- Trade/ship landing system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            block_bombardment_system,
            block_solar_power_system,
            block_trade_system,
        ));
        app
    }

    #[test]
    fn test_world_shell_blocks_bombardment() {
        let mut app = setup_app();

        // Setup world shell
        app.world_mut().insert_resource(WorldShell { is_active: true });

        // Spawn a bombardment event
        let event = app.world_mut().spawn(BombardmentEvent { damage: 100.0, blocked: false }).id();

        app.update();

        // Bombardment should be blocked
        let event_data = app.world().get::<BombardmentEvent>(event).unwrap();
        assert!(event_data.blocked);
    }

    #[test]
    fn test_world_shell_blocks_solar_power() {
        let mut app = setup_app();

        // Setup world shell
        app.world_mut().insert_resource(WorldShell { is_active: true });

        // Spawn a solar panel
        let panel = app.world_mut().spawn(SolarPanel { base_output: 50.0, current_output: 50.0 }).id();

        app.update();

        // Solar output should be zero
        let panel_data = app.world().get::<SolarPanel>(panel).unwrap();
        assert_eq!(panel_data.current_output, 0.0);
    }

    #[test]
    fn test_world_shell_blocks_trade_ships() {
        let mut app = setup_app();

        // Setup world shell
        app.world_mut().insert_resource(WorldShell { is_active: true });

        // Spawn a trade ship attempting to land
        let ship = app.world_mut().spawn(TradeShip { is_landing: true, landing_blocked: false }).id();

        app.update();

        // Landing should be blocked
        let ship_data = app.world().get::<TradeShip>(ship).unwrap();
        assert!(ship_data.landing_blocked);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct WorldShell {
    pub is_active: bool,
}

#[derive(Component)]
pub struct BombardmentEvent {
    pub damage: f32,
    pub blocked: bool,
}

#[derive(Component)]
pub struct SolarPanel {
    pub base_output: f32,
    pub current_output: f32,
}

#[derive(Component)]
pub struct TradeShip {
    pub is_landing: bool,
    pub landing_blocked: bool,
}

pub fn block_bombardment_system(
    shell: Option<Res<WorldShell>>,
    mut events: Query<&mut BombardmentEvent>,
) {
    if let Some(shell) = shell {
        if shell.is_active {
            for mut event in events.iter_mut() {
                event.blocked = true;
            }
        }
    }
}

pub fn block_solar_power_system(
    shell: Option<Res<WorldShell>>,
    mut panels: Query<&mut SolarPanel>,
) {
    if let Some(shell) = shell {
        if shell.is_active {
            for mut panel in panels.iter_mut() {
                panel.current_output = 0.0;
            }
        } else {
             for mut panel in panels.iter_mut() {
                panel.current_output = panel.base_output;
            }
        }
    }
}

pub fn block_trade_system(
    shell: Option<Res<WorldShell>>,
    mut ships: Query<&mut TradeShip>,
) {
    if let Some(shell) = shell {
        if shell.is_active {
            for mut ship in ships.iter_mut() {
                if ship.is_landing {
                    ship.landing_blocked = true;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an interactive UI button to toggle the `WorldShell.is_active` state.
- Add an energy upkeep cost to maintain the active shield. If energy runs out, the shield toggles off automatically.
- Introduce an event `WorldShellToggledEvent` so UI and other systems can react efficiently.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] When `WorldShell` is active, `BombardmentEvent` entities are marked as blocked.
- [ ] When `WorldShell` is active, `SolarPanel` current output is 0.
- [ ] When `WorldShell` is active, `TradeShip` entities attempting to land are marked as blocked.

## 7. Technical Guidance
- Ensure systems executing bombardment damage check the `blocked` flag before applying damage.
- Integrate with the existing `ColonyResources` to handle the energy upkeep cost.

## 8. Questions
*Builder: add questions here if spec is unclear.*
