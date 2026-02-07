use bevy_ecs::prelude::*;
use crate::layer1::needs::Needs;

/// Types of memories a pop can acquire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    /// Witnessed a pop die.
    WitnessedDeath,
    /// Suffered from starvation.
    StarvationTrauma,
    /// Ate a high-quality meal.
    AteFineMeal,
    /// Won a fight.
    WonFight,
}

impl MemoryType {
    /// Returns the base impact on morale (-1.0 to 1.0).
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
            Self::WitnessedDeath => 0.0005,    // Slow fade (2000 ticks)
            Self::StarvationTrauma => 0.001,   // Medium
            Self::AteFineMeal | Self::WonFight => 0.002, // Fast (500 ticks)
        }
    }
}

/// An active memory instance affecting a pop.
#[derive(Debug, Clone)]
pub struct ActiveMemory {
    /// The type of memory.
    pub memory_type: MemoryType,
    /// The tick when this memory was added.
    pub added_at: u64,
    /// Current intensity of the memory (0.0 to 1.0).
    pub intensity: f32,
}

/// Component storing a pop's memories.
#[derive(Component, Default, Debug, Clone)]
pub struct Memories {
    /// List of active memories.
    pub items: Vec<ActiveMemory>,
}

impl Memories {
    /// Adds a new memory to the pop.
    pub fn add(&mut self, memory_type: MemoryType, current_tick: u64) {
        self.items.push(ActiveMemory {
            memory_type,
            added_at: current_tick,
            intensity: 1.0,
        });
    }

    /// Decays all memories and removes faded ones.
    pub fn decay(&mut self, ticks_passed: u64) {
        #[allow(clippy::cast_precision_loss)]
        let ticks_f32 = ticks_passed as f32;
        for memory in &mut self.items {
            memory.intensity -= memory.memory_type.decay_rate() * ticks_f32;
        }
        self.items.retain(|m| m.intensity > 0.0);
    }
}

/// Calculates the effective morale including memory modifiers.
#[must_use]
pub fn calculate_effective_morale(needs: &Needs, memories: &Memories) -> f32 {
    let base = needs.morale();
    let memory_modifier: f32 = memories.items.iter()
        .map(|m| m.memory_type.base_mood_impact() * m.intensity)
        .sum();

    (base + memory_modifier).clamp(0.0, 1.0)
}

/// System that decays memories for all pops each tick.
///
/// This system assumes it runs once per tick.
pub fn memory_decay_system(
    mut query: Query<&mut Memories>,
) {
    // We assume 1 tick per execution for now.
    // If we have variable time steps, we might need Res<Time> or SimulationTime.
    for mut memories in &mut query {
        memories.decay(1);
    }
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

        assert!(memories.items[0].intensity < 1.0, "Intensity should decay");
        assert!(memories.items[0].intensity > 0.0, "Intensity should not be zero yet");
        // 1.0 - (0.0005 * 100) = 0.95
        assert!((memories.items[0].intensity - 0.95).abs() < 0.0001);
    }

    #[test]
    fn test_memory_removal_when_faded() {
        let mut memories = Memories::default();
        memories.add(MemoryType::WitnessedDeath, 0);

        // Force decay to zero
        memories.items[0].intensity = 0.0;
        memories.decay(1);

        assert!(memories.items.is_empty(), "Faded memory should be removed");
    }

    #[test]
    fn test_calculate_effective_morale() {
        let needs = Needs { hunger: 0.5, rest: 0.5, leisure: 0.5 };
        let mut memories = Memories::default();

        // Base morale = (0.5+0.5+0.5)/3 = 0.5

        // WitnessedDeath: -0.2 mood impact at max intensity
        memories.add(MemoryType::WitnessedDeath, 0);

        let effective = calculate_effective_morale(&needs, &memories);

        // 0.5 - 0.2 = 0.3
        assert!((effective - 0.3).abs() < 0.001, "Effective morale should be 0.3, got {}", effective);
    }

    #[test]
    fn test_multiple_memories_stack() {
        let needs = Needs { hunger: 0.5, rest: 0.5, leisure: 0.5 };
        let mut memories = Memories::default();

        memories.add(MemoryType::WitnessedDeath, 0); // -0.2
        memories.add(MemoryType::AteFineMeal, 0);    // +0.1

        let effective = calculate_effective_morale(&needs, &memories);

        // 0.5 - 0.2 + 0.1 = 0.4
        assert!((effective - 0.4).abs() < 0.001, "Effective morale should be 0.4, got {}", effective);
    }

    #[test]
    fn test_morale_clamping() {
        let needs = Needs { hunger: 1.0, rest: 1.0, leisure: 1.0 }; // Base 1.0
        let mut memories = Memories::default();
        memories.add(MemoryType::AteFineMeal, 0); // +0.1

        let effective = calculate_effective_morale(&needs, &memories);
        assert!(effective <= 1.0, "Morale should clamp to 1.0");
    }
}
