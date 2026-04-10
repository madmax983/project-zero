use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::lighting::LightMap;
use crate::layer1::map::GridPosition;
use crate::layer1::nature::temperature::TemperatureGrid;
use crate::layer1::structure::Structure;
use bevy_ecs::prelude::*;

#[derive(Event, Debug, Clone)]
pub struct SymbioticDormancyEvent {
    pub entity: Entity,
}

#[derive(Component)]
pub struct SymbioticStructure {
    pub regeneration_rate: f32,
    pub is_dormant: bool,
    pub is_starving: bool,
}

#[derive(Component)]
pub struct SymbioticNeeds {
    pub required_light: f32,
    pub required_temp: f32,
}

// Dummy environment component for testing
#[derive(Component)]
pub struct EnvironmentStatus {
    pub current_light: f32,
    pub current_temp: f32,
}

pub fn process_symbiotic_needs_system(
    mut query: Query<(
        Entity,
        &mut SymbioticStructure,
        &SymbioticNeeds,
        &GridPosition,
    )>,
    light_map: Option<Res<LightMap>>,
    temp_grid: Option<Res<TemperatureGrid>>,
    mut resources: Option<ResMut<ColonyResources>>,
    mut structures: Query<&mut Structure>,
    mut dormancy_events: EventWriter<SymbioticDormancyEvent>,
) {
    let has_light_map = light_map.is_some();
    let has_temp_grid = temp_grid.is_some();

    for (entity, mut symbiotic, needs, pos) in query.iter_mut() {
        let x = pos.x.try_into().unwrap_or(0);
        let y = pos.y.try_into().unwrap_or(0);
        let current_light = if let Some(lm) = &light_map {
            lm.get(x, y)
        } else {
            0.0
        };
        let current_temp = if let Some(tg) = &temp_grid {
            tg.get(x as usize, y as usize)
        } else {
            0.0
        };

        let previously_dormant = symbiotic.is_dormant;

        symbiotic.is_dormant = (has_light_map && current_light < needs.required_light)
            || (has_temp_grid && current_temp < needs.required_temp);

        if !previously_dormant && symbiotic.is_dormant {
            dormancy_events.send(SymbioticDormancyEvent { entity });
        }

        if !symbiotic.is_dormant {
            let mut cost = ColonyResources::zeroed();
            cost.food = 1.0;
            if let Some(res) = &mut resources {
                if res.try_deduct(&cost) {
                    symbiotic.is_starving = false;
                } else {
                    symbiotic.is_starving = true;
                    if let Ok(mut structure) = structures.get_mut(entity) {
                        structure.current_hp = (structure.current_hp - 1.0).max(0.0);
                    }
                }
            } else {
                // If resources don't exist, we starve
                symbiotic.is_starving = true;
                if let Ok(mut structure) = structures.get_mut(entity) {
                    structure.current_hp = (structure.current_hp - 1.0).max(0.0);
                }
            }
        } else {
            symbiotic.is_starving = false;
        }
    }
}

pub fn process_symbiotic_regeneration_system(
    mut query: Query<(&mut Structure, &SymbioticStructure)>,
) {
    for (mut structure, symbiotic) in query.iter_mut() {
        if !symbiotic.is_dormant && !symbiotic.is_starving {
            structure.current_hp += symbiotic.regeneration_rate;
            if structure.current_hp > structure.max_hp {
                structure.current_hp = structure.max_hp;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::structure::Structure;
    use bevy_app::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(LightMap::new(10, 10));
        app.insert_resource(TemperatureGrid::new(10, 10, 20.0));
        let resources = ColonyResources {
            food: 100.0,
            ..Default::default()
        };
        app.insert_resource(resources);
        app.add_event::<SymbioticDormancyEvent>();

        app.add_systems(
            Update,
            (
                process_symbiotic_needs_system,
                process_symbiotic_regeneration_system.after(process_symbiotic_needs_system),
            ),
        );
        app
    }

    #[test]
    fn test_symbiotic_structure_regenerates_health() {
        let mut app = setup_app();

        // Needs light
        app.world_mut().resource_mut::<LightMap>().set(0, 0, 1.0);
        app.world_mut()
            .resource_mut::<TemperatureGrid>()
            .set(0, 0, 20.0);

        // Spawn a damaged symbiotic structure
        let entity = app
            .world_mut()
            .spawn((
                Structure {
                    current_hp: 50.0,
                    max_hp: 100.0,
                },
                SymbioticStructure {
                    regeneration_rate: 5.0,
                    is_dormant: false,
                    is_starving: false,
                },
                SymbioticNeeds {
                    required_light: 0.5,
                    required_temp: 10.0,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        app.update();

        // Structure HP should increase
        let structure = app.world().get::<Structure>(entity).unwrap();
        assert_eq!(
            structure.current_hp, 55.0,
            "Symbiotic structure should regenerate HP when not dormant"
        );
    }

    #[test]
    fn test_symbiotic_structure_goes_dormant_if_needs_unmet() {
        let mut app = setup_app();

        app.world_mut().resource_mut::<LightMap>().set(0, 0, 0.0);

        // Spawn structure with specific light need
        let entity = app
            .world_mut()
            .spawn((
                Structure {
                    current_hp: 50.0,
                    max_hp: 100.0,
                },
                SymbioticStructure {
                    regeneration_rate: 5.0,
                    is_dormant: false,
                    is_starving: false,
                },
                SymbioticNeeds {
                    required_light: 100.0,
                    required_temp: 20.0,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        app.update();

        // Structure should become dormant
        let symbiotic = app.world().get::<SymbioticStructure>(entity).unwrap();
        assert!(
            symbiotic.is_dormant,
            "Structure should enter dormancy when light needs are unmet"
        );

        // Dormant structure should NOT regenerate
        app.update();
        let structure = app.world().get::<Structure>(entity).unwrap();
        assert_eq!(
            structure.current_hp, 50.0,
            "Dormant structure should not regenerate health"
        );
    }

    #[test]
    fn test_symbiotic_structure_starves() {
        let mut app = setup_app();

        app.world_mut().resource_mut::<LightMap>().set(0, 0, 1.0);
        app.world_mut()
            .resource_mut::<TemperatureGrid>()
            .set(0, 0, 20.0);
        app.world_mut().resource_mut::<ColonyResources>().food = 0.0;

        let entity = app
            .world_mut()
            .spawn((
                Structure {
                    current_hp: 50.0,
                    max_hp: 100.0,
                },
                SymbioticStructure {
                    regeneration_rate: 5.0,
                    is_dormant: false,
                    is_starving: false,
                },
                SymbioticNeeds {
                    required_light: 0.5,
                    required_temp: 10.0,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        app.update();

        let structure = app.world().get::<Structure>(entity).unwrap();
        assert_eq!(structure.current_hp, 49.0, "Should take starvation damage");
    }
}
