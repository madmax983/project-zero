#[cfg(test)]
mod tests {
    use crate::layer1::environment::hazards::calculate_risk;
    use crate::layer1::map::GridPosition;
    use crate::layer1::mother_lode::MotherLode;
    use crate::layer1::resources::{ColonyResources, ResourceType};
    use crate::layer1::seasons::SeasonState;
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::structure::Structure;
    use crate::layer1::temperature::{update_temperature_system, TemperatureGrid};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_mother_lode_initialization() {
        let lode = MotherLode {
            resource_type: ResourceType::Metal,
            current_hazard: 1.0,
            heat_output: 10.0,
        };
        assert_eq!(lode.resource_type, ResourceType::Metal);
        assert_eq!(lode.current_hazard, 1.0);
    }

    #[test]
    fn test_mining_increases_hazard_and_heat() {
        let mut world = World::new();
        // Setup Mother Lode entity
        let lode_entity = world
            .spawn((
                MotherLode {
                    resource_type: ResourceType::Metal,
                    current_hazard: 1.0,
                    heat_output: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Setup resources
        world.insert_resource(ColonyResources::default());

        let mut lode = world.get_mut::<MotherLode>(lode_entity).unwrap();
        lode.increment_hazard();

        assert!(lode.current_hazard > 1.0, "Hazard should increase");
        assert!(lode.heat_output > 10.0, "Heat should increase");
    }

    #[test]
    fn test_heat_emission_into_grid() {
        let mut world = World::new();
        // Setup Grid
        let grid = TemperatureGrid::new(10, 10, 0.0);
        world.insert_resource(grid);
        world.insert_resource(SeasonState::default());
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        // Spawn Lode
        world.spawn((
            MotherLode {
                resource_type: ResourceType::Metal,
                current_hazard: 1.0,
                heat_output: 50.0, // Hot!
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Run temperature update
        world.run_system_once(update_temperature_system).unwrap();

        let grid = world.resource::<TemperatureGrid>();
        // Heat is added, then diffused. So it will be slightly less than 50.0.
        // Ambient is 0.0.
        assert!(
            grid.get(5, 5) > 5.0,
            "Grid should receive heat from Lode. Got: {}",
            grid.get(5, 5)
        );
    }

    #[test]
    fn test_hazard_impacts_risk_calculation() {
        let skills = Skills::default();
        let structure = Structure::default();
        let base_risk = 0.001;

        let risk_normal = calculate_risk(base_risk, &skills, SkillType::Mining, &structure, 1.0);
        let risk_mother_lode =
            calculate_risk(base_risk, &skills, SkillType::Mining, &structure, 10.0); // 10x hazard

        assert!(
            risk_mother_lode > risk_normal * 5.0,
            "High hazard should significantly increase risk. Normal: {}, Mother Lode: {}",
            risk_normal,
            risk_mother_lode
        );
    }
}
