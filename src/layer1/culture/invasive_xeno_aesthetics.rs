use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct XenoColony;

#[derive(Component)]
pub struct ResidentOf(pub Entity);

#[derive(Component)]
pub struct XenoMood {
    pub level: f32,
}

#[derive(Component)]
pub struct XenoCulturalArtifact {
    pub target_colony: Entity,
    pub source_culture: u32,
}

#[derive(Component)]
pub struct AestheticDemand {
    pub culture_id: u32,
    pub timer: u32,
}

#[derive(Component)]
pub struct AestheticDeprivation;

pub fn process_cultural_artifact_system(
    mut commands: Commands,
    artifacts: Query<(Entity, &XenoCulturalArtifact)>,
    mut pops: Query<(Entity, &ResidentOf, &mut XenoMood)>,
) {
    for (artifact_entity, artifact) in artifacts.iter() {
        for (pop_entity, resident_of, mut mood) in pops.iter_mut() {
            if resident_of.0 == artifact.target_colony {
                // Massive mood boost
                mood.level += 30.0;

                // Add the demand
                commands.entity(pop_entity).insert(AestheticDemand {
                    culture_id: artifact.source_culture,
                    timer: 500 // Arbitrary time to fulfill
                });
            }
        }
        // Consume the artifact
        commands.entity(artifact_entity).despawn();
    }
}

pub fn process_aesthetic_deprivation_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut AestheticDemand, &mut XenoMood)>,
) {
    for (entity, mut demand, mut mood) in pops.iter_mut() {
        if demand.timer > 0 {
            demand.timer -= 1;
        } else {
            // Demand expired unfulfilled
            mood.level -= 40.0; // Severe drop
            commands.entity(entity).insert(AestheticDeprivation);
            commands.entity(entity).remove::<AestheticDemand>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;

    // RED Phase Test Setup
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            process_cultural_artifact_system,
            process_aesthetic_deprivation_system,
        ));
        app
    }

    #[test]
    fn test_cultural_artifact_boosts_mood_and_adds_demand() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            crate::layer1::entities::pop::Pop,
            XenoMood { level: 50.0 },
        )).id();

        let colony = app.world_mut().spawn(XenoColony).id();

        // Simulate artifact arrival
        app.world_mut().spawn(XenoCulturalArtifact { target_colony: colony, source_culture: 2 });
        app.world_mut().entity_mut(pop).insert(ResidentOf(colony));

        app.update();

        let mood = app.world().get::<XenoMood>(pop).unwrap();
        assert!(mood.level > 50.0, "Artifact should boost mood");

        assert!(app.world().get::<AestheticDemand>(pop).is_some(), "crate::layer1::entities::pop::Pop should demand new aesthetics after exposure");
    }

    #[test]
    fn test_unfulfilled_aesthetic_demand_causes_deprivation() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            crate::layer1::entities::pop::Pop,
            XenoMood { level: 100.0 },
            AestheticDemand { culture_id: 2, timer: 10 },
        )).id();

        // Simulate timer expiration
        let mut demand = app.world_mut().get_mut::<AestheticDemand>(pop).unwrap();
        demand.timer = 0;

        app.update();

        let mood = app.world().get::<XenoMood>(pop).unwrap();
        assert!(mood.level < 100.0, "Mood should drop due to aesthetic deprivation");
        assert!(app.world().get::<AestheticDeprivation>(pop).is_some(), "crate::layer1::entities::pop::Pop should gain deprivation component");
    }
}
