use crate::layer1::day_night::DayNightCycle;
use crate::layer1::edicts::ColonyPolicies;
use crate::layer1::mascot::MascotBuff;
use crate::layer1::memory::{Memories, calculate_raw_morale};
use crate::layer1::needs::Needs;
use crate::layer1::social::SocialBuff;
use crate::layer1::traits::Traits;
use bevy_ecs::prelude::*;

/// A temporary modifier affecting a Pop's morale.
#[derive(Clone, Debug)]
pub struct MoodModifier {
    /// The name of the modifier (e.g. "Witnessed Joy").
    pub label: String,
    /// The value of the modifier (-1.0 to 1.0).
    pub value: f32,
    /// Remaining duration in ticks.
    pub duration: u32,
}

/// Component storing morale state and modifiers.
#[derive(Component)]
pub struct Morale {
    /// Cached effective morale (updated every tick).
    pub value: f32,
    /// List of active mood modifiers.
    pub modifiers: Vec<MoodModifier>,
}

impl Default for Morale {
    fn default() -> Self {
        Self {
            value: 0.8, // Matches Needs default
            modifiers: Vec::new(),
        }
    }
}

impl Morale {
    /// Adds a mood modifier to the pop.
    pub fn add_modifier(&mut self, modifier: MoodModifier) {
        self.modifiers.push(modifier);
    }
}

/// System to decay morale modifiers.
pub fn morale_decay_system(mut query: Query<&mut Morale>) {
    query.par_iter_mut().for_each(|mut morale| {
        for modifier in &mut morale.modifiers {
            if modifier.duration > 0 {
                modifier.duration -= 1;
            }
        }
        morale.modifiers.retain(|m| m.duration > 0);
    });
}

/// System to update the cached morale value.
/// This runs after needs decay and before AI/Contagion.
#[allow(clippy::type_complexity)]
pub fn update_morale_cache_system(
    mut query: Query<(
        &mut Morale,
        &Needs,
        Option<&Memories>,
        Option<&SocialBuff>,
        Option<&Traits>,
        Option<&MascotBuff>,
    )>,
    policies: Option<Res<ColonyPolicies>>,
    day_night: Option<Res<DayNightCycle>>,
) {
    let cycle = day_night.map(|d| d.time_of_day);

    query.par_iter_mut().for_each(
        |(mut morale, needs, memories, social, traits, mascot_buff)| {
            let raw = calculate_raw_morale(
                needs,
                memories,
                social,
                policies.as_deref(),
                traits,
                cycle,
                None,
            );

            let mascot_bonus = mascot_buff.map_or(0.0, |b| b.amount);
            let modifier_sum: f32 = morale.modifiers.iter().map(|m| m.value).sum();

            morale.value = (raw + modifier_sum + mascot_bonus).clamp(0.0, 1.0);
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morale_modifiers_apply() {
        let mut morale = Morale::default();
        morale.modifiers.push(MoodModifier {
            label: "Happy".to_string(),
            value: 0.1,
            duration: 10,
        });

        assert_eq!(morale.modifiers.len(), 1);
        assert!((morale.modifiers[0].value - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_decay_system() {
        let mut world = World::new();
        let entity = world
            .spawn(Morale {
                value: 0.5,
                modifiers: vec![MoodModifier {
                    label: "Temp".to_string(),
                    value: 0.1,
                    duration: 1,
                }],
            })
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(morale_decay_system);

        // Tick 1: Duration becomes 0
        schedule.run(&mut world);

        let morale = world.get::<Morale>(entity).unwrap();
        assert!(morale.modifiers.is_empty());
    }
}
