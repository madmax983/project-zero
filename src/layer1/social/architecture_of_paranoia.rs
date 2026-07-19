use bevy_ecs::prelude::*;
use crate::layer1::core::map::GridPosition;
use crate::layer1::social::morale::{Morale, MoodModifier};

#[derive(Component)]
pub struct Subversive {
    pub dissent: f32,
}

#[derive(Component)]
pub struct Surveillance {
    pub radius: i32,
}

#[allow(clippy::type_complexity)]
pub fn run_subversion_spread_system(
    mut commands: Commands,
    mut param_set: ParamSet<(
        Query<(&GridPosition, &Subversive)>,
        Query<(Entity, &GridPosition, Option<&mut Subversive>), With<crate::layer1::pop::Pop>>,
        Query<(&GridPosition, &Surveillance)>,
    )>
) {
    let mut subversion_spread = Vec::new();

    let mut spreads = Vec::new();
    for (pos, sub) in param_set.p0().iter() {
        if sub.dissent > 0.0 {
            spreads.push((*pos, sub.dissent));
        }
    }

    for (pos, dissent) in spreads {
        let mut blocked = false;
        for (surv_pos, surv) in param_set.p2().iter() {
            if pos.distance_chebyshev(*surv_pos) <= surv.radius as u32 {
                blocked = true;
                break;
            }
        }
        if !blocked {
            subversion_spread.push((pos, dissent));
        }
    }

    let mut innocent_query = param_set.p1();
    for (spread_pos, dissent_amount) in subversion_spread {
        for (entity, pos, mut sub_opt) in innocent_query.iter_mut() {
            if pos.distance_chebyshev(spread_pos) <= 1 {
                if let Some(ref mut sub) = sub_opt {
                    if sub.dissent == 0.0 {
                        sub.dissent = dissent_amount;
                    }
                } else {
                    commands.entity(entity).insert(Subversive { dissent: dissent_amount });
                }
            }
        }
    }
}

pub fn run_surveillance_morale_system(
    mut morale_query: Query<(&GridPosition, &mut Morale)>,
    surveillance_query: Query<(&GridPosition, &Surveillance)>,
) {
    for (pos, mut morale) in morale_query.iter_mut() {
        for (surv_pos, surv) in surveillance_query.iter() {
            if pos.distance_chebyshev(*surv_pos) <= surv.radius as u32 {
                if !morale.modifiers.iter().any(|m| m.label == "Surveillance") {
                    morale.modifiers.push(MoodModifier {
                        label: "Surveillance".to_string(),
                        value: -0.1,
                        duration: 2, // slightly > 1 to ensure overlap between ticks
                    });
                }
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subversive_trait_spreads_to_nearby_pops() {
        let mut app = bevy::app::App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_systems(bevy::app::Update, run_subversion_spread_system);

        // Arrange: Two pops close to each other, one is subversive
        let _sub = app.world_mut().spawn((
            crate::layer1::pop::Pop,
            crate::layer1::core::map::GridPosition { x: 10, y: 10 },
            Subversive { dissent: 10.0 }
        )).id();
        let innocent = app.world_mut().spawn((
            crate::layer1::pop::Pop,
            crate::layer1::core::map::GridPosition { x: 10, y: 11 },
            Subversive { dissent: 0.0 }
        )).id();

        // Act
        app.update();

        // Assert
        let innocent_dissent = app.world().get::<Subversive>(innocent).unwrap().dissent;
        assert!(innocent_dissent > 0.0, "Dissent should spread to nearby pops");
    }

    #[test]
    fn test_surveillance_building_reduces_subversion_spread() {
        let mut app = bevy::app::App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_systems(bevy::app::Update, run_subversion_spread_system);

        let _sub = app.world_mut().spawn((
            crate::layer1::pop::Pop,
            crate::layer1::core::map::GridPosition { x: 10, y: 10 },
            Subversive { dissent: 10.0 }
        )).id();
        let innocent = app.world_mut().spawn((
            crate::layer1::pop::Pop,
            crate::layer1::core::map::GridPosition { x: 10, y: 11 },
            Subversive { dissent: 0.0 }
        )).id();

        use crate::layer1::architecture::building::{Building, BuildingType};
        // Add surveillance building covering both pops
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Tower,
            },
            Surveillance { radius: 5 },
            crate::layer1::core::map::GridPosition { x: 10, y: 10 }
        ));

        // Act
        app.update();

        // Assert
        let innocent_dissent = app.world().get::<Subversive>(innocent).unwrap().dissent;
        assert_eq!(innocent_dissent, 0.0, "Surveillance should prevent dissent spread");
    }

    #[test]
    fn test_surveillance_building_lowers_pop_morale() {
        let mut app = bevy::app::App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_systems(bevy::app::Update, run_surveillance_morale_system);

        let pop = app.world_mut().spawn((
            crate::layer1::pop::Pop,
            crate::layer1::core::map::GridPosition { x: 10, y: 10 },
            crate::layer1::social::morale::Morale::default()
        )).id();

        use crate::layer1::architecture::building::{Building, BuildingType};
        // Add surveillance building covering the pop
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Tower,
            },
            Surveillance { radius: 5 },
            crate::layer1::core::map::GridPosition { x: 10, y: 10 }
        ));

        // Act
        app.update();

        // Assert
        let pop_morale = app.world().get::<crate::layer1::social::morale::Morale>(pop).unwrap();
        // Verify morale is penalized by the surveillance building
        assert!(pop_morale.modifiers.iter().any(|m| m.value < 0.0), "Surveillance should lower morale");
    }
}
