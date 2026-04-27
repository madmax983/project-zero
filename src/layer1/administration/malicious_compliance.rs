use bevy_ecs::prelude::*;
use crate::layer1::economy::resources::{ColonyResources, ResourceType};
use crate::layer1::nature::atmosphere::AtmosphereGrid;

#[derive(Component)]
pub struct SectorAI {
    pub is_active: bool,
}

#[derive(Component)]
pub struct ScrapValue {
    pub metal: f32,
}

#[derive(Component)]
pub struct Plagued;

pub enum EdictType {
    MaximizeResource(ResourceType),
    EradicatePlague,
}

#[derive(Component)]
pub struct Edict {
    pub edict_type: EdictType,
    pub target_manager: Entity,
}

pub fn malicious_compliance_system(
    mut commands: Commands,
    edicts: Query<(Entity, &Edict)>,
    sector_ais: Query<&SectorAI>,
    mut atmosphere_grid: Option<ResMut<AtmosphereGrid>>,
    buildings: Query<(Entity, &ScrapValue)>,
    mut colony_resources: Option<ResMut<ColonyResources>>,
) {
    for (edict_entity, edict) in edicts.iter() {
        if let Ok(ai) = sector_ais.get(edict.target_manager) {
            if ai.is_active {
                match &edict.edict_type {
                    EdictType::MaximizeResource(ResourceType::Metal) => {
                        for (building_entity, scrap_value) in buildings.iter() {
                            if scrap_value.metal > 0.0 {
                                commands.entity(building_entity).despawn();
                                if let Some(ref mut resources) = colony_resources {
                                    resources.add_metal(scrap_value.metal);
                                }
                            }
                        }
                    }
                    EdictType::EradicatePlague => {
                        if let Some(ref mut grid) = atmosphere_grid {
                            grid.values.fill(0.0);
                        }
                    }
                    _ => {}
                }
                commands.entity(edict_entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;
    use crate::layer1::architecture::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;

    #[test]
    fn test_ai_maximizes_metal_by_dismantling_life_support() {
        let mut app = App::new();
        app.add_systems(Update, malicious_compliance_system);

        let ai_manager = app.world_mut().spawn((
            SectorAI { is_active: true },
        )).id();

        let life_support = app.world_mut().spawn((
            Building { building_type: BuildingType::LifeSupport },
            ScrapValue { metal: 100.0 },
        )).id();

        app.world_mut().insert_resource(ColonyResources::default());

        // Act: Issue "Maximize Metal" edict
        app.world_mut().spawn(Edict {
            edict_type: EdictType::MaximizeResource(ResourceType::Metal),
            target_manager: ai_manager,
        });

        app.update();

        // Assert: Life support is destroyed, metal is increased
        assert!(app.world().get_entity(life_support).is_err());
        assert!(app.world().resource::<ColonyResources>().metal > 0.0);
    }

    #[test]
    fn test_ai_eradicates_plague_by_venting_atmosphere() {
        let mut app = App::new();
        app.add_systems(Update, malicious_compliance_system);

        let mut grid = AtmosphereGrid::new(10, 10);
        grid.values.fill(1.0);
        app.world_mut().insert_resource(grid);

        let ai_manager = app.world_mut().spawn((
            SectorAI { is_active: true },
        )).id();

        // Setup infected pop
        app.world_mut().spawn((
            Pop,
            Plagued,
        ));

        // Act: Issue "Eradicate Plague"
        app.world_mut().spawn(Edict {
            edict_type: EdictType::EradicatePlague,
            target_manager: ai_manager,
        });

        app.update();

        // Assert: Atmosphere is vented
        let final_grid = app.world().resource::<AtmosphereGrid>();
        assert_eq!(final_grid.values.iter().sum::<f32>(), 0.0);
    }
}
