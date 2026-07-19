use crate::layer1::architecture::building::Building;
use crate::layer1::core::map::GridPosition;
use crate::layer1::entities::pop::Pop;
use crate::layer1::social::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Subversive {
    pub dissent: f32,
}

#[derive(Component)]
pub struct Surveillance {
    pub radius: i32,
}

pub fn run_subversion_spread_system(
    mut query: Query<(&mut Subversive, &GridPosition)>,
    surveillance_query: Query<(&Surveillance, &GridPosition)>,
) {
    let mut changes = Vec::new();

    for (subversive, pos) in query.iter() {
        if subversive.dissent > 0.0 {
            changes.push((pos.x, pos.y, subversive.dissent));
        }
    }

    for (x, y, dissent) in changes {
        let mut blocked = false;
        for (surveillance, s_pos) in surveillance_query.iter() {
            let dist = s_pos.distance_chebyshev(GridPosition { x, y });
            if dist <= surveillance.radius as u32 {
                blocked = true;
                break;
            }
        }

        if !blocked {
            for (mut target_sub, t_pos) in query.iter_mut() {
                if t_pos.x == x && t_pos.y == y {
                    continue;
                }
                let dist = t_pos.distance_chebyshev(GridPosition { x, y });
                if dist <= 1 {
                    target_sub.dissent += dissent * 0.1;
                }
            }
        }
    }
}

pub fn run_surveillance_morale_system(
    mut pop_query: Query<(&mut Morale, &GridPosition), With<Pop>>,
    surveillance_query: Query<(&Surveillance, &GridPosition), With<Building>>,
) {
    for (surveillance, s_pos) in surveillance_query.iter() {
        for (mut morale, p_pos) in pop_query.iter_mut() {
            let dist = s_pos.distance_chebyshev(GridPosition { x: p_pos.x, y: p_pos.y });
            if dist <= surveillance.radius as u32 {
                morale.add_modifier(MoodModifier {
                    label: "Under Surveillance".to_string(),
                    value: -0.1,
                    duration: 1, // Only 1 tick, will be reapplied constantly while under surveillance
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::building::BuildingType;
    use bevy::app::App;
    use bevy::MinimalPlugins;

    #[test]
    fn test_subversive_trait_spreads_to_nearby_pops() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(bevy::app::Update, run_subversion_spread_system);

        let _sub = app.world_mut().spawn((
            Pop,
            GridPosition { x: 10, y: 10 },
            Subversive { dissent: 10.0 },
        )).id();
        let innocent = app.world_mut().spawn((
            Pop,
            GridPosition { x: 10, y: 11 },
            Subversive { dissent: 0.0 },
        )).id();

        app.update();

        let innocent_dissent = app.world().get::<Subversive>(innocent).unwrap().dissent;
        assert!(innocent_dissent > 0.0, "Dissent should spread to nearby pops");
    }

    #[test]
    fn test_surveillance_building_reduces_subversion_spread() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(bevy::app::Update, run_subversion_spread_system);

        let _sub = app.world_mut().spawn((
            Pop,
            GridPosition { x: 10, y: 10 },
            Subversive { dissent: 10.0 },
        )).id();
        let innocent = app.world_mut().spawn((
            Pop,
            GridPosition { x: 10, y: 11 },
            Subversive { dissent: 0.0 },
        )).id();

        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Tower,
            },
            Surveillance { radius: 5 },
            GridPosition { x: 10, y: 10 },
        ));

        app.update();

        let innocent_dissent = app.world().get::<Subversive>(innocent).unwrap().dissent;
        assert_eq!(innocent_dissent, 0.0, "Surveillance should prevent dissent spread");
    }

    #[test]
    fn test_surveillance_building_lowers_pop_morale() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(bevy::app::Update, run_surveillance_morale_system);

        let pop = app.world_mut().spawn((
            Pop,
            GridPosition { x: 10, y: 10 },
            Morale::default(),
        )).id();

        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Tower,
            },
            Surveillance { radius: 5 },
            GridPosition { x: 10, y: 10 },
        ));

        app.update();

        let pop_morale = app.world().get::<Morale>(pop).unwrap();
        assert!(pop_morale.modifiers.iter().any(|m| m.value < 0.0), "Surveillance should lower morale");
    }
}
