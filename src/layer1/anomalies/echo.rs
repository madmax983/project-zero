//! # Echoes of the Past
//!
//! Echoes are residual emotional or psychic impressions left behind by significant events.
//! Pops who wander near an Echo will be dazed and receive a morale modifier based on the echo's nature.

use crate::layer1::core::map::GridPosition;
use crate::layer1::entities::pop::Pop;
use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::layer1::social::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;

#[derive(Event, Debug, Clone)]
pub struct SpawnEchoSourceEvent {
    pub event_id: String,
    pub location: GridPosition,
    pub echo_type: EchoType,
}

#[derive(Component, Debug)]
pub struct EchoSource {
    pub event_id: String,
    pub intensity: f32, // Fade over years
    pub frequency: u32, // Ticks between manifestations
    pub timer: u32,
    pub event_type: EchoType,
}

#[derive(Component, Debug)]
pub struct Echo {
    pub event_type: EchoType,
    pub duration: u32, // Ticks to exist
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EchoType {
    Tragedy, // Deaths -> Stress (Morale Penalty)
    Triumph, // Success -> Inspiration (Morale Boost)
    Mystery, // Discovery -> Curiosity (Morale Boost)
}

pub fn create_echo_source_system(
    mut commands: Commands,
    mut events: EventReader<SpawnEchoSourceEvent>,
) {
    for event in events.read() {
        commands.spawn((
            EchoSource {
                event_id: event.event_id.clone(),
                intensity: 1.0,
                frequency: 1000,
                timer: 0,
                event_type: event.echo_type,
            },
            event.location,
        ));
    }
}

pub fn manifest_echo_system(
    mut commands: Commands,
    mut query: Query<(&mut EchoSource, &GridPosition)>,
) {
    for (mut source, pos) in &mut query {
        source.timer += 1;
        if source.timer >= source.frequency {
            source.timer = 0;
            commands.spawn((
                Echo {
                    event_type: source.event_type,
                    duration: 100,
                },
                *pos,
            ));
        }
    }
}

pub fn despawn_echo_system(mut commands: Commands, mut query: Query<(Entity, &mut Echo)>) {
    for (entity, mut echo) in &mut query {
        if echo.duration == 0 {
            commands.entity(entity).despawn();
        } else {
            echo.duration -= 1;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn echo_reaction_system(
    echoes: Query<(&Echo, &GridPosition)>,
    mut pops: Query<(&GridPosition, &mut PopAction, &mut Morale), (With<Pop>, Without<Echo>)>,
) {
    for (echo, echo_pos) in &echoes {
        for (pop_pos, mut action, mut morale) in &mut pops {
            if pop_pos.distance_chebyshev(*echo_pos) <= 2 {
                action.current = ActionType::Daze;
                action.ticks_committed = 10;
                match echo.event_type {
                    EchoType::Tragedy => morale.add_modifier(MoodModifier {
                        label: "Witnessed Tragic Echo".to_string(),
                        value: -0.1,
                        duration: 100,
                    }),
                    EchoType::Triumph => morale.add_modifier(MoodModifier {
                        label: "Witnessed Triumphant Echo".to_string(),
                        value: 0.1,
                        duration: 100,
                    }),
                    EchoType::Mystery => morale.add_modifier(MoodModifier {
                        label: "Witnessed Mysterious Echo".to_string(),
                        value: 0.05,
                        duration: 100,
                    }),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::core::map::GridPosition;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::mind::utility_types::{ActionType, PopAction};
    use crate::layer1::social::morale::Morale;

    #[test]
    fn test_high_importance_event_creates_echo_source() {
        let mut world = World::new();
        world.init_resource::<Events<SpawnEchoSourceEvent>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(create_echo_source_system);

        world.send_event(SpawnEchoSourceEvent {
            event_id: "evt_1".to_string(),
            location: GridPosition { x: 10, y: 10 },
            echo_type: EchoType::Tragedy,
        });

        schedule.run(&mut world);

        let mut query = world.query::<(Entity, &EchoSource)>();
        let entity = query.get_single(&world).unwrap().0;

        let source_pos = world.get::<GridPosition>(entity).unwrap();
        assert_eq!(*source_pos, GridPosition { x: 10, y: 10 });
    }

    #[test]
    fn test_echo_manifestation_cycle() {
        let mut world = World::new();
        world.spawn((
            EchoSource {
                event_id: "evt_1".to_string(),
                intensity: 1.0,
                frequency: 10,
                timer: 9,
                event_type: EchoType::Tragedy,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(manifest_echo_system);

        schedule.run(&mut world);

        let mut query = world.query::<&Echo>();
        let echo = query.get_single(&world).unwrap();
        assert_eq!(echo.event_type, EchoType::Tragedy);
    }

    #[test]
    fn test_pop_reaction_to_echo() {
        let mut world = World::new();

        let echo_pos = GridPosition { x: 2, y: 2 };
        world.spawn((
            Echo {
                event_type: EchoType::Tragedy,
                duration: 5,
            },
            echo_pos,
        ));

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 2, y: 3 },
                PopAction::default(),
                Morale::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(echo_reaction_system);

        schedule.run(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Daze);

        let morale = world.get::<Morale>(pop).unwrap();
        assert_eq!(morale.modifiers.len(), 1);
        assert_eq!(morale.modifiers[0].value, -0.1);
    }
}
