use crate::layer1::map::GridPosition;
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
    mut query: Query<(Entity, &GridPosition, &mut Subversive)>,
    surveillance_query: Query<(&GridPosition, &Surveillance)>,
) {
    let mut interactions = Vec::new();

    // Collect all subversives and their positions
    let pops: Vec<(Entity, GridPosition, f32)> = query
        .iter()
        .map(|(e, pos, sub)| (e, *pos, sub.dissent))
        .collect();

    // Calculate spread
    for i in 0..pops.len() {
        let (_e1, pos1, dissent1) = pops[i];
        if dissent1 <= 0.0 {
            continue;
        }

        for (j, item) in pops.iter().enumerate() {
            if i == j {
                continue;
            }
            let (e2, pos2, _) = item;

            // Check distance (simple Manhattan distance for adjacent check)
            let dx = (pos1.x - pos2.x).abs();
            let dy = (pos1.y - pos2.y).abs();

            if dx <= 1 && dy <= 1 {
                // Check if target is under surveillance
                let mut under_surveillance = false;
                for (s_pos, surv) in surveillance_query.iter() {
                    let s_dx = (pos2.x - s_pos.x).abs();
                    let s_dy = (pos2.y - s_pos.y).abs();
                    // Using Chebyshev distance for radius
                    if s_dx <= surv.radius && s_dy <= surv.radius {
                        under_surveillance = true;
                        break;
                    }
                }

                if !under_surveillance {
                    interactions.push((*e2, 1.0)); // Spread amount
                }
            }
        }
    }

    // Apply spread
    for (target, amount) in interactions {
        if let Ok((_, _, mut sub)) = query.get_mut(target) {
            sub.dissent += amount;
        }
    }
}

pub fn run_surveillance_morale_system(
    mut query: Query<(&GridPosition, &mut Morale)>,
    surveillance_query: Query<(&GridPosition, &Surveillance)>,
) {
    for (pos, mut morale) in query.iter_mut() {
        let mut under_surveillance = false;
        for (s_pos, surv) in surveillance_query.iter() {
            let dx = (pos.x - s_pos.x).abs();
            let dy = (pos.y - s_pos.y).abs();
            if dx <= surv.radius && dy <= surv.radius {
                under_surveillance = true;
                break;
            }
        }

        if under_surveillance {
            // Apply morale penalty
            morale.add_modifier(MoodModifier {
                label: "Under Surveillance".to_string(),
                value: -0.1,
                duration: 1, // temporary penalty
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;

    #[test]
    fn test_subversive_trait_spreads_to_nearby_pops() {
        let mut app = bevy::app::App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_systems(bevy::app::Update, run_subversion_spread_system);

        // Arrange: Two pops close to each other, one is subversive
        let _sub = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 10, y: 10 },
                Subversive { dissent: 10.0 },
            ))
            .id();
        let innocent = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 10, y: 11 },
                Subversive { dissent: 0.0 },
            ))
            .id();

        // Act
        app.update();

        // Assert
        let innocent_dissent = app.world().get::<Subversive>(innocent).unwrap().dissent;
        assert!(
            innocent_dissent > 0.0,
            "Dissent should spread to nearby pops"
        );
    }

    #[test]
    fn test_surveillance_building_reduces_subversion_spread() {
        let mut app = bevy::app::App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_systems(bevy::app::Update, run_subversion_spread_system);

        let _sub = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 10, y: 10 },
                Subversive { dissent: 10.0 },
            ))
            .id();
        let innocent = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 10, y: 11 },
                Subversive { dissent: 0.0 },
            ))
            .id();

        // Add surveillance building covering both pops
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Tower,
            },
            Surveillance { radius: 5 },
            GridPosition { x: 10, y: 10 },
        ));

        // Act
        app.update();

        // Assert
        let innocent_dissent = app.world().get::<Subversive>(innocent).unwrap().dissent;
        assert_eq!(
            innocent_dissent, 0.0,
            "Surveillance should prevent dissent spread"
        );
    }

    #[test]
    fn test_surveillance_building_lowers_pop_morale() {
        let mut app = bevy::app::App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_systems(bevy::app::Update, run_surveillance_morale_system);

        let pop = app
            .world_mut()
            .spawn((Pop, GridPosition { x: 10, y: 10 }, Morale::default()))
            .id();

        // Add surveillance building covering the pop
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Tower,
            },
            Surveillance { radius: 5 },
            GridPosition { x: 10, y: 10 },
        ));

        // Act
        app.update();

        // Assert
        let pop_morale = app.world().get::<Morale>(pop).unwrap();
        // Verify morale is penalized by the surveillance building
        assert!(
            pop_morale.modifiers.iter().any(|m| m.value < 0.0),
            "Surveillance should lower morale"
        );
    }
}
