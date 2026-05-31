use bevy_ecs::prelude::*;

#[derive(Component, Clone)]
pub struct MaterialProperties {
    pub flammability: f32,
    pub insulation: f32,
    pub beauty: f32,
}

#[derive(Component)]
pub struct ConstructedFrom {
    pub materials: Vec<Entity>,
}

#[derive(Component)]
pub struct InheritedMaterialProperties {
    pub flammability: f32,
    pub insulation: f32,
    pub beauty: f32,
}

pub fn apply_material_provenance_system(
    mut commands: Commands,
    buildings: Query<(Entity, &ConstructedFrom), Without<InheritedMaterialProperties>>,
    materials: Query<&MaterialProperties>,
) {
    for (entity, constructed_from) in buildings.iter() {
        let mut total_flammability = 0.0;
        let mut total_insulation = 0.0;
        let mut total_beauty = 0.0;
        let mut count = 0.0;

        for &mat_entity in &constructed_from.materials {
            if let Ok(props) = materials.get(mat_entity) {
                total_flammability += props.flammability;
                total_insulation += props.insulation;
                total_beauty += props.beauty;
                count += 1.0;
            }
        }

        if count > 0.0 {
            commands.entity(entity).insert(InheritedMaterialProperties {
                flammability: total_flammability / count,
                insulation: total_insulation / count,
                beauty: total_beauty / count,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_building_inherits_material_properties() {
        let mut app = App::new();
        app.add_systems(Update, apply_material_provenance_system);

        let material_entity = app
            .world_mut()
            .spawn(MaterialProperties {
                flammability: 0.8,
                insulation: 0.2,
                beauty: 0.1,
            })
            .id();

        let building_entity = app
            .world_mut()
            .spawn(ConstructedFrom {
                materials: vec![material_entity],
            })
            .id();

        app.update();

        let inherited_props = app
            .world()
            .get::<InheritedMaterialProperties>(building_entity)
            .unwrap();
        assert_eq!(inherited_props.flammability, 0.8);
        assert_eq!(inherited_props.insulation, 0.2);
        assert_eq!(inherited_props.beauty, 0.1);
    }

    #[test]
    fn test_building_averages_multiple_materials() {
        let mut app = App::new();
        app.add_systems(Update, apply_material_provenance_system);

        let material1 = app
            .world_mut()
            .spawn(MaterialProperties {
                flammability: 1.0,
                insulation: 0.0,
                beauty: 0.0,
            })
            .id();

        let material2 = app
            .world_mut()
            .spawn(MaterialProperties {
                flammability: 0.0,
                insulation: 1.0,
                beauty: 1.0,
            })
            .id();

        let building_entity = app
            .world_mut()
            .spawn(ConstructedFrom {
                materials: vec![material1, material2],
            })
            .id();

        app.update();

        let inherited_props = app
            .world()
            .get::<InheritedMaterialProperties>(building_entity)
            .unwrap();
        assert_eq!(inherited_props.flammability, 0.5);
        assert_eq!(inherited_props.insulation, 0.5);
        assert_eq!(inherited_props.beauty, 0.5);
    }
}
