//! Trait definitions for Pop personality quirks.

use crate::layer1::day_night::TimeOfDay;
use bevy_ecs::prelude::*;
use rand::Rng;
use std::collections::HashSet;

/// Trait enum defining possible personality quirks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trait {
    /// +20% Work Speed.
    HardWorker,
    /// -20% Work Speed.
    Lazy,
    /// +20% Hunger Decay.
    Glutton,
    /// -20% Hunger Decay.
    Ascetic,
    /// +Mood at Night, -Mood at Day.
    NightOwl,
    /// +Mood at Morning, -Mood at Night.
    EarlyBird,
    /// +10% Move Speed.
    FastWalker,
    /// Hoards Valuables (Metal, Luxuries).
    Greedy,
    /// Hoards Survival Goods (Food, Meds).
    Anxious,
    /// Resists atmospheric hazards (+30% Biocompatibility).
    NativeBorn,
    /// Vulnerable to atmospheric hazards (-20% Biocompatibility).
    WeakImmunity,
    /// Loves fire (starts fires during breakdowns).
    Pyromaniac,
    /// Raised in the wild (+20% Move Speed, -Intellectual).
    Feral,
    /// Optimistic outlook (+Mood from Observatory).
    Optimist,
    /// Curious nature (+Knowledge/Mood from Observatory).
    Curious,
    /// Traditional values (-Mood from Observatory).
    Traditionalist,
    /// Prone to violent outbursts (+Risk of breakdown).
    Volatile,
    /// Creative mindset (+Cryo Dream rate, +Art quality).
    Creative,
    /// Intellectual mindset (+Cryo Dream rate, +Research speed).
    Intellectual,
    /// Unaffected by eating rations or corpses.
    Cannibal,
    /// Accepts survival necessities without complaint (ignore Ration mood penalty).
    Pragmatist,
    /// Socially isolated and often blamed.
    Outsider,
    /// Genetically deviant and mistrusted.
    Mutant,
    /// Highly empathetic and prone to guilt.
    Compassionate,
    /// Sensitive to The Hum (Spec 238).
    Sensitive,
}

impl Trait {
    /// Returns a human-readable label for the trait.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::HardWorker => "Hard Worker",
            Self::Lazy => "Lazy",
            Self::Glutton => "Glutton",
            Self::Ascetic => "Ascetic",
            Self::NightOwl => "Night Owl",
            Self::EarlyBird => "Early Bird",
            Self::FastWalker => "Fast Walker",
            Self::Greedy => "Greedy",
            Self::Anxious => "Anxious",
            Self::NativeBorn => "Native Born",
            Self::WeakImmunity => "Weak Immunity",
            Self::Pyromaniac => "Pyromaniac",
            Self::Feral => "Feral",
            Self::Optimist => "Optimist",
            Self::Curious => "Curious",
            Self::Traditionalist => "Traditionalist",
            Self::Volatile => "Volatile",
            Self::Creative => "Creative",
            Self::Intellectual => "Intellectual",
            Self::Cannibal => "Cannibal",
            Self::Pragmatist => "Pragmatist",
            Self::Outsider => "Outsider",
            Self::Mutant => "Mutant",
            Self::Compassionate => "Compassionate",
            Self::Sensitive => "Sensitive",
        }
    }
}

/// Component storing a set of traits for a pop.
#[derive(Component, Debug, Clone, Default)]
pub struct Traits(pub HashSet<Trait>);

impl Traits {
    /// Checks if the pop has the given trait.
    #[must_use]
    pub fn has(&self, t: Trait) -> bool {
        self.0.contains(&t)
    }

    /// Adds a trait to the set.
    pub fn add(&mut self, t: Trait) {
        self.0.insert(t);
    }

    /// Generates a random set of traits.
    pub fn random<R: Rng>(rng: &mut R) -> Self {
        let mut set = HashSet::new();
        // Simple logic: 50% chance to get 1 trait, 20% for 2.
        let count = if rng.gen_bool(0.2) {
            2
        } else {
            usize::from(rng.gen_bool(0.5))
        };

        // Pool of all traits
        let pool = [
            Trait::HardWorker,
            Trait::Lazy,
            Trait::Glutton,
            Trait::Ascetic,
            Trait::NightOwl,
            Trait::EarlyBird,
            Trait::FastWalker,
            Trait::Greedy,
            Trait::Anxious,
            Trait::NativeBorn,
            Trait::WeakImmunity,
            Trait::Pyromaniac,
            Trait::Optimist,
            Trait::Curious,
            Trait::Traditionalist,
            Trait::Volatile,
            Trait::Creative,
            Trait::Intellectual,
            Trait::Cannibal,
            Trait::Pragmatist,
            Trait::Outsider,
            Trait::Mutant,
            Trait::Compassionate,
        ];

        while set.len() < count {
            let t = pool[rng.gen_range(0..pool.len())];

            // Check conflicts
            if t == Trait::HardWorker && set.contains(&Trait::Lazy) {
                continue;
            }
            if t == Trait::Lazy && set.contains(&Trait::HardWorker) {
                continue;
            }
            if t == Trait::Glutton && set.contains(&Trait::Ascetic) {
                continue;
            }
            if t == Trait::Ascetic && set.contains(&Trait::Glutton) {
                continue;
            }
            if t == Trait::NightOwl && set.contains(&Trait::EarlyBird) {
                continue;
            }
            if t == Trait::EarlyBird && set.contains(&Trait::NightOwl) {
                continue;
            }
            if t == Trait::NativeBorn && set.contains(&Trait::WeakImmunity) {
                continue;
            }
            if t == Trait::WeakImmunity && set.contains(&Trait::NativeBorn) {
                continue;
            }
            if t == Trait::Optimist && set.contains(&Trait::Anxious) {
                continue;
            }
            if t == Trait::Anxious && set.contains(&Trait::Optimist) {
                continue;
            }
            if t == Trait::Curious && set.contains(&Trait::Traditionalist) {
                continue;
            }
            if t == Trait::Traditionalist && set.contains(&Trait::Curious) {
                continue;
            }

            set.insert(t);
        }

        Self(set)
    }
}

/// Returns the work speed modifier from traits.
#[must_use]
pub fn get_trait_work_speed_modifier(traits: &Traits) -> f32 {
    let mut modifier = 1.0;
    if traits.0.contains(&Trait::HardWorker) {
        modifier += 0.2;
    }
    if traits.0.contains(&Trait::Lazy) {
        modifier -= 0.2;
    }
    modifier
}

/// Returns the hunger decay modifier from traits.
#[must_use]
pub fn get_trait_hunger_decay_modifier(traits: &Traits) -> f32 {
    let mut modifier = 1.0;
    if traits.0.contains(&Trait::Glutton) {
        modifier += 0.2;
    }
    if traits.0.contains(&Trait::Ascetic) {
        modifier -= 0.2;
    }
    modifier
}

/// Returns the movement speed modifier from traits.
#[must_use]
pub fn get_trait_move_speed_modifier(traits: &Traits) -> f32 {
    let mut modifier = 1.0;
    if traits.0.contains(&Trait::FastWalker) {
        modifier += 0.1;
    }
    if traits.0.contains(&Trait::Feral) {
        modifier += 0.2;
    }
    modifier
}

/// Returns the mood modifier from traits based on time of day.
#[must_use]
pub fn get_trait_mood_modifier(traits: &Traits, time_of_day: TimeOfDay) -> f32 {
    let mut modifier = 0.0;
    if traits.0.contains(&Trait::NightOwl) {
        match time_of_day {
            TimeOfDay::Night => modifier += 0.1,
            TimeOfDay::Day => modifier -= 0.05,
            TimeOfDay::Dawn | TimeOfDay::Dusk => {}
        }
    }
    if traits.0.contains(&Trait::EarlyBird) {
        match time_of_day {
            TimeOfDay::Dawn | TimeOfDay::Day => modifier += 0.05,
            TimeOfDay::Night => modifier -= 0.1,
            TimeOfDay::Dusk => {}
        }
    }
    modifier
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::day_night::TimeOfDay;
    use std::collections::HashSet;

    #[test]
    fn test_traits_random_generation() {
        let mut rng = rand::thread_rng();
        // Just verify it doesn't panic and produces valid Traits
        let _ = Traits::random(&mut rng);
    }

    #[test]
    fn test_work_speed_modifiers() {
        let hard_worker = Traits(HashSet::from([Trait::HardWorker]));
        let lazy = Traits(HashSet::from([Trait::Lazy]));
        let normal = Traits(HashSet::new());

        assert!(
            get_trait_work_speed_modifier(&hard_worker) > 1.0,
            "HardWorker should work faster"
        );
        assert!(
            get_trait_work_speed_modifier(&lazy) < 1.0,
            "Lazy should work slower"
        );
        assert!(
            (get_trait_work_speed_modifier(&normal) - 1.0).abs() < f32::EPSILON,
            "Normal should work at normal speed"
        );
    }

    #[test]
    fn test_hunger_decay_modifiers() {
        let glutton = Traits(HashSet::from([Trait::Glutton]));
        let ascetic = Traits(HashSet::from([Trait::Ascetic]));
        let normal = Traits(HashSet::new());

        assert!(
            get_trait_hunger_decay_modifier(&glutton) > 1.0,
            "Glutton should eat more"
        );
        assert!(
            get_trait_hunger_decay_modifier(&ascetic) < 1.0,
            "Ascetic should eat less"
        );
        assert!(
            (get_trait_hunger_decay_modifier(&normal) - 1.0).abs() < f32::EPSILON,
            "Normal should eat normally"
        );
    }

    #[test]
    fn test_feral_speed_modifier() {
        let feral = Traits(HashSet::from([Trait::Feral]));
        let fast = Traits(HashSet::from([Trait::FastWalker]));
        let both = Traits(HashSet::from([Trait::Feral, Trait::FastWalker]));

        assert!(
            (get_trait_move_speed_modifier(&feral) - 1.2).abs() < 0.0001,
            "Feral should be 20% faster"
        );
        assert!(
            (get_trait_move_speed_modifier(&fast) - 1.1).abs() < 0.0001,
            "FastWalker should be 10% faster"
        );
        assert!(
            (get_trait_move_speed_modifier(&both) - 1.3).abs() < 0.0001,
            "Both should be 30% faster"
        );
    }

    #[test]
    fn test_night_owl_mood_modifier() {
        let night_owl = Traits(HashSet::from([Trait::NightOwl]));

        let mood_night = get_trait_mood_modifier(&night_owl, TimeOfDay::Night);
        assert!(mood_night > 0.0, "NightOwl should be happier at night");

        let mood_day = get_trait_mood_modifier(&night_owl, TimeOfDay::Day);
        assert!(mood_day < 0.0, "NightOwl should be sadder during day");
    }

    #[test]
    fn test_conflicting_traits() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let traits = Traits::random(&mut rng);
            let has_lazy = traits.0.contains(&Trait::Lazy);
            let has_hard_worker = traits.0.contains(&Trait::HardWorker);
            assert!(
                !(has_lazy && has_hard_worker),
                "Should not be both Lazy and HardWorker"
            );

            let has_glutton = traits.0.contains(&Trait::Glutton);
            let has_ascetic = traits.0.contains(&Trait::Ascetic);
            assert!(
                !(has_glutton && has_ascetic),
                "Should not be both Glutton and Ascetic"
            );

            let has_night_owl = traits.0.contains(&Trait::NightOwl);
            let has_early_bird = traits.0.contains(&Trait::EarlyBird);
            assert!(
                !(has_night_owl && has_early_bird),
                "Should not be both NightOwl and EarlyBird"
            );
        }
    }
}
