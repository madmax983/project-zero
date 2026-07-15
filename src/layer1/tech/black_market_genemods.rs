#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::biology::health::Health;
    use crate::layer1::culture::cultural_influence::{Alignment, Culture};
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(
            Update,
            (
                update_genemod_instability_system,
                process_mutational_meltdown_system,
            ),
        );
        app.add_event::<AddChronicleEvent>();
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

        // Spawn pop near meltdown threshold
        let pop = app
            .world_mut()
            .spawn((
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                UnstableGenemod {
                    instability: 99.99,
                    mod_type: GenemodType::Wakefulness,
                },
            ))
            .id();

        app.update();
        app.update();

        // Pop should no longer have UnstableGenemod
        assert!(app.world().get::<UnstableGenemod>(pop).is_none());

        // Pop should now be hostile
        let culture = app.world().get::<Culture>(pop).unwrap();
        assert_eq!(culture.alignment, Alignment::Hostile);

        // Pop health should be massive
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
            get_genemod_efficiency_modifier(&app.world().get::<UnstableGenemod>(pop).unwrap());

        assert!(modifier > 1.0); // Should be a massive buff (e.g., 2.0x)
    }
}

use crate::layer1::biology::health::Health;
use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::culture::cultural_influence::{Alignment, Culture};
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

pub fn update_genemod_instability_system(mut query: Query<&mut UnstableGenemod>) {
    for mut genemod in query.iter_mut() {
        genemod.instability += 0.1; // Slow increase
    }
}

pub fn process_mutational_meltdown_system(
    mut commands: Commands,
    mut query: Query<(Entity, &UnstableGenemod, &mut Health)>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for (entity, genemod, mut health) in query.iter_mut() {
        if genemod.instability >= 100.0 {
            // Transform into an anomaly
            health.max = 500.0;
            health.current = 500.0;

            commands
                .entity(entity)
                .remove::<UnstableGenemod>()
                // Remove generic Civilian tags here if needed in full implementation
                .insert(Culture {
                    alignment: Alignment::Hostile,
                });

            chronicle_events.send(AddChronicleEvent {
                text: "A colonist has succumbed to genetic instability and suffered a mutational meltdown, becoming a hostile anomaly!".to_string(),
                importance: EventImportance::Major,
            });
        }
    }
}

pub fn get_genemod_efficiency_modifier(genemod: &UnstableGenemod) -> f32 {
    match genemod.mod_type {
        GenemodType::MuscleGraft => 2.0, // Double work speed
        GenemodType::Wakefulness => 1.5, // 50% faster
    }
}
