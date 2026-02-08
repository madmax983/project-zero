use crate::layer1::needs::Needs;
use crate::layer1::social::SocialBuff;
use bevy_ecs::prelude::*;

/// Types of memories a pop can acquire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    /// Witnessed a pop die.
    WitnessedDeath,
    /// Suffered from starvation (hunger = 0).
    StarvationTrauma,
    /// Ate a high quality meal.
    AteFineMeal,
    /// Won a combat encounter.
    WonFight,
}

impl MemoryType {
    /// Returns the mood impact (0.0 to 1.0 or negative).
    #[must_use]
    pub const fn base_mood_impact(&self) -> f32 {
        match self {
            Self::WitnessedDeath => -0.2,
            Self::StarvationTrauma => -0.15,
            Self::AteFineMeal => 0.1,
            Self::WonFight => 0.05,
        }
    }

    /// Returns the decay rate per tick.
    #[must_use]
    pub const fn decay_rate(&self) -> f32 {
        // Ticks to fade completely
        match self {
            Self::WitnessedDeath => 0.0005,              // Slow fade (2000 ticks)
            Self::StarvationTrauma => 0.001,             // Medium
            Self::AteFineMeal | Self::WonFight => 0.002, // Fast (500 ticks)
        }
    }
}

/// A specific instance of a memory on a pop.
#[derive(Debug, Clone)]
pub struct ActiveMemory {
    /// The type of memory.
    pub memory_type: MemoryType,
    /// The tick when it was added.
    pub added_at: u64,
    /// Current intensity (starts at 1.0, decays to 0.0).
    pub intensity: f32,
}

/// Component storing all active memories for a pop.
#[derive(Component, Default, Debug, Clone)]
pub struct Memories {
    /// List of active memories.
    pub items: Vec<ActiveMemory>,
}

impl Memories {
    /// Adds a new memory with full intensity.
    pub fn add(&mut self, memory_type: MemoryType, current_tick: u64) {
        self.items.push(ActiveMemory {
            memory_type,
            added_at: current_tick,
            intensity: 1.0,
        });
    }

    /// Decays all memories by the given number of ticks.
    /// Removes memories that have faded to 0 intensity.
    pub fn decay(&mut self, ticks_passed: u64) {
        #[allow(clippy::cast_precision_loss)]
        let ticks_f32 = ticks_passed as f32;
        for memory in &mut self.items {
            let decay = memory.memory_type.decay_rate() * ticks_f32;
            memory.intensity = (memory.intensity - decay).max(0.0);
        }
        self.items.retain(|m| m.intensity > 0.0);
    }
}

/// Calculates the effective morale including memory and social modifiers.
#[must_use]
pub fn calculate_effective_morale(
    needs: &Needs,
    memories: Option<&Memories>,
    social_buff: Option<&SocialBuff>,
) -> f32 {
    let base = needs.morale();
    let memory_modifier: f32 = memories.map_or(0.0, |m| {
        m.items
            .iter()
            .map(|i| i.memory_type.base_mood_impact() * i.intensity)
            .sum()
    });

    let social_modifier = social_buff.map_or(0.0, |s| s.value);

    (base + memory_modifier + social_modifier).clamp(0.0, 1.0)
}

/// System to decay memories every tick.
pub fn memory_decay_system(mut query: Query<&mut Memories>) {
    // Assuming this runs every tick
    query.par_iter_mut().for_each(|mut memories| {
        memories.decay(1);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::Needs;

    #[test]
    fn test_add_memory() {
        let mut memories = Memories::default();
        memories.add(MemoryType::WitnessedDeath, 100);

        assert_eq!(memories.items.len(), 1);
        assert_eq!(memories.items[0].memory_type, MemoryType::WitnessedDeath);
        assert_eq!(memories.items[0].added_at, 100);
        assert!((memories.items[0].intensity - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_memory_decay() {
        let mut memories = Memories::default();
        memories.add(MemoryType::WitnessedDeath, 0);

        // Decay logic: Intensity reduces by decay_rate * ticks
        // WitnessedDeath decay_rate = 0.0005 per tick
        memories.decay(100);

        let expected_intensity = 1.0 - (0.0005 * 100.0);
        assert!((memories.items[0].intensity - expected_intensity).abs() < 0.0001);
        assert!(memories.items[0].intensity < 1.0);
        assert!(memories.items[0].intensity > 0.0);
    }

    #[test]
    fn test_memory_removal_when_faded() {
        let mut memories = Memories::default();
        memories.items.push(ActiveMemory {
            memory_type: MemoryType::WitnessedDeath,
            added_at: 0,
            intensity: 0.0001, // Almost 0
        });

        // Decay enough to kill it
        // 0.0005 per tick. 1 tick -> -0.0005. 0.0001 - 0.0005 < 0.
        memories.decay(1);

        assert!(memories.items.is_empty());
    }

    #[test]
    fn test_calculate_effective_morale() {
        let needs = Needs {
            hunger: 0.5,
            rest: 0.5,
            leisure: 0.5,
        };
        let mut memories = Memories::default();

        // Base morale = (0.5+0.5+0.5)/3 = 0.5
        let _base = needs.morale();

        // WitnessedDeath: -0.2 mood impact at max intensity
        memories.add(MemoryType::WitnessedDeath, 0);

        let effective = calculate_effective_morale(&needs, Some(&memories), None);

        // 0.5 - 0.2 = 0.3
        assert!((effective - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_multiple_memories_stack() {
        let needs = Needs {
            hunger: 0.5,
            rest: 0.5,
            leisure: 0.5,
        };
        let mut memories = Memories::default();

        memories.add(MemoryType::WitnessedDeath, 0); // -0.2
        memories.add(MemoryType::AteFineMeal, 0); // +0.1

        let effective = calculate_effective_morale(&needs, Some(&memories), None);

        // 0.5 - 0.2 + 0.1 = 0.4
        assert!((effective - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_morale_clamping() {
        let needs = Needs {
            hunger: 1.0,
            rest: 1.0,
            leisure: 1.0,
        }; // Base 1.0
        let mut memories = Memories::default();
        memories.add(MemoryType::AteFineMeal, 0); // +0.1

        let effective = calculate_effective_morale(&needs, Some(&memories), None);
        assert!((effective - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_social_buff_stacking() {
        let needs = Needs {
            hunger: 0.5,
            rest: 0.5,
            leisure: 0.5,
        }; // Base 0.5
        let buff = crate::layer1::social::SocialBuff { value: 0.1 };

        let effective = calculate_effective_morale(&needs, None, Some(&buff));
        assert!((effective - 0.6).abs() < f32::EPSILON);
    }
}
