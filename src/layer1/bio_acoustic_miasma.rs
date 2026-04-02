use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct MiasmaCloud {
    pub position: GridPosition,
    pub lifetime: u32,
}

#[derive(Resource, Default)]
pub struct MiasmaRecordedSecret {
    pub secrets: Vec<String>,
}

#[derive(Component)]
pub struct ParanoiaTracker {
    pub level: u32,
}

pub fn record_miasma_secret(
    mut secrets: ResMut<MiasmaRecordedSecret>,
    cloud_query: Query<&MiasmaCloud>,
    pop_query: Query<(&GridPosition, &StressTracker), With<Pop>>,
) {
    // For each pop with high stress, check if they are in a miasma cloud
    for (pop_pos, stress) in pop_query.iter() {
        if stress.accumulated_stress > 50.0
            && cloud_query.iter().any(|cloud| cloud.position == *pop_pos)
        {
            secrets.secrets.push("high_stress_complaint".to_string());
        }
    }
}

pub fn broadcast_miasma_secrets(
    mut secrets: ResMut<MiasmaRecordedSecret>,
    mut paranoia_query: Query<&mut ParanoiaTracker>,
    mut events: EventWriter<AddChronicleEvent>,
) {
    if !secrets.secrets.is_empty() {
        for mut paranoia in paranoia_query.iter_mut() {
            paranoia.level += 10;
        }
        secrets.secrets.clear();
        events.send(AddChronicleEvent {
            text: "secret_broadcast".to_string(),
            importance: EventImportance::Minor,
        });
    }
}

pub fn update_miasma_clouds(
    mut commands: Commands,
    mut cloud_query: Query<(Entity, &mut MiasmaCloud)>,
) {
    for (entity, mut cloud) in cloud_query.iter_mut() {
        if cloud.lifetime <= 1 {
            commands.entity(entity).despawn();
        } else {
            cloud.lifetime -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::chronicle::Chronicle;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_miasma_records_interaction() {
        let mut world = World::new();
        world.init_resource::<MiasmaRecordedSecret>();

        let pos = GridPosition { x: 5, y: 5 };
        world.spawn(MiasmaCloud {
            position: pos,
            lifetime: 5,
        });
        let _gossiper = world
            .spawn((
                Pop,
                pos,
                StressTracker {
                    accumulated_stress: 80.0,
                },
            ))
            .id();

        world.run_system_once(record_miasma_secret).unwrap();

        let secrets = world.get_resource::<MiasmaRecordedSecret>().unwrap();
        assert!(secrets
            .secrets
            .contains(&"high_stress_complaint".to_string()));
    }

    #[test]
    fn test_miasma_broadcasts_secret() {
        let mut world = World::new();
        world.insert_resource(MiasmaRecordedSecret {
            secrets: vec!["plot_strike".to_string()],
        });
        world.init_resource::<Chronicle>();
        world.init_resource::<Events<AddChronicleEvent>>();

        let listener = world
            .spawn((
                Pop,
                GridPosition { x: 10, y: 10 },
                ParanoiaTracker { level: 0 },
            ))
            .id();

        world.run_system_once(broadcast_miasma_secrets).unwrap();

        let paranoia = world.get::<ParanoiaTracker>(listener).unwrap().level;
        assert!(paranoia > 0);

        let events = world.get_resource::<Events<AddChronicleEvent>>().unwrap();
        assert!(!events.is_empty());
    }

    #[test]
    fn test_miasma_dissipates() {
        let mut world = World::new();
        let pos = GridPosition { x: 5, y: 5 };
        let cloud = world
            .spawn(MiasmaCloud {
                position: pos,
                lifetime: 1,
            })
            .id();

        world.run_system_once(update_miasma_clouds).unwrap();

        assert!(world.get::<MiasmaCloud>(cloud).is_none());
    }
}
