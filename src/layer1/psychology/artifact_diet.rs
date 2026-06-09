use crate::layer1::culture::artifacts::Artifact;
use crate::layer1::entities::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

/// Marker component for artifacts that can be consumed in emergencies.
#[derive(Component)]
pub struct EdibleAnomaly;

pub fn consume_artifacts_during_famine_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut Needs, Option<&mut Traits>), With<Pop>>,
    artifacts: Query<Entity, (With<Artifact>, With<EdibleAnomaly>)>,
) {
    for (pop_entity, mut needs, traits_opt) in pops.iter_mut() {
        // Threshold for severe famine: hunger approaches 0 in Needs system
        if needs.hunger <= 0.1 {
            if let Some(artifact_entity) = artifacts.iter().next() {
                // Consume artifact
                commands.entity(artifact_entity).despawn();

                // Satisfy hunger (1.0 is full)
                needs.hunger = 1.0;

                // Apply mutation (Phantom is an example of an alien-like trait)
                if let Some(mut traits) = traits_opt {
                    if !traits.has(Trait::Phantom) {
                        traits.add(Trait::Phantom);
                    }
                } else {
                    let mut new_traits = Traits::default();
                    new_traits.add(Trait::Phantom);
                    commands.entity(pop_entity).insert(new_traits);
                }

                break; // One artifact consumed per pop evaluation
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::culture::artifacts::Artifact;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::psychology::needs::Needs;
    use crate::layer1::psychology::traits::{Trait, Traits};
    use bevy_app::{App, Update};

    #[test]
    fn test_starving_pop_consumes_artifact() {
        let mut app = App::new();
        app.add_systems(Update, consume_artifacts_during_famine_system);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                Needs {
                    hunger: 0.05,
                    rest: 1.0,
                    leisure: 1.0,
                    hygiene: 1.0,
                }, // Starving
            ))
            .id();

        let artifact_entity = app.world_mut().spawn((Artifact, EdibleAnomaly)).id();

        app.update();

        let needs = app.world().get::<Needs>(pop_entity).unwrap();
        assert!(
            needs.hunger > 0.9,
            "Hunger should be satisfied by the artifact"
        );
        assert!(
            app.world().get_entity(artifact_entity).is_err(),
            "Artifact should be consumed"
        );

        let traits = app.world().get::<Traits>(pop_entity).unwrap();
        assert!(
            traits.has(Trait::Phantom),
            "Consuming an artifact should cause a trait mutation"
        );
    }

    #[test]
    fn test_well_fed_pop_ignores_artifact() {
        let mut app = App::new();
        app.add_systems(Update, consume_artifacts_during_famine_system);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                Needs {
                    hunger: 0.8,
                    rest: 1.0,
                    leisure: 1.0,
                    hygiene: 1.0,
                }, // Well-fed
            ))
            .id();

        let artifact_entity = app.world_mut().spawn((Artifact, EdibleAnomaly)).id();

        app.update();

        let needs = app.world().get::<Needs>(pop_entity).unwrap();
        assert_eq!(needs.hunger, 0.8, "Hunger should remain unchanged");
        assert!(
            app.world().get_entity(artifact_entity).is_ok(),
            "Artifact should NOT be consumed if pop is not starving"
        );
    }
}
