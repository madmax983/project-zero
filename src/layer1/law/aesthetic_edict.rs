use bevy_ecs::prelude::*;

/// Marker component for a building that has been halted.
#[derive(Component)]
pub struct Halted {
    pub reason: String,
}

use crate::layer1::administration::edicts::{ColonyPolicies, Policy};
use crate::layer1::building::{Building, Category};
use crate::layer1::nature::atmosphere::AtmosphereGrid;
use crate::layer2::station::Station;
use crate::layer2::fleet::StationType;

pub fn evaluate_aesthetic_edict_system(
    policies: Option<ResMut<ColonyPolicies>>,
    grid: Option<Res<AtmosphereGrid>>,
    stations: Query<&Station>,
) {
    let mut policies = match policies {
        Some(p) => p,
        None => return,
    };

    let mut grid_pollution = 0.0;
    if let Some(g) = grid {
        for x in 0..g.width {
            for y in 0..g.height {
                grid_pollution += g.get(x as i32, y as i32);
            }
        }
    }

    let has_habitat = stations
        .iter()
        .any(|s| s.station_type == StationType::Habitat);
    let high_pollution = grid_pollution > 1000.0; // Tuning threshold

    let should_be_active = has_habitat && high_pollution;
    let is_active = policies.is_active(Policy::Aesthetic);

    if should_be_active && !is_active {
        policies.active_policies.insert(Policy::Aesthetic);
    } else if !should_be_active && is_active {
        policies.active_policies.remove(&Policy::Aesthetic);
    }
}

pub fn enforce_aesthetic_edict_system(
    mut commands: Commands,
    policies: Option<Res<ColonyPolicies>>,
    buildings: Query<(Entity, &Building, Option<&Halted>)>,
) {
    let is_active = policies.is_some_and(|p| p.is_active(Policy::Aesthetic));

    for (entity, building, halted) in buildings.iter() {
        // "Heavy Industry" defined as Manufacturing Category buildings.
        let is_heavy_industry = building
            .building_type
            .tier_info()
            .is_some_and(|(category, _)| category == Category::Manufacturing);

        if is_heavy_industry {
            if is_active && halted.is_none() {
                commands.entity(entity).insert(Halted {
                    reason: "Aesthetic Orbital Blockade".to_string(),
                });
            } else if !is_active && halted.is_some() {
                commands.entity(entity).remove::<Halted>();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::administration::edicts::{ColonyPolicies, Policy};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::nature::atmosphere::AtmosphereGrid;
    use crate::layer2::station::Station;
use crate::layer2::fleet::StationType;
    use bevy_app::App;

    #[test]
    fn test_high_pollution_triggers_aesthetic_edict() {
        // Arrange: App with an orbital station and high-wealth pops, and a surface with high pollution
        let mut app = App::new();

        app.insert_resource(ColonyPolicies::default());

        let mut grid = AtmosphereGrid::new(10, 10);
        // Setup Layer 1 pollution levels above threshold
        for x in 0..10 {
            for y in 0..10 {
                grid.set(x, y, 100.0);
            }
        }
        app.insert_resource(grid);

        // Layer 2 OrbitalStation (Habitat)
        app.world_mut().spawn(Station {
            station_type: StationType::Habitat,
        });

        // Add the system to evaluate the edict
        app.add_systems(bevy_app::Update, evaluate_aesthetic_edict_system);

        // Act: Update app to process orbital observations and politics
        app.update();

        // Assert: The active edicts resource is updated with 'Aesthetic'.
        let policies = app.world().resource::<ColonyPolicies>();
        assert!(policies.is_active(Policy::Aesthetic));
    }

    #[test]
    fn test_aesthetic_edict_halts_factory_production() {
        // Arrange: Active Aesthetic Edict, and a surface factory
        let mut app = App::new();

        let mut policies = ColonyPolicies::default();
        policies.active_policies.insert(Policy::Aesthetic);
        app.insert_resource(policies);

        let factory = app
            .world_mut()
            .spawn(Building {
                building_type: BuildingType::Smelter,
            })
            .id();

        // Add the system to enforce the edict
        app.add_systems(bevy_app::Update, enforce_aesthetic_edict_system);

        // Act: Process production systems
        app.update();

        // Assert: Factory should have a 'Halted' component.
        let is_halted = app.world().get::<Halted>(factory).is_some();
        assert!(is_halted, "Factory should have a 'Halted' component.");
    }

    #[test]
    fn test_tearing_down_habitats_restores_industry() {
        // Arrange: App with halted factories and an orbital station
        let mut app = App::new();

        let mut policies = ColonyPolicies::default();
        policies.active_policies.insert(Policy::Aesthetic);
        app.insert_resource(policies);

        let grid = AtmosphereGrid::new(10, 10);
        app.insert_resource(grid);

        let factory = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Smelter,
                },
                Halted {
                    reason: "Aesthetic Edict".to_string(),
                },
            ))
            .id();

        app.add_systems(
            bevy_app::Update,
            (
                evaluate_aesthetic_edict_system,
                enforce_aesthetic_edict_system,
            )
                .chain(),
        );

        // Act: Update without a habitat present
        app.update();

        // Assert: The 'AestheticEdict' is lifted, and factory production resumes.
        let policies = app.world().resource::<ColonyPolicies>();
        assert!(!policies.is_active(Policy::Aesthetic));

        let is_halted = app.world().get::<Halted>(factory).is_some();
        assert!(!is_halted, "Factory should no longer be halted.");
    }
}
