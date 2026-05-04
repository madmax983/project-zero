#![allow(clippy::type_complexity)]
use bevy_ecs::prelude::*;

use crate::layer1::map::GridPosition;
use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::mind::utility_types::{PopAction, ActionType};
use crate::layer1::social::morale::{Morale, MoodModifier};


#[derive(Component, Debug, Clone)]
pub struct EchoSource {
    pub event_id: String,
    pub intensity: f32, // Fade over years
    pub frequency: u32, // Ticks between manifestations
    pub timer: u32,
    pub event_type: EchoType,
}

#[derive(Component, Debug, Clone)]
pub struct Echo {
    pub event_type: EchoType,
    pub duration: u32, // Ticks to exist
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EchoType {
    Tragedy, // Deaths -> Stress
    Triumph, // Success -> Inspiration
    Mystery, // Discovery -> Curiosity
}

pub fn create_echo_source_system(
    mut commands: Commands,
    mut events: EventReader<AddChronicleEvent>,
) {
    for event in events.read() {
        if let Some(loc) = event.location {
            if matches!(event.importance, EventImportance::Major | EventImportance::Legendary) {
                commands.spawn((
                    EchoSource {
                        event_id: event.id.clone().unwrap_or_else(|| event.text.clone()),
                        intensity: 1.0,
                        frequency: 10,
                        timer: 0,
                        event_type: determine_type(event),
                    },
                    loc,
                ));
            }
        }
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

pub fn echo_reaction_system(
    echoes: Query<(&Echo, &GridPosition)>,
    mut pops: Query<(&GridPosition, &mut PopAction, &mut Morale), Without<Echo>>,
) {
    for (echo, echo_pos) in &echoes {
        for (pop_pos, mut action, mut morale) in &mut pops {
            if pop_pos.distance_chebyshev(*echo_pos) <= 2u32 {
                action.current = ActionType::WatchEcho;
                action.ticks_committed = 10;

                let (label, value) = match echo.event_type {
                    EchoType::Tragedy => ("Witnessed Tragedy".to_string(), -0.05),
                    EchoType::Triumph => ("Witnessed Triumph".to_string(), 0.05),
                    EchoType::Mystery => ("Witnessed Mystery".to_string(), 0.01),
                };
                morale.add_modifier(MoodModifier { label: label.to_string(), value, duration: 100 });
            }
        }
    }
}

pub fn despawn_echo_system(
    mut commands: Commands,
    mut echoes: Query<(Entity, &mut Echo)>,
) {
    for (entity, mut echo) in &mut echoes {
        if echo.duration > 0 {
            echo.duration -= 1;
        } else {
            commands.entity(entity).despawn();
        }
    }
}

fn determine_type(event: &AddChronicleEvent) -> EchoType {
    let lower = event.text.to_lowercase();
    if lower.contains("died") || lower.contains("disaster") || lower.contains("killed") {
        EchoType::Tragedy
    } else if lower.contains("miracle") || lower.contains("success") || lower.contains("completed") {
        EchoType::Triumph
    } else {
        EchoType::Mystery
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
    use crate::layer1::map::GridPosition;
    use crate::layer1::mind::utility_types::{PopAction, ActionType};
    use crate::layer1::social::morale::Morale;

    #[test]
    fn test_high_importance_event_creates_echo_source() {
        let mut world = bevy_ecs::prelude::World::new();
        world.init_resource::<Events<AddChronicleEvent>>();
        let mut schedule = bevy_ecs::prelude::Schedule::default();
        schedule.add_systems(create_echo_source_system);

        world.send_event(AddChronicleEvent {
            text: "Miner Bob died here.".to_string(),
            importance: EventImportance::Major,
            location: Some(GridPosition { x: 10, y: 10 }),
            ..Default::default()
        });

        schedule.run(&mut world);

        let sources = world.query::<(Entity, &EchoSource)>().iter(&world).collect::<Vec<_>>();
        assert_eq!(sources.len(), 1);
        let source_pos = world.get::<GridPosition>(sources[0].0).unwrap();
        assert_eq!(*source_pos, GridPosition { x: 10, y: 10 });
    }

    #[test]
    fn test_echo_manifestation_cycle() {
        let mut world = bevy_ecs::prelude::World::new();
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

        let mut schedule = bevy_ecs::prelude::Schedule::default();
        schedule.add_systems(manifest_echo_system);

        schedule.run(&mut world);

        let echoes = world.query::<&Echo>().iter(&world).collect::<Vec<_>>();
        assert_eq!(echoes.len(), 1);
        let echo = echoes[0];
        assert_eq!(echo.event_type, EchoType::Tragedy);
    }

    #[test]
    fn test_pop_reaction_to_echo() {
        let mut world = bevy_ecs::prelude::World::new();
        let echo_pos = GridPosition { x: 2, y: 2 };
        world.spawn((
            Echo { event_type: EchoType::Tragedy, duration: 5 },
            echo_pos,
        ));

        let pop = world.spawn((
            GridPosition { x: 2, y: 3 },
            PopAction::default(),
            Morale { value: 0.5, modifiers: vec![] },
        )).id();

        let mut schedule = bevy_ecs::prelude::Schedule::default();
        schedule.add_systems(echo_reaction_system);
        schedule.run(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::WatchEcho);

        let morale = world.get::<Morale>(pop).unwrap();
        assert_eq!(morale.modifiers.len(), 1);
        assert_eq!(morale.modifiers[0].value, -0.05);
    }
}
