use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Stress {
    pub level: f32,
}

#[derive(Component)]
pub struct Colony;

#[derive(Component, Default)]
pub struct ColonyStress {
    pub average_level: f32,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum GridState {
    Normal,
    Anxious,
    Eager,
    Lockdown,
}

#[derive(Component)]
pub struct SmartGrid {
    pub state: GridState,
}

#[derive(Component)]
pub struct ResidentOf(pub Entity);

#[derive(Component)]
pub struct Machine {
    pub efficiency: f32,
    pub burnout_risk: f32,
}

// Optimization: Use `ResidentOf` to partition stress per `Colony` if multiple colonies are supported in the same simulation map
pub fn calculate_colony_stress_system(
    pops: Query<(&Stress, Option<&ResidentOf>), With<Pop>>,
    mut colonies: Query<(Entity, &mut ColonyStress), With<Colony>>,
) {
    for (colony_entity, mut colony_stress) in colonies.iter_mut() {
        let mut total_stress = 0.0;
        let mut count = 0;

        for (stress, resident) in pops.iter() {
            // Include pop if they are a resident of this colony, or if they don't have a specific residency
            let belongs_to_colony = resident.is_none_or(|r| r.0 == colony_entity);
            if belongs_to_colony {
                total_stress += stress.level;
                count += 1;
            }
        }

        colony_stress.average_level = if count > 0 {
            total_stress / count as f32
        } else {
            0.0
        };
    }
}

pub fn apply_subconscious_grid_effects_system(
    mut colonies: Query<(&ColonyStress, &mut SmartGrid), With<Colony>>,
) {
    for (stress, mut grid) in colonies.iter_mut() {
        if stress.average_level > 95.0 {
            grid.state = GridState::Lockdown;
        } else if stress.average_level >= 80.0 {
            grid.state = GridState::Anxious;
        } else if stress.average_level < 20.0 {
            grid.state = GridState::Eager;
        } else {
            grid.state = GridState::Normal;
        }
    }
}

#[derive(Component)]
pub struct SubconsciousGridEffect {
    pub efficiency_multiplier: f32,
    pub burnout_risk_modifier: f32,
}

impl Default for SubconsciousGridEffect {
    fn default() -> Self {
        Self {
            efficiency_multiplier: 1.0,
            burnout_risk_modifier: 0.0,
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_machine_efficiency_system(
    colonies: Query<&SmartGrid, With<Colony>>,
    mut machines: Query<
        (
            Entity,
            Option<&ResidentOf>,
            Option<&mut SubconsciousGridEffect>,
        ),
        With<Machine>,
    >,
    mut commands: Commands,
) {
    for (entity, resident, effect_opt) in machines.iter_mut() {
        // Assume single colony setup for machines without ResidentOf
        let grid_state_opt = if let Some(res) = resident {
            colonies.get(res.0).map(|g| g.state).ok()
        } else {
            colonies.get_single().map(|g| g.state).ok()
        };

        if let Some(grid_state) = grid_state_opt {
            let (mult, risk) = match grid_state {
                GridState::Eager => (1.2, 0.04),
                GridState::Anxious => (0.8, 0.0),
                GridState::Lockdown => (0.0, 0.0),
                GridState::Normal => (1.0, 0.0),
            };

            if let Some(mut effect) = effect_opt {
                effect.efficiency_multiplier = mult;
                effect.burnout_risk_modifier = risk;
            } else {
                commands.entity(entity).insert(SubconsciousGridEffect {
                    efficiency_multiplier: mult,
                    burnout_risk_modifier: risk,
                });
            }
        }
    }
}

pub fn apply_lockdown_system(
    colonies: Query<&SmartGrid, With<Colony>>,
    mut doors: Query<(
        &mut crate::layer1::access_control::AccessControl,
        Option<&ResidentOf>,
    )>,
) {
    for (mut door, resident_opt) in doors.iter_mut() {
        if let Some(resident) = resident_opt {
            if let Ok(grid) = colonies.get(resident.0) {
                if grid.state == GridState::Lockdown
                    && door.mode != crate::layer1::access_control::AccessMode::Lockdown
                {
                    door.mode = crate::layer1::access_control::AccessMode::Lockdown;
                }
            }
        } else {
            // Assume single colony setup for doors without ResidentOf
            if let Ok(grid) = colonies.get_single() {
                if grid.state == GridState::Lockdown
                    && door.mode != crate::layer1::access_control::AccessMode::Lockdown
                {
                    door.mode = crate::layer1::access_control::AccessMode::Lockdown;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase Test Setup
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(
            Update,
            (
                calculate_colony_stress_system,
                apply_subconscious_grid_effects_system,
                update_machine_efficiency_system,
            )
                .chain(),
        );
        app
    }

    #[test]
    fn test_high_stress_causes_grid_anxiety() {
        let mut app = setup_app();

        // Spawn stressed pops
        app.world_mut().spawn((Pop, Stress { level: 90.0 }));
        app.world_mut().spawn((Pop, Stress { level: 95.0 }));

        let colony = app
            .world_mut()
            .spawn((
                Colony,
                ColonyStress { average_level: 0.0 },
                SmartGrid {
                    state: GridState::Normal,
                },
            ))
            .id();

        app.update();

        let grid = app.world().get::<SmartGrid>(colony).unwrap();
        assert_eq!(
            grid.state,
            GridState::Anxious,
            "High stress should make the grid anxious"
        );
    }

    #[test]
    fn test_low_stress_causes_grid_eagerness() {
        let mut app = setup_app();

        // Spawn relaxed pops
        app.world_mut().spawn((Pop, Stress { level: 10.0 }));
        app.world_mut().spawn((Pop, Stress { level: 5.0 }));

        let colony = app
            .world_mut()
            .spawn((
                Colony,
                ColonyStress { average_level: 0.0 },
                SmartGrid {
                    state: GridState::Normal,
                },
            ))
            .id();

        app.update();

        let grid = app.world().get::<SmartGrid>(colony).unwrap();
        assert_eq!(
            grid.state,
            GridState::Eager,
            "Low stress should make the grid eager"
        );
    }

    #[test]
    fn test_grid_state_affects_machines() {
        let mut app = setup_app();

        let colony = app
            .world_mut()
            .spawn((
                Colony,
                ColonyStress { average_level: 0.0 },
                SmartGrid {
                    state: GridState::Eager,
                },
            ))
            .id();

        let machine = app
            .world_mut()
            .spawn((
                Machine {
                    efficiency: 1.0,
                    burnout_risk: 0.01,
                },
                ResidentOf(colony),
            ))
            .id();

        app.update();

        // Check if component exists
        let effect = app.world().get::<SubconsciousGridEffect>(machine).unwrap();
        assert!(
            effect.efficiency_multiplier > 1.0,
            "Eager grid should over-clock machines"
        );
        assert!(
            effect.burnout_risk_modifier > 0.0,
            "Eager grid should increase burnout risk"
        );
    }

    #[test]
    fn test_apply_lockdown_system() {
        use crate::layer1::access_control::{AccessControl, AccessMode};

        let mut app = setup_app();
        app.add_systems(Update, apply_lockdown_system);

        let colony = app
            .world_mut()
            .spawn((
                Colony,
                ColonyStress { average_level: 0.0 },
                SmartGrid {
                    state: GridState::Lockdown,
                },
            ))
            .id();

        let door = app
            .world_mut()
            .spawn((
                AccessControl {
                    mode: AccessMode::Public,
                    allowed_pops: std::collections::HashSet::new(),
                    allowed_roles: std::collections::HashSet::new(),
                },
                ResidentOf(colony),
            ))
            .id();

        app.update();

        let door_comp = app.world().get::<AccessControl>(door).unwrap();
        assert_eq!(
            door_comp.mode,
            AccessMode::Lockdown,
            "Door should be locked down when grid is in lockdown"
        );
    }
}

#[cfg(test)]
mod normal_state_tests {
    use super::*;

    #[test]
    fn test_apply_subconscious_grid_effects_system_normal_state() {
        let mut app = App::new();
        app.add_systems(
            Update,
            (
                calculate_colony_stress_system,
                apply_subconscious_grid_effects_system,
                update_machine_efficiency_system,
            )
                .chain(),
        );

        // Spawn average stress pops
        app.world_mut().spawn((Pop, Stress { level: 50.0 }));

        let colony = app
            .world_mut()
            .spawn((
                Colony,
                ColonyStress { average_level: 0.0 },
                SmartGrid {
                    state: GridState::Anxious, // start it out anxious
                },
            ))
            .id();

        app.update();

        let grid = app.world().get::<SmartGrid>(colony).unwrap();
        assert_eq!(
            grid.state,
            GridState::Normal,
            "Average stress should make the grid normal"
        );
    }
}
