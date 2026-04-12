use bevy_ecs::prelude::*;
use bevy::math::Vec2;

#[derive(Component)]
pub struct Storm {
    pub position: Vec2,
    pub velocity: Vec2,
    pub strength: f32,
}

#[derive(Component)]
pub struct AggroTarget {
    pub position: Vec2,
    pub energy_emission: f32,
    pub heat_signature: f32,
}

#[derive(Event)]
pub struct StormImpactEvent {
    pub storm: Entity,
    pub target: Entity,
    pub damage: f32,
}

pub fn weather_movement_system(
    mut storms: Query<(Entity, &mut Storm)>,
    targets: Query<(Entity, &AggroTarget)>,
    mut impact_events: EventWriter<StormImpactEvent>,
) {
    let aggro_threshold = 100.0;

    for (storm_entity, mut storm) in storms.iter_mut() {
        let mut best_target: Option<(Entity, f32, Vec2)> = None;

        for (target_entity, target) in targets.iter() {
            let emission = target.energy_emission + target.heat_signature;
            if emission > aggro_threshold {
                if let Some((_, best_emission, _)) = best_target {
                    if emission > best_emission {
                        best_target = Some((target_entity, emission, target.position));
                    }
                } else {
                    best_target = Some((target_entity, emission, target.position));
                }
            }
        }

        if let Some((target_entity, _, target_pos)) = best_target {
            let dir = target_pos - storm.position;
            let dist = dir.length();
            if dist > 0.1 {
                let attract = dir.normalize() * 0.5; // Simple attraction
                storm.velocity += attract; // Basic steering
                if storm.velocity.length() > 2.0 {
                    storm.velocity = storm.velocity.normalize() * 2.0; // max speed
                }
            }

            if dist < 1.0 { // Impact threshold
                impact_events.send(StormImpactEvent {
                    storm: storm_entity,
                    target: target_entity,
                    damage: storm.strength * 10.0,
                });
            }
        }

        let vel = storm.velocity;
        storm.position += vel;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storm_pathfinding_towards_energy_spike() {
        // Arrange: Set up a Layer 2 weather grid with a Storm entity and a Layer 1 colony tile generating massive energy.
        let mut app = bevy_app::App::new();
        app.add_event::<StormImpactEvent>();
        app.add_systems(bevy_app::Update, weather_movement_system);

        let storm = app.world_mut().spawn(Storm {
            position: Vec2::new(0.0, 0.0),
            velocity: Vec2::new(0.0, 1.0), // Initially moving up
            strength: 1.0,
        }).id();

        app.world_mut().spawn(AggroTarget {
            position: Vec2::new(10.0, 0.0),
            energy_emission: 200.0, // Over threshold
            heat_signature: 0.0,
        });

        // Act: Advance simulation time to process weather movement.
        app.update();

        // Assert: The Storm's movement vector shifts towards the colony tile rather than following random or global wind patterns.
        let storm_data = app.world().get::<Storm>(storm).unwrap();
        assert!(storm_data.velocity.x > 0.0, "Storm should accelerate towards the target");
    }

    #[test]
    fn test_storm_ignores_low_energy() {
        // Arrange: Set up a storm and a colony running on stealth/low power mode.
        let mut app = bevy_app::App::new();
        app.add_event::<StormImpactEvent>();
        app.add_systems(bevy_app::Update, weather_movement_system);

        let storm = app.world_mut().spawn(Storm {
            position: Vec2::new(0.0, 0.0),
            velocity: Vec2::new(0.0, 1.0), // Initially moving up
            strength: 1.0,
        }).id();

        app.world_mut().spawn(AggroTarget {
            position: Vec2::new(10.0, 0.0),
            energy_emission: 10.0, // Under threshold (100)
            heat_signature: 0.0,
        });

        // Act: Advance simulation time.
        app.update();

        // Assert: The Storm continues on its natural path without homing in on the colony.
        let storm_data = app.world().get::<Storm>(storm).unwrap();
        assert_eq!(storm_data.velocity.x, 0.0, "Storm should not accelerate towards low energy target");
    }

    #[test]
    fn test_storm_impact_on_colony() {
        // Arrange: A storm positioned directly over the colony tile.
        let mut app = bevy_app::App::new();
        app.add_event::<StormImpactEvent>();
        app.add_systems(bevy_app::Update, weather_movement_system);

        let storm = app.world_mut().spawn(Storm {
            position: Vec2::new(0.0, 0.0),
            velocity: Vec2::new(0.0, 0.0),
            strength: 2.0,
        }).id();

        let colony = app.world_mut().spawn(AggroTarget {
            position: Vec2::new(0.5, 0.0), // Close enough to impact (< 1.0)
            energy_emission: 200.0,
            heat_signature: 0.0,
        }).id();

        // Act: Process the storm impact event.
        app.update();

        // Assert: The colony infrastructure takes damage, and outdoor activities are halted.
        let events = app.world().resource::<Events<StormImpactEvent>>();
        let mut reader = events.get_cursor();
        let impact_events: Vec<_> = reader.read(events).collect();

        assert_eq!(impact_events.len(), 1, "Impact event should be fired");
        assert_eq!(impact_events[0].storm, storm);
        assert_eq!(impact_events[0].target, colony);
        assert_eq!(impact_events[0].damage, 20.0);
    }
}
