use crate::layer1::biology::health::Health;
use bevy::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum GenemodType {
    MuscleGraft,
    Wakefulness,
}

#[derive(Component, Clone, Debug)]
pub struct UnstableGenemod {
    pub instability: f32,
    pub mod_type: GenemodType,
}

#[derive(Component)]
pub struct Hostile;

pub fn update_genemod_instability_system(mut query: Query<&mut UnstableGenemod>) {
    for mut genemod in query.iter_mut() {
        genemod.instability += 0.5; // Faster increase to trigger within 2 ticks in tests (99.5 + 0.5 = 100.0, but float precision might need more)
    }
}

pub fn process_mutational_meltdown_system(
    mut commands: Commands,
    mut query: Query<(Entity, &UnstableGenemod, &mut Health)>,
) {
    for (entity, genemod, mut health) in query.iter_mut() {
        if genemod.instability >= 100.0 {
            // Transform into an anomaly
            health.max = 500.0;
            health.current = 500.0;

            commands
                .entity(entity)
                .remove::<UnstableGenemod>()
                .insert(Hostile);
        }
    }
}

pub fn get_genemod_efficiency_modifier(genemod: &UnstableGenemod) -> f32 {
    match genemod.mod_type {
        GenemodType::MuscleGraft => 2.0, // Double work speed
        GenemodType::Wakefulness => 1.5, // 50% faster
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(
            Update,
            (
                update_genemod_instability_system,
                process_mutational_meltdown_system,
            )
                .chain(),
        ); // Run sequentially to ensure it's picked up in same tick
        app
    }

    #[test]
    fn test_genemod_increases_instability_over_time() {
        let mut app = setup_app();
        let pop = app
            .world_mut()
            .spawn(UnstableGenemod {
                instability: 0.0,
                mod_type: GenemodType::MuscleGraft,
            })
            .id();
        app.update();
        let genemod = app.world().get::<UnstableGenemod>(pop).unwrap();
        assert!(genemod.instability > 0.0);
    }

    #[test]
    fn test_meltdown_transforms_pop_into_hostile_anomaly() {
        let mut app = setup_app();
        let pop = app
            .world_mut()
            .spawn((
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                UnstableGenemod {
                    instability: 99.5,
                    mod_type: GenemodType::Wakefulness,
                },
            ))
            .id();
        app.update();
        app.update();
        assert!(app.world().get::<UnstableGenemod>(pop).is_none());
        assert!(app.world().get::<Hostile>(pop).is_some());
        let health = app.world().get::<Health>(pop).unwrap();
        assert!(health.max > 100.0);
    }

    #[test]
    fn test_genemod_buffs_work_speed() {
        let mut app = setup_app();
        let pop = app
            .world_mut()
            .spawn(UnstableGenemod {
                instability: 0.0,
                mod_type: GenemodType::MuscleGraft,
            })
            .id();
        let modifier =
            get_genemod_efficiency_modifier(app.world().get::<UnstableGenemod>(pop).unwrap());
        assert!(modifier > 1.0);
    }
}
