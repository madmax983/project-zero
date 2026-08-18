use crate::layer1::economy::resources::ColonyResources;
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

pub fn block_trade_system(shell: Option<Res<WorldShell>>, mut ships: Query<&mut TradeShip>) {
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

pub fn maintain_world_shell_system(
    shell: Option<ResMut<WorldShell>>,
    resources: Option<ResMut<ColonyResources>>,
) {
    let upkeep_cost = 50.0; // Use fuel or credits as proxy for energy since we don't have it

    if let (Some(mut shell), Some(mut resources)) = (shell, resources) {
        if shell.is_active {
            if resources.fuel >= upkeep_cost {
                resources.fuel -= upkeep_cost;
            } else {
                shell.is_active = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(
            Update,
            (
                block_bombardment_system,
                block_solar_power_system,
                block_trade_system,
                maintain_world_shell_system,
            ),
        );
        app
    }

    #[test]
    fn test_world_shell_blocks_bombardment() {
        let mut app = setup_app();

        // Setup world shell
        app.world_mut()
            .insert_resource(WorldShell { is_active: true });

        // Spawn a bombardment event
        let event = app
            .world_mut()
            .spawn(BombardmentEvent {
                damage: 100.0,
                blocked: false,
            })
            .id();

        app.update();

        // Bombardment should be blocked
        let event_data = app.world().get::<BombardmentEvent>(event).unwrap();
        assert!(event_data.blocked);
    }

    #[test]
    fn test_world_shell_blocks_solar_power() {
        let mut app = setup_app();

        // Setup world shell
        app.world_mut()
            .insert_resource(WorldShell { is_active: true });

        // Spawn a solar panel
        let panel = app
            .world_mut()
            .spawn(SolarPanel {
                base_output: 50.0,
                current_output: 50.0,
            })
            .id();

        app.update();

        // Solar output should be zero
        let panel_data = app.world().get::<SolarPanel>(panel).unwrap();
        assert_eq!(panel_data.current_output, 0.0);
    }

    #[test]
    fn test_world_shell_blocks_trade_ships() {
        let mut app = setup_app();

        // Setup world shell
        app.world_mut()
            .insert_resource(WorldShell { is_active: true });

        // Spawn a trade ship attempting to land
        let ship = app
            .world_mut()
            .spawn(TradeShip {
                is_landing: true,
                landing_blocked: false,
            })
            .id();

        app.update();

        // Landing should be blocked
        let ship_data = app.world().get::<TradeShip>(ship).unwrap();
        assert!(ship_data.landing_blocked);
    }

    #[test]
    fn test_world_shell_inactive_does_not_block_bombardment() {
        let mut app = setup_app();

        // Setup world shell (inactive)
        app.world_mut()
            .insert_resource(WorldShell { is_active: false });

        // Spawn a bombardment event
        let event = app
            .world_mut()
            .spawn(BombardmentEvent {
                damage: 100.0,
                blocked: false,
            })
            .id();

        app.update();

        // Bombardment should not be blocked
        let event_data = app.world().get::<BombardmentEvent>(event).unwrap();
        assert!(!event_data.blocked);
    }

    #[test]
    fn test_world_shell_inactive_restores_solar_power() {
        let mut app = setup_app();

        // Setup world shell (inactive)
        app.world_mut()
            .insert_resource(WorldShell { is_active: false });

        // Spawn a solar panel
        let panel = app
            .world_mut()
            .spawn(SolarPanel {
                base_output: 50.0,
                current_output: 0.0,
            })
            .id();

        app.update();

        // Solar output should be restored
        let panel_data = app.world().get::<SolarPanel>(panel).unwrap();
        assert_eq!(panel_data.current_output, 50.0);
    }

    #[test]
    fn test_world_shell_inactive_does_not_block_trade_ships() {
        let mut app = setup_app();

        // Setup world shell (inactive)
        app.world_mut()
            .insert_resource(WorldShell { is_active: false });

        // Spawn a trade ship attempting to land
        let ship = app
            .world_mut()
            .spawn(TradeShip {
                is_landing: true,
                landing_blocked: false,
            })
            .id();

        app.update();

        // Landing should not be blocked
        let ship_data = app.world().get::<TradeShip>(ship).unwrap();
        assert!(!ship_data.landing_blocked);
    }

    #[test]
    fn test_world_shell_missing_does_not_block_trade_ships() {
        let mut app = setup_app();

        // No world shell resource

        // Spawn a trade ship attempting to land
        let ship = app
            .world_mut()
            .spawn(TradeShip {
                is_landing: true,
                landing_blocked: false,
            })
            .id();

        app.update();

        // Landing should not be blocked
        let ship_data = app.world().get::<TradeShip>(ship).unwrap();
        assert!(!ship_data.landing_blocked);
    }

    #[test]
    fn test_world_shell_upkeep_deducts_fuel() {
        let mut app = setup_app();

        app.world_mut()
            .insert_resource(WorldShell { is_active: true });

        let res = ColonyResources {
            fuel: 100.0,
            ..Default::default()
        };
        app.world_mut().insert_resource(res);

        app.update();

        let updated_res = app.world().resource::<ColonyResources>();
        assert_eq!(updated_res.fuel, 50.0);

        let shell = app.world().resource::<WorldShell>();
        assert!(shell.is_active);
    }

    #[test]
    fn test_world_shell_upkeep_fails_without_fuel() {
        let mut app = setup_app();

        app.world_mut()
            .insert_resource(WorldShell { is_active: true });

        let res = ColonyResources {
            fuel: 10.0,
            ..Default::default()
        };
        app.world_mut().insert_resource(res);

        app.update();

        let updated_res = app.world().resource::<ColonyResources>();
        assert_eq!(updated_res.fuel, 10.0); // Fuel should not be deducted if insufficient

        let shell = app.world().resource::<WorldShell>();
        assert!(!shell.is_active); // Shell should be deactivated
    }
}
