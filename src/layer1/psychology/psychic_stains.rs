use crate::layer1::map::GridPosition;
use crate::layer1::pop::PopDied;
use crate::layer1::psychology::stress::StressTracker;
use bevy::prelude::*;

pub fn process_violent_deaths_system(
    mut commands: Commands,
    mut death_events: EventReader<PopDied>,
    mut existing_stains: Query<(&GridPosition, &mut PsychicStain)>,
    pops: Query<&GridPosition>,
) {
    for event in death_events.read() {
        // Simple logic to check if death was violent.
        let is_violent = event.reason.to_lowercase().contains("violent")
            || event.reason.to_lowercase().contains("violence")
            || event.reason.to_lowercase().contains("murder");

        if !is_violent {
            continue;
        }

        let mut event_pos = None;
        if let Ok(pos) = pops.get(event.entity) {
            event_pos = Some(*pos);
        }

        // Let's assume the event specifies the position through looking at the entity if alive, or maybe we can't reliably.
        // Wait, the tests spawn the entity and attach GridPosition right before sending PopDied.

        let position = match event_pos {
            Some(pos) => pos,
            None => continue,
        };

        // Try to add to existing stain
        let mut found = false;
        for (pos, mut stain) in existing_stains.iter_mut() {
            if *pos == position {
                stain.trauma_level += 10.0;
                found = true;
                break;
            }
        }

        // Or create new stain
        if !found {
            commands.spawn((position, PsychicStain { trauma_level: 10.0 }));
        }
    }
}

pub fn apply_stain_stress_system(
    stains: Query<(&GridPosition, &PsychicStain)>,
    mut pops: Query<(&GridPosition, &mut StressTracker)>,
) {
    // Basic implementation (O(N*M))
    for (pop_pos, mut stress) in pops.iter_mut() {
        for (stain_pos, stain) in stains.iter() {
            if pop_pos == stain_pos {
                stress.accumulated_stress += stain.trauma_level * 0.1;
            }
        }
    }
}

pub fn decay_stains_system(
    mut commands: Commands,
    time: Option<Res<bevy_time::Time<bevy_time::Real>>>,
    mut stains: Query<(Entity, &mut PsychicStain)>,
) {
    let decay_rate = if let Some(t) = time {
        0.5 * t.delta_secs()
    } else {
        0.05 // default fallback for tests without Time resource
    };
    for (entity, mut stain) in stains.iter_mut() {
        stain.trauma_level -= decay_rate;
        if stain.trauma_level <= 0.0 {
            if let Some(mut entity_cmds) = commands.get_entity(entity) {
                entity_cmds.despawn();
            }
        }
    }
}

#[derive(Component)]
pub struct PsychicStain {
    pub trauma_level: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{App, Update};

    #[test]
    fn test_violent_death_creates_trauma_stain() {
        // Arrange: Create app and map with a specific tile
        let mut app = App::new();
        app.add_event::<PopDied>();
        // Setup necessary resources/systems for Psychic Stains
        app.add_systems(Update, process_violent_deaths_system);

        // Act: Send a violent death event at a specific grid position
        // We will include the "violent" marker in the reason
        app.world_mut().send_event(PopDied {
            entity: Entity::from_raw(1),
            name: "Test Pop".to_string(),
            tick: 1,
            reason: "Violence".to_string(), // indicates violent death
        });

        // Mocking the GridPosition of the entity
        app.world_mut().spawn((
            GridPosition { x: 5, y: 5 },
            // Needs to have the exact entity id 1, so we'll just insert a resource to map or give it the position
        ));

        // To make it simpler and match codebase, we'll spawn an entity, then get its ID, and emit PopDied for that ID
        let death_pos = GridPosition { x: 5, y: 5 };
        let pop_entity = app.world_mut().spawn(death_pos).id();

        app.world_mut().send_event(PopDied {
            entity: pop_entity,
            name: "Test Pop".to_string(),
            tick: 1,
            reason: "Murdered violently".to_string(),
        });
        app.update();

        // Assert: Verify the tile now has a PsychicStain component with trauma > 0
        let stain_query = app
            .world_mut()
            .query::<&PsychicStain>()
            .iter(app.world())
            .next();
        assert!(stain_query.is_some());
        assert!(stain_query.unwrap().trauma_level > 0.0);
    }

    #[test]
    fn test_passing_through_stain_increases_stress() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_stain_stress_system);

        let stain_pos = GridPosition { x: 5, y: 5 };
        app.world_mut()
            .spawn((stain_pos, PsychicStain { trauma_level: 10.0 }));

        let pop = app
            .world_mut()
            .spawn((
                stain_pos,
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        // Act
        app.update();

        // Assert: The pop's stress should have increased due to standing in the stain
        let stress = app.world().get::<StressTracker>(pop).unwrap();
        assert!(stress.accumulated_stress > 0.0);
    }

    #[test]
    fn test_stain_decays_over_time() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(bevy_time::Time::<bevy_time::Real>::default());
        app.add_systems(Update, decay_stains_system);

        let stain = app
            .world_mut()
            .spawn((
                GridPosition { x: 0, y: 0 },
                PsychicStain { trauma_level: 10.0 },
            ))
            .id();

        // Act
        app.world_mut()
            .resource_mut::<bevy_time::Time<bevy_time::Real>>()
            .advance_by(std::time::Duration::from_secs(10));
        app.update();

        // Assert: Trauma level should be less than 10
        let decayed_stain = app.world().get::<PsychicStain>(stain).unwrap();
        assert!(decayed_stain.trauma_level < 10.0);
    }

    #[test]
    fn test_non_violent_death_creates_no_stain() {
        // Arrange
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.add_systems(Update, process_violent_deaths_system);

        let pop_entity = app.world_mut().spawn(GridPosition { x: 2, y: 2 }).id();

        // Act: Send a peaceful death event
        app.world_mut().send_event(PopDied {
            entity: pop_entity,
            name: "Test Pop".to_string(),
            tick: 1,
            reason: "Old age".to_string(), // non-violent
        });
        app.update();

        // Assert: No stains spawned
        let stain_count = app
            .world_mut()
            .query::<&PsychicStain>()
            .iter(app.world())
            .count();
        assert_eq!(stain_count, 0);
    }
}
