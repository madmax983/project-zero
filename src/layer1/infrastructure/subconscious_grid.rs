use bevy_ecs::prelude::*;
use crate::layer1::stress::StressTracker;
use crate::layer1::pop::Pop;
use crate::layer1::tech::symbiotic_habitation::ResidentOf;

#[derive(Component)]
pub struct GridColony;

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
pub struct GridMachine {
    pub efficiency: f32,
    pub burnout_risk: f32,
}

pub fn calculate_colony_stress_system(
    pops: Query<&StressTracker, With<Pop>>,
    mut colonies: Query<&mut ColonyStress, With<GridColony>>,
) {
    let mut total_stress = 0.0;
    let mut count = 0;

    for stress in pops.iter() {
        total_stress += stress.accumulated_stress;
        count += 1;
    }

    let avg = if count > 0 { total_stress / count as f32 } else { 0.0 };

    for mut colony_stress in colonies.iter_mut() {
        colony_stress.average_level = avg;
    }
}

pub fn apply_subconscious_grid_effects_system(
    mut colonies: Query<(&ColonyStress, &mut SmartGrid), With<GridColony>>,
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
    colonies: Query<&SmartGrid, With<GridColony>>,
    mut machines: Query<(&mut GridMachine, &ResidentOf)>,
) {
    for (mut machine, resident) in machines.iter_mut() {
        if let Ok(grid) = colonies.get(resident.building) {
            match grid.state {
                GridState::Eager => {
                    machine.efficiency = 1.2;
                    machine.burnout_risk = 0.05;
                }
                GridState::Anxious => {
                    machine.efficiency = 0.8;
                    machine.burnout_risk = 0.01;
                }
                GridState::Lockdown => {
                    machine.efficiency = 0.0;
                    machine.burnout_risk = 0.0;
                }
                GridState::Normal => {
                    machine.efficiency = 1.0;
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
        app.add_systems(Update, (
            calculate_colony_stress_system,
            apply_subconscious_grid_effects_system,
            update_machine_efficiency_system,
        ).chain());
        app
    }

    #[test]
    fn test_high_stress_causes_grid_anxiety() {
        let mut app = setup_app();

        // Spawn stressed pops
        app.world_mut().spawn((Pop, StressTracker { accumulated_stress: 85.0 }));
        app.world_mut().spawn((Pop, StressTracker { accumulated_stress: 90.0 }));

        let colony = app.world_mut().spawn((
            GridColony,
            ColonyStress { average_level: 0.0 },
            SmartGrid { state: GridState::Normal },
        )).id();

        app.update();

        let grid = app.world().get::<SmartGrid>(colony).unwrap();
        assert_eq!(grid.state, GridState::Anxious, "High stress should make the grid anxious");
    }

    #[test]
    fn test_extreme_stress_causes_grid_lockdown() {
        let mut app = setup_app();

        // Spawn stressed pops
        app.world_mut().spawn((Pop, StressTracker { accumulated_stress: 98.0 }));
        app.world_mut().spawn((Pop, StressTracker { accumulated_stress: 98.0 }));

        let colony = app.world_mut().spawn((
            GridColony,
            ColonyStress { average_level: 0.0 },
            SmartGrid { state: GridState::Normal },
        )).id();

        app.update();

        let grid = app.world().get::<SmartGrid>(colony).unwrap();
        assert_eq!(grid.state, GridState::Lockdown, "Extreme stress should make the grid lockdown");
    }

    #[test]
    fn test_low_stress_causes_grid_eagerness() {
        let mut app = setup_app();

        // Spawn relaxed pops
        app.world_mut().spawn((Pop, StressTracker { accumulated_stress: 10.0 }));
        app.world_mut().spawn((Pop, StressTracker { accumulated_stress: 5.0 }));

        let colony = app.world_mut().spawn((
            GridColony,
            ColonyStress { average_level: 0.0 },
            SmartGrid { state: GridState::Normal },
        )).id();

        app.update();

        let grid = app.world().get::<SmartGrid>(colony).unwrap();
        assert_eq!(grid.state, GridState::Eager, "Low stress should make the grid eager");
    }

    #[test]
    fn test_grid_state_affects_machines() {
        let mut app = setup_app();

        let colony = app.world_mut().spawn((
            GridColony,
            ColonyStress { average_level: 0.0 },
            SmartGrid { state: GridState::Eager },
        )).id();

        let machine = app.world_mut().spawn((
            GridMachine { efficiency: 1.0, burnout_risk: 0.01 },
            ResidentOf { building: colony },
        )).id();

        app.update();

        let updated_machine = app.world().get::<GridMachine>(machine).unwrap();
        assert!(updated_machine.efficiency > 1.0, "Eager grid should over-clock machines");
        assert!(updated_machine.burnout_risk > 0.01, "Eager grid should increase burnout risk");
    }
}
