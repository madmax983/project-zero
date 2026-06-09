use bevy::prelude::*;
#[cfg(test)]
use crate::layer1::building::Building;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::social::morale::{MoodModifier, Morale};
use rand::Rng;

pub struct HostageProtocolPlugin;

impl Plugin for HostageProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                hostage_protocol_suppression_system,
                hostage_protocol_malfunction_system,
                defuse_countdown_system,
                hostage_protocol_detonation_system,
            ),
        );
    }
}

#[derive(Component)]
pub struct HostageProtocol {
    pub is_active: bool,
    pub suppression_power: f32,
    pub malfunction_chance: f32,
}

#[derive(Component)]
pub struct SelfDestructCountdown {
    pub timer: Timer,
}

pub fn hostage_protocol_suppression_system(
    protocol_query: Query<(&HostageProtocol, &GridPosition)>,
    mut pop_query: Query<(&mut Morale, &GridPosition), With<Pop>>,
) {
    for (protocol, building_pos) in protocol_query.iter() {
        if protocol.is_active && protocol.suppression_power > 0.0 {
            for (mut morale, pop_pos) in pop_query.iter_mut() {
                // Apply only to Pops within a certain radius (e.g., 10 units)
                if building_pos.distance_chebyshev(*pop_pos) <= 10 {
                    let mut found = false;
                    for modifier in &mut morale.modifiers {
                        if modifier.label == "Hostage Suppression" {
                            modifier.duration = 1;
                            modifier.value = protocol.suppression_power;
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        morale.modifiers.push(MoodModifier {
                            label: "Hostage Suppression".to_string(),
                            value: protocol.suppression_power, // Adds to Morale (which lowers Unrest globally)
                            duration: 1, // Applies only while active and near
                        });
                    }
                }
            }
        }
    }
}

pub fn hostage_protocol_malfunction_system(
    mut commands: Commands,
    time: Res<Time>,
    protocol_query: Query<(Entity, &HostageProtocol), Without<SelfDestructCountdown>>,
) {
    let mut rng = rand::thread_rng();
    let dt = time.delta_secs();
    for (entity, protocol) in protocol_query.iter() {
        if protocol.is_active {
            // Scale chance by delta time
            let chance_per_frame = protocol.malfunction_chance * dt;
            if rng.gen::<f32>() < chance_per_frame {
                commands.entity(entity).insert(SelfDestructCountdown {
                    timer: Timer::from_seconds(60.0, TimerMode::Once),
                });
            }
        }
    }
}

pub fn defuse_countdown_system(
    mut commands: Commands,
    countdown_query: Query<(Entity, &GridPosition), With<SelfDestructCountdown>>,
    pop_query: Query<&GridPosition, With<Pop>>,
) {
    // Pops physically near the malfunctioning building (e.g., within radius 2) can defuse it
    for (entity, building_pos) in countdown_query.iter() {
        for pop_pos in pop_query.iter() {
            if building_pos.distance_chebyshev(*pop_pos) <= 2 {
                commands.entity(entity).remove::<SelfDestructCountdown>();
                break;
            }
        }
    }
}

pub fn hostage_protocol_detonation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut countdown_query: Query<(Entity, &mut SelfDestructCountdown, &GridPosition)>,
    pop_query: Query<(Entity, &GridPosition), With<Pop>>,
) {
    for (entity, mut countdown, building_pos) in countdown_query.iter_mut() {
        countdown.timer.tick(time.delta());
        if countdown.timer.finished() {
            // Explode: Despawn building and nearby pops (radius 10)
            commands.entity(entity).despawn();
            for (pop_entity, pop_pos) in pop_query.iter() {
                if building_pos.distance_chebyshev(*pop_pos) <= 10 {
                    commands.entity(pop_entity).despawn();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::BuildingType;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource::<Time>(Time::default());
        app.add_plugins(HostageProtocolPlugin);
        app
    }

    #[test]
    fn test_hostage_protocol_suppresses_unrest() {
        let mut app = setup_app();

        // Spawn a building with a Hostage Protocol
        app.world_mut().spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 0, y: 0 },
            HostageProtocol {
                is_active: true,
                suppression_power: 0.5,
                malfunction_chance: 0.01,
            },
        ));

        // Spawn a Pop near the building
        let pop_near = app.world_mut().spawn((
            Pop,
            GridPosition { x: 5, y: 5 }, // Distance 5
            Morale::default(),
        )).id();

        // Spawn a Pop far from the building
        let pop_far = app.world_mut().spawn((
            Pop,
            GridPosition { x: 20, y: 20 }, // Distance 20
            Morale::default(),
        )).id();

        app.update();

        // The unrest should be artificially suppressed for near pop
        let updated_near = app.world().get::<Morale>(pop_near).unwrap();
        assert_eq!(updated_near.modifiers.len(), 1, "Near pop should be suppressed by Hostage Protocol");
        assert_eq!(updated_near.modifiers[0].value, 0.5);

        // The far pop should not be suppressed
        let updated_far = app.world().get::<Morale>(pop_far).unwrap();
        assert_eq!(updated_far.modifiers.len(), 0, "Far pop should NOT be suppressed by Hostage Protocol");

        // Running update again should not duplicate modifier
        app.update();
        let updated_near = app.world().get::<Morale>(pop_near).unwrap();
        assert_eq!(updated_near.modifiers.len(), 1, "Should not duplicate modifier");
    }

    #[test]
    fn test_hostage_protocol_malfunction_triggers_countdown() {
        let mut app = setup_app();

        // Spawn a building with a massive malfunction chance for the test
        let building = app.world_mut().spawn((
            Building { building_type: BuildingType::Housing },
            HostageProtocol {
                is_active: true,
                suppression_power: 0.5,
                malfunction_chance: 100000.0, // High chance so it procs even with dt
            },
        )).id();

        // Advance time a lot to ensure it triggers
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(1));


        app.update();

        // The building should now have a SelfDestructCountdown
        assert!(app.world().get::<SelfDestructCountdown>(building).is_some(), "Malfunction should trigger self-destruct countdown");
    }

    #[test]
    fn test_defuse_countdown_system() {
        let mut app = setup_app();

        // Spawn a building with a countdown
        let building = app.world_mut().spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 0, y: 0 },
            SelfDestructCountdown {
                timer: Timer::from_seconds(60.0, TimerMode::Once),
            },
        )).id();

        // Defusal should not happen if pop is far
        let pop_far = app.world_mut().spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
        )).id();

        app.update();
        assert!(app.world().get::<SelfDestructCountdown>(building).is_some(), "Should not defuse if far");

        // Move pop close
        app.world_mut().get_mut::<GridPosition>(pop_far).unwrap().x = 1;
        app.world_mut().get_mut::<GridPosition>(pop_far).unwrap().y = 1;

        app.update();
        assert!(app.world().get::<SelfDestructCountdown>(building).is_none(), "Should defuse if near");
    }

    #[test]
    fn test_hostage_protocol_detonation_system() {
        let mut app = setup_app();

        let mut timer = Timer::from_seconds(1.0, TimerMode::Once);
        // Set it nearly finished
        timer.tick(std::time::Duration::from_millis(999));

        // Spawn a building with a countdown about to pop
        let building = app.world_mut().spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 0, y: 0 },
            SelfDestructCountdown {
                timer,
            },
        )).id();

        // Spawn a near pop
        let pop_near = app.world_mut().spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
        )).id();

        // Spawn a far pop
        let pop_far = app.world_mut().spawn((
            Pop,
            GridPosition { x: 20, y: 20 },
        )).id();

        // Advance time and update
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(10));

        app.update();

        // Building should be despawned
        assert!(app.world().get_entity(building).is_err(), "Building should be despawned");

        // Near pop should be despawned
        assert!(app.world().get_entity(pop_near).is_err(), "Near pop should be despawned");

        // Far pop should survive
        assert!(app.world().get_entity(pop_far).is_ok(), "Far pop should survive");
    }
}
