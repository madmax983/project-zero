use crate::layer1::pop::Pop;
use bevy::prelude::*;

#[derive(Component)]
pub struct Stress {
    pub level: f32,
}

#[derive(Component)]
pub struct SubconsciousColony;

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
pub struct SubconsciousMachine {
    pub base_efficiency: f32,
    pub efficiency_multiplier: f32,
    pub burnout_risk: f32,
}

impl SubconsciousMachine {
    pub fn current_efficiency(&self) -> f32 {
        self.base_efficiency * self.efficiency_multiplier
    }
}

pub fn calculate_colony_stress_system(
    pops: Query<(&Stress, &ResidentOf), With<Pop>>,
    mut colonies: Query<(Entity, &mut ColonyStress), With<SubconsciousColony>>,
) {
    for (colony_entity, mut colony_stress) in colonies.iter_mut() {
        let mut total_stress = 0.0;
        let mut count = 0;

        for (stress, resident) in pops.iter() {
            if resident.0 == colony_entity {
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
    mut colonies: Query<(&ColonyStress, &mut SmartGrid), With<SubconsciousColony>>,
) {
    for (stress, mut grid) in colonies.iter_mut() {
        if stress.average_level > 95.0 {
            grid.state = GridState::Lockdown;
        } else if stress.average_level > 80.0 {
            grid.state = GridState::Anxious;
        } else if stress.average_level < 20.0 {
            grid.state = GridState::Eager;
        } else {
            grid.state = GridState::Normal;
        }
    }
}

pub fn update_machine_efficiency_system(
    colonies: Query<&SmartGrid, With<SubconsciousColony>>,
    mut machines: Query<(&mut SubconsciousMachine, &ResidentOf)>,
) {
    for (mut machine, resident) in machines.iter_mut() {
        if let Ok(grid) = colonies.get(resident.0) {
            match grid.state {
                GridState::Eager => {
                    machine.efficiency_multiplier = 1.2;
                    machine.burnout_risk = 0.05;
                }
                GridState::Anxious => {
                    machine.efficiency_multiplier = 0.8;
                    machine.burnout_risk = 0.01;
                }
                GridState::Lockdown => {
                    machine.efficiency_multiplier = 0.0;
                    machine.burnout_risk = 0.0;
                }
                GridState::Normal => {
                    machine.efficiency_multiplier = 1.0;
                    machine.burnout_risk = 0.01;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // RED Phase Test Setup
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(
            Update,
            (
                calculate_colony_stress_system,
                apply_subconscious_grid_effects_system,
                update_machine_efficiency_system,
            ),
        );
        app
    }

    #[test]
    fn test_high_stress_causes_grid_anxiety() {
        let mut app = setup_app();

        let colony = app
            .world_mut()
            .spawn((
                SubconsciousColony,
                ColonyStress { average_level: 0.0 },
                SmartGrid {
                    state: GridState::Normal,
                },
            ))
            .id();

        // Spawn stressed pops
        app.world_mut()
            .spawn((Pop, ResidentOf(colony), Stress { level: 90.0 }));
        app.world_mut()
            .spawn((Pop, ResidentOf(colony), Stress { level: 96.0 }));

        app.update();
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

        let colony = app
            .world_mut()
            .spawn((
                SubconsciousColony,
                ColonyStress { average_level: 0.0 },
                SmartGrid {
                    state: GridState::Normal,
                },
            ))
            .id();

        // Spawn relaxed pops
        app.world_mut()
            .spawn((Pop, ResidentOf(colony), Stress { level: 10.0 }));
        app.world_mut()
            .spawn((Pop, ResidentOf(colony), Stress { level: 5.0 }));

        app.update();
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
                SubconsciousColony,
                ColonyStress { average_level: 0.0 },
                SmartGrid {
                    state: GridState::Eager,
                },
            ))
            .id();

        let machine = app
            .world_mut()
            .spawn((
                SubconsciousMachine {
                    base_efficiency: 1.0,
                    efficiency_multiplier: 1.0,
                    burnout_risk: 0.01,
                },
                ResidentOf(colony),
            ))
            .id();

        app.update();
        app.update();

        let updated_machine = app.world().get::<SubconsciousMachine>(machine).unwrap();
        assert!(
            updated_machine.current_efficiency() > 1.0,
            "Eager grid should over-clock machines"
        );
        assert!(
            updated_machine.burnout_risk > 0.01,
            "Eager grid should increase burnout risk"
        );
    }
}
