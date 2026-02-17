//! Echoes of the Past (Whispering Walls)
//!
//! Adds an emotional persistence layer to the map. Events leave "Echoes" that affect
//! the mood of Pops standing in that location later.
//!
//! - **Screams:** Left by death. Cause "Heard ghostly screams" (-Mood).
//! - **Laughter:** Left by crowded taverns. Cause "Felt a warm presence" (+Mood).

#![allow(clippy::cast_precision_loss)]
use crate::layer1::funeral::Corpse;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::pop::Pop;
use crate::layer1::social::Tavern;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// The type of emotional residue left behind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EchoType {
    /// Caused by death. Negative impact.
    Scream,
    /// Caused by joy/parties. Positive impact.
    Laughter,
}

/// A specific echo instance.
#[derive(Debug, Clone)]
pub struct Echo {
    /// The type of echo.
    pub echo_type: EchoType,
    /// Current intensity (decays over time).
    pub intensity: f32,
    /// Tick when created or last refreshed.
    pub created_at: u64,
}

/// Resource storing the spatial map of echoes.
#[derive(Resource, Default)]
pub struct EchoMap {
    /// Map from (x, y) to the Echo present there.
    pub echoes: HashMap<(i32, i32), Echo>,
}

impl EchoMap {
    /// Adds or updates an echo at the given location.
    pub fn add(&mut self, x: i32, y: i32, echo_type: EchoType, intensity: f32, tick: u64) {
        let key = (x, y);
        match self.echoes.get_mut(&key) {
            Some(existing) => {
                if existing.echo_type == echo_type {
                    // Reinforce existing echo
                    existing.intensity = (existing.intensity + intensity).min(2.0);
                    existing.created_at = tick;
                } else if intensity > existing.intensity {
                    // Overwrite if new emotion is stronger
                    *existing = Echo {
                        echo_type,
                        intensity,
                        created_at: tick,
                    };
                }
            }
            None => {
                self.echoes.insert(
                    key,
                    Echo {
                        echo_type,
                        intensity,
                        created_at: tick,
                    },
                );
            }
        }
    }

    /// Decays all echoes and removes faded ones.
    pub fn decay(&mut self) {
        // Decay rate: 0.001 per tick. 1.0 intensity lasts 1000 ticks.
        const DECAY_RATE: f32 = 0.001;
        self.echoes.retain(|_, echo| {
            echo.intensity -= DECAY_RATE;
            echo.intensity > 0.0
        });
    }
}

/// System to decay echoes over time.
pub fn update_echoes_system(mut echo_map: Option<ResMut<EchoMap>>) {
    if let Some(map) = echo_map.as_mut() {
        map.decay();
    }
}

/// System that creates "Scream" echoes when corpses appear.
pub fn absorb_death_echoes_system(
    mut echo_map: Option<ResMut<EchoMap>>,
    time: Res<SimulationTime>,
    // Detect newly added Corpse components
    query: Query<&GridPosition, Added<Corpse>>,
) {
    if let Some(map) = echo_map.as_mut() {
        for pos in &query {
            // High intensity scream
            map.add(pos.x, pos.y, EchoType::Scream, 1.0, time.tick);
        }
    }
}

/// System that creates "Laughter" echoes from busy taverns.
pub fn absorb_joy_echoes_system(
    mut echo_map: Option<ResMut<EchoMap>>,
    time: Res<SimulationTime>,
    query: Query<(&Tavern, &GridPosition)>,
) {
    if let Some(map) = echo_map.as_mut() {
        for (tavern, pos) in &query {
            // If tavern is bustling (more than 3 visitors)
            if tavern.visitors.len() > 3 {
                // Low intensity but constant accumulation
                map.add(pos.x, pos.y, EchoType::Laughter, 0.05, time.tick);
            }
        }
    }
}

/// System that applies mood modifiers to pops standing in echoes.
pub fn apply_echo_effects_system(
    echo_map: Option<Res<EchoMap>>,
    mut pops: Query<(&GridPosition, &mut Morale), With<Pop>>,
) {
    // Threshold to feel the echo
    const THRESHOLD: f32 = 0.5;

    let Some(map) = echo_map else { return };

    for (pos, mut morale) in &mut pops {
        if let Some(echo) = map.echoes.get(&(pos.x, pos.y)) {
            if echo.intensity < THRESHOLD {
                continue;
            }

            match echo.echo_type {
                EchoType::Scream => {
                    // Avoid spamming if already present?
                    // MoodModifier doesn't unique-check by label automatically, but
                    // Morale system might stack them. We'll add a short duration one.
                    // To prevent stack overflow, maybe check if one exists?
                    // For now, let's just add a small short one.
                    if !morale
                        .modifiers
                        .iter()
                        .any(|m| m.label == "Heard ghostly screams")
                    {
                        morale.add_modifier(MoodModifier {
                            label: "Heard ghostly screams".to_string(),
                            value: -0.05 * echo.intensity,
                            duration: 50, // Short duration, refreshes while standing there
                        });
                    }
                }
                EchoType::Laughter => {
                    if !morale
                        .modifiers
                        .iter()
                        .any(|m| m.label == "Felt a warm presence")
                    {
                        morale.add_modifier(MoodModifier {
                            label: "Felt a warm presence".to_string(),
                            value: 0.05 * echo.intensity,
                            duration: 50,
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_echo_map_decay() {
        let mut map = EchoMap::default();
        map.add(0, 0, EchoType::Scream, 0.002, 0);

        map.decay(); // 0.001 left
        assert!(map.echoes.contains_key(&(0, 0)));
        assert!((map.echoes.get(&(0, 0)).unwrap().intensity - 0.001).abs() < f32::EPSILON);

        map.decay(); // 0.0 left -> removed
        assert!(!map.echoes.contains_key(&(0, 0)));
    }

    #[test]
    fn test_echo_overwrite_logic() {
        let mut map = EchoMap::default();
        // Add weak laughter
        map.add(0, 0, EchoType::Laughter, 0.5, 0);
        // Add strong scream
        map.add(0, 0, EchoType::Scream, 1.0, 10);

        let echo = map.echoes.get(&(0, 0)).unwrap();
        assert_eq!(echo.echo_type, EchoType::Scream);
        assert!((echo.intensity - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_echo_reinforce_logic() {
        let mut map = EchoMap::default();
        map.add(0, 0, EchoType::Scream, 0.5, 0);
        map.add(0, 0, EchoType::Scream, 0.5, 10);

        let echo = map.echoes.get(&(0, 0)).unwrap();
        assert_eq!(echo.echo_type, EchoType::Scream);
        assert!((echo.intensity - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_absorb_death_echoes() {
        let mut world = World::new();
        world.insert_resource(EchoMap::default());
        world.insert_resource(SimulationTime::default());

        // Spawn a new Corpse
        world.spawn((
            Corpse {
                name: "Bob".into(),
                decay: 0.0,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Need to simulate "Added<Corpse>" behavior?
        // In unit tests, Added query filters work if we run the schedule.
        let mut schedule = Schedule::default();
        schedule.add_systems(absorb_death_echoes_system);
        schedule.run(&mut world);

        let map = world.resource::<EchoMap>();
        assert!(map.echoes.contains_key(&(5, 5)));
        let echo = map.echoes.get(&(5, 5)).unwrap();
        assert_eq!(echo.echo_type, EchoType::Scream);
        assert!(echo.intensity >= 1.0);
    }

    #[test]
    fn test_apply_echo_effects() {
        let mut world = World::new();
        let mut map = EchoMap::default();
        map.add(0, 0, EchoType::Scream, 1.0, 0);
        world.insert_resource(map);

        let pop = world
            .spawn((Pop, GridPosition { x: 0, y: 0 }, Morale::default()))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_echo_effects_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(
            morale
                .modifiers
                .iter()
                .any(|m| m.label == "Heard ghostly screams")
        );
        let modifier = morale
            .modifiers
            .iter()
            .find(|m| m.label == "Heard ghostly screams")
            .unwrap();
        assert!(modifier.value < 0.0);
    }
}
