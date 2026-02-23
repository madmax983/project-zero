//! Genius Loci: The Spirit of the Place.
//!
//! Stores spatial memories of significant events (Death, Celebration, etc.)
//! on the map, affecting the mood of pops who pass through.

use bevy_ecs::prelude::*;
use crate::layer1::health::Dead;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::pop::Pop;
use crate::shared::time::SimulationTime;

/// Types of loci that can form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocusType {
    /// A place where someone died. Causes unease.
    Tragedy,
    /// A place of great joy or celebration. Causes inspiration.
    Joy,
    /// A mysterious or strange location. Causes curiosity or fear.
    Mystery,
}

/// A single locus of memory on the map.
#[derive(Debug, Clone)]
pub struct Locus {
    /// The type of emotional residue.
    pub locus_type: LocusType,
    /// Intensity from 0.0 to 1.0.
    pub intensity: f32,
    /// The tick when this locus was formed.
    pub formed_at_tick: u64,
}

/// Resource storing the spatial map of loci.
#[derive(Resource)]
pub struct LociMap {
    /// Width of the map.
    pub width: usize,
    /// Height of the map.
    pub height: usize,
    /// Flat vector of loci.
    pub loci: Vec<Option<Locus>>,
}

impl Default for LociMap {
    fn default() -> Self {
        Self::new(80, 50)
    }
}

impl LociMap {
    /// Creates a new empty LociMap.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            loci: vec![None; width * height],
        }
    }

    /// Gets a reference to the locus at the given position.
    #[must_use]
    pub fn get(&self, x: i32, y: i32) -> Option<&Locus> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        self.loci[(y as usize) * self.width + (x as usize)].as_ref()
    }

    /// Sets a locus at the given position.
    pub fn set(&mut self, x: i32, y: i32, locus: Locus) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        self.loci[(y as usize) * self.width + (x as usize)] = Some(locus);
    }

    /// Clears a locus at the given position.
    pub fn clear(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        self.loci[(y as usize) * self.width + (x as usize)] = None;
    }
}

/// System to record death locations as Tragedy loci.
/// Must run after `handle_pop_death_system` but before `despawn_dead_entities_system`.
pub fn record_death_loci_system(
    mut loci_map: ResMut<LociMap>,
    time: Res<SimulationTime>,
    // Query entities that just died and have a position
    query: Query<(&GridPosition, &Dead), Added<Dead>>,
) {
    for (pos, _) in &query {
        // Create a new Tragedy locus
        let locus = Locus {
            locus_type: LocusType::Tragedy,
            intensity: 1.0, // Maximum intensity at moment of death
            formed_at_tick: time.tick,
        };

        // If there's already a locus, we could merge them or overwrite.
        // For simplicity, overwrite or max out intensity if same type.
        if let Some(existing) = loci_map.get(pos.x, pos.y) {
            if existing.locus_type == LocusType::Tragedy {
                // Refresh intensity
                loci_map.set(pos.x, pos.y, locus);
            } else {
                // Overwrite (newest wins)
                loci_map.set(pos.x, pos.y, locus);
            }
        } else {
            loci_map.set(pos.x, pos.y, locus);
        }
    }
}

/// System to decay loci intensity over time.
pub fn update_loci_system(mut loci_map: ResMut<LociMap>) {
    for locus_opt in loci_map.loci.iter_mut() {
        if let Some(locus) = locus_opt {
            // Decay by 0.0001 per tick
            // 1.0 -> 0.0 in 10,000 ticks.
            locus.intensity -= 0.0001;
            if locus.intensity <= 0.0 {
                *locus_opt = None;
            }
        }
    }
}

/// System to apply emotional residue effects to pops.
pub fn apply_loci_effects_system(
    loci_map: Res<LociMap>,
    mut query: Query<(&GridPosition, &mut Morale), With<Pop>>,
) {
    for (pos, mut morale) in &mut query {
        if let Some(locus) = loci_map.get(pos.x, pos.y) {
            // Check intensity threshold (e.g. > 0.1)
            if locus.intensity > 0.1 {
                let (label, base_value, duration) = match locus.locus_type {
                    LocusType::Tragedy => ("Felt a chill", -0.05, 20),
                    LocusType::Joy => ("Echoes of laughter", 0.05, 20),
                    LocusType::Mystery => ("Intrigued by mystery", 0.02, 20),
                };

                let value = base_value * locus.intensity;

                if let Some(existing) = morale.modifiers.iter_mut().find(|m| m.label == label) {
                    existing.duration = duration;
                    existing.value = value;
                } else {
                    morale.add_modifier(MoodModifier {
                        label: label.to_string(),
                        value,
                        duration,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_record_death_loci() {
        let mut world = World::new();
        world.insert_resource(LociMap::new(10, 10));
        world.insert_resource(SimulationTime::default());

        // Spawn a pop that just died
        world.spawn((
            GridPosition { x: 5, y: 5 },
            Dead, // Added<Dead> trigger
        ));

        // Run system
        world.run_system_once(record_death_loci_system).unwrap();

        let map = world.resource::<LociMap>();
        let locus = map.get(5, 5).expect("Should have created a locus");

        assert_eq!(locus.locus_type, LocusType::Tragedy);
        assert!((locus.intensity - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_update_loci_decay() {
        let mut world = World::new();
        let mut map = LociMap::new(10, 10);

        map.set(0, 0, Locus {
            locus_type: LocusType::Joy,
            intensity: 0.00005, // Should decay to 0
            formed_at_tick: 0,
        });
        map.set(1, 1, Locus {
            locus_type: LocusType::Joy,
            intensity: 1.0,
            formed_at_tick: 0,
        });

        world.insert_resource(map);

        world.run_system_once(update_loci_system).unwrap();

        let map = world.resource::<LociMap>();
        assert!(map.get(0, 0).is_none());
        assert!(map.get(1, 1).is_some());
        assert!(map.get(1, 1).unwrap().intensity < 1.0);
    }

    #[test]
    fn test_apply_loci_effects() {
        let mut world = World::new();
        let mut map = LociMap::new(10, 10);

        map.set(5, 5, Locus {
            locus_type: LocusType::Tragedy,
            intensity: 1.0,
            formed_at_tick: 0,
        });

        world.insert_resource(map);

        let pop = world
            .spawn((Pop, GridPosition { x: 5, y: 5 }, Morale::default()))
            .id();

        world.run_system_once(apply_loci_effects_system).unwrap();

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(!morale.modifiers.is_empty());
        assert_eq!(morale.modifiers[0].label, "Felt a chill");
        assert_eq!(morale.modifiers[0].duration, 20);
        assert!((morale.modifiers[0].value + 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_loci_effects_refreshes() {
        let mut world = World::new();
        let mut map = LociMap::new(10, 10);
        map.set(5, 5, Locus {
            locus_type: LocusType::Tragedy,
            intensity: 1.0,
            formed_at_tick: 0,
        });
        world.insert_resource(map);

        let mut morale = Morale::default();
        morale.add_modifier(MoodModifier {
            label: "Felt a chill".to_string(),
            value: -0.01,
            duration: 1,
        });

        let pop = world
            .spawn((Pop, GridPosition { x: 5, y: 5 }, morale))
            .id();

        world.run_system_once(apply_loci_effects_system).unwrap();

        let morale = world.get::<Morale>(pop).unwrap();
        assert_eq!(morale.modifiers.len(), 1); // Should reuse existing
        assert_eq!(morale.modifiers[0].duration, 20); // Should refresh duration
        assert!((morale.modifiers[0].value + 0.05).abs() < f32::EPSILON); // Should update value
    }
}
