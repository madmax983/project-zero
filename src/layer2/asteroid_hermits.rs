use bevy::prelude::*;
use crate::layer1::entities::pop::Pop;
use crate::layer2::orbit::asteroid_claims::Asteroid;

#[derive(Component)]
pub struct Dissatisfaction {
    pub level: f32,
}

#[derive(Component)]
pub struct SocialNeed {
    pub value: f32,
}

#[derive(Component)]
pub struct Uncolonized;

#[derive(Component)]
pub struct HermitOutpost;

#[derive(Component, Default)]
pub struct DiscoveryProgress {
    pub progress: f32,
}

#[derive(Event)]
pub struct DiscoveryEvent {
    pub outpost: Entity,
    pub item_type: String, // E.g., "Rare Mineral", "Anomalous Artifact"
}

const HERMIT_EXODUS_DISSATISFACTION_THRESHOLD: f32 = 90.0;
const HERMIT_EXODUS_SOCIAL_NEED_THRESHOLD: f32 = 20.0;
const DISCOVERY_PROGRESS_INCREMENT: f32 = 1.0;
const DISCOVERY_PROGRESS_THRESHOLD: f32 = 100.0;

pub fn evaluate_hermit_exodus_system(
    mut commands: Commands,
    pops: Query<(Entity, &Dissatisfaction, &SocialNeed), With<Pop>>,
    asteroids: Query<Entity, (With<Asteroid>, With<Uncolonized>)>,
) {
    let mut available_asteroids: Vec<Entity> = asteroids.iter().collect();

    for (pop_entity, dissatisfaction, social_need) in pops.iter() {
        if dissatisfaction.level > HERMIT_EXODUS_DISSATISFACTION_THRESHOLD && social_need.value < HERMIT_EXODUS_SOCIAL_NEED_THRESHOLD {
            if let Some(asteroid_entity) = available_asteroids.pop() {
                // Remove the pop
                commands.entity(pop_entity).despawn();

                // Convert the asteroid
                commands.entity(asteroid_entity).remove::<Uncolonized>();
                commands.entity(asteroid_entity).insert((
                    HermitOutpost,
                    DiscoveryProgress::default(),
                ));
            }
        }
    }
}

pub fn process_hermit_discoveries_system(
    mut outposts: Query<(Entity, &mut DiscoveryProgress), With<HermitOutpost>>,
    mut event_writer: EventWriter<DiscoveryEvent>,
) {
    for (entity, mut discovery) in outposts.iter_mut() {
        discovery.progress += DISCOVERY_PROGRESS_INCREMENT; // Fixed increment for minimal implementation

        if discovery.progress >= DISCOVERY_PROGRESS_THRESHOLD {
            discovery.progress = 0.0;
            event_writer.send(DiscoveryEvent {
                outpost: entity,
                item_type: "Anomalous Artifact".to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase Test Setup
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<DiscoveryEvent>();
        app.add_systems(Update, (
            evaluate_hermit_exodus_system,
            process_hermit_discoveries_system,
        ));
        app
    }

    #[test]
    fn test_dissatisfied_pop_becomes_hermit() {
        let mut app = setup_app();

        // Spawn an uncolonized asteroid
        let asteroid = app.world_mut().spawn((
            Asteroid,
            Uncolonized,
        )).id();

        // Spawn a highly dissatisfied pop with low social need
        let pop = app.world_mut().spawn((
            Pop,
            Dissatisfaction { level: 95.0 }, // High dissatisfaction
            SocialNeed { value: 10.0 },      // Low social need
        )).id();

        app.update();

        // Assert: Pop is removed/converted and Asteroid becomes HermitOutpost
        assert!(app.world().get::<Pop>(pop).is_none(), "Pop should leave the colony");

        let outpost = app.world().get::<HermitOutpost>(asteroid);
        assert!(outpost.is_some(), "Asteroid should now be a Hermit Outpost");
        assert!(app.world().get::<Uncolonized>(asteroid).is_none(), "Asteroid should no longer be Uncolonized");
    }

    #[test]
    fn test_satisfied_pop_does_not_become_hermit() {
        let mut app = setup_app();

        let asteroid = app.world_mut().spawn((
            Asteroid,
            Uncolonized,
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            Dissatisfaction { level: 20.0 }, // Low dissatisfaction
            SocialNeed { value: 10.0 },
        )).id();

        app.update();

        assert!(app.world().get::<Pop>(pop).is_some(), "Satisfied Pop should stay");
        assert!(app.world().get::<HermitOutpost>(asteroid).is_none(), "Asteroid should remain uncolonized");
    }

    #[test]
    fn test_hermit_outpost_discovers_artifacts() {
        let mut app = setup_app();

        // Spawn a Hermit Outpost with high discovery progress
        app.world_mut().spawn((
            HermitOutpost,
            DiscoveryProgress { progress: 100.0 }, // Ready to discover
        ));

        app.update();

        // Assert: DiscoveryEvent should be emitted
        let events = app.world().resource::<Events<DiscoveryEvent>>();
        let mut reader = events.get_cursor();
        assert!(reader.read(events).next().is_some(), "Hermit Outpost should generate a DiscoveryEvent");
    }
}
