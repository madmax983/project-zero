use bevy_ecs::prelude::*;
use bevy::prelude::{Transform, Vec3};
use crate::layer1::geology::tectonic::TectonicStress;

#[derive(Component)]
pub struct TectonicRoot {
    pub growth_progress: f32,
    pub base_growth_rate: f32,
}

#[derive(Event)]
pub struct HarvestCropEvent {
    pub crop: Entity,
}

#[derive(Event)]
pub struct LocalizedTremorEvent {
    pub location: Vec3,
    pub magnitude: f32,
}

pub fn grow_tectonic_roots_system(
    mut query: Query<&mut TectonicRoot>,
    stress: Res<TectonicStress>,
) {
    let stress_multiplier = 1.0 + (stress.current / 100.0);
    for mut root in query.iter_mut() {
        root.growth_progress += root.base_growth_rate * stress_multiplier;
    }
}

pub fn harvest_tectonic_roots_system(
    mut harvest_events: EventReader<HarvestCropEvent>,
    mut stress: ResMut<TectonicStress>,
    mut tremor_events: EventWriter<LocalizedTremorEvent>,
    roots: Query<&bevy::hierarchy::Parent, With<TectonicRoot>>,
    transforms: Query<&Transform>,
) {
    for event in harvest_events.read() {
        if let Ok(parent) = roots.get(event.crop) {
            if let Ok(transform) = transforms.get(parent.get()) {
                stress.current += 5.0; // Increase stress slightly per harvest

                if stress.current >= stress.threshold {
                    tremor_events.send(LocalizedTremorEvent {
                        location: transform.translation,
                        magnitude: stress.current / 10.0,
                    });
                    stress.current = (stress.current - 50.0).max(0.0); // Release some stress after the tremor
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_tectonic_root_growth_accelerated_by_stress() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<TectonicStress>()
            .add_systems(Update, grow_tectonic_roots_system);

        let root_entity = app.world_mut().spawn((
            TectonicRoot { growth_progress: 0.0, base_growth_rate: 1.0 },
        )).id();

        // High stress accelerates growth
        app.world_mut().resource_mut::<TectonicStress>().current = 80.0;
        app.update();

        let growth = app.world().get::<TectonicRoot>(root_entity).unwrap().growth_progress;
        assert!(growth > 1.0, "Growth should be accelerated by high tectonic stress");
    }

    #[test]
    fn test_harvesting_roots_increases_local_instability() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_event::<HarvestCropEvent>()
            .add_event::<LocalizedTremorEvent>()
            .init_resource::<TectonicStress>()
            .add_systems(Update, harvest_tectonic_roots_system);

        let farm_entity = app.world_mut().spawn(Transform::from_xyz(10.0, 0.0, 10.0)).id();
        let mut farm = app.world_mut().entity_mut(farm_entity);
        farm.with_children(|parent| {
            parent.spawn(TectonicRoot { growth_progress: 100.0, base_growth_rate: 1.0 });
        });

        // We need to retrieve root_entity, which is the child of farm_entity
        let root_entity = app.world().get::<bevy::hierarchy::Children>(farm_entity).unwrap()[0];

        app.world_mut().send_event(HarvestCropEvent { crop: root_entity });

        let initial_stress = app.world().resource::<TectonicStress>().current;
        app.update();

        let final_stress = app.world().resource::<TectonicStress>().current;
        assert!(final_stress > initial_stress, "Harvesting tectonic roots should increase global tectonic stress");
    }

    #[test]
    fn test_excessive_harvesting_triggers_localized_tremor() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_event::<HarvestCropEvent>()
            .add_event::<LocalizedTremorEvent>()
            .init_resource::<TectonicStress>()
            .add_systems(Update, harvest_tectonic_roots_system);

        let farm_entity = app.world_mut().spawn(Transform::from_xyz(10.0, 0.0, 10.0)).id();
        let mut farm = app.world_mut().entity_mut(farm_entity);
        farm.with_children(|parent| {
            parent.spawn(TectonicRoot { growth_progress: 100.0, base_growth_rate: 1.0 });
        });
        let root_entity = app.world().get::<bevy::hierarchy::Children>(farm_entity).unwrap()[0];

        // Push stress near the threshold
        app.world_mut().resource_mut::<TectonicStress>().current = 95.0;

        app.world_mut().send_event(HarvestCropEvent { crop: root_entity });
        app.update();

        let tremor_events = app.world().resource::<Events<LocalizedTremorEvent>>();
        assert_eq!(tremor_events.get_cursor().len(&tremor_events), 1, "Excessive harvesting should trigger a localized tremor event");
    }
}
