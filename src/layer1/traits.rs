//! Trait definitions for Pop personality quirks.

use crate::layer1::day_night::TimeOfDay;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Trait enum defining possible personality quirks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
#[derive(strum_macros::EnumIter)]
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
    /// Logistics expert (+Production on planets when Governor).
    LogisticsExpert,
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
    /// Specialized trait for farming.
    GreenThumb,
    /// Specialized trait for administration and diplomacy.
    SilverTongue,
    /// Specialized trait for mining/underground work.
    MoleEyes,
    /// Specialized trait for hauling.
    Hunchback,
    /// Specialized trait for engineering.
    StaticSkin,
    /// Manufactured in a Clone Vat.
    Clone,
    /// Reduced social needs.
    Soulless,
    /// Resistant to Void Stare effects (Spec 216).
    VoidTouched,
    /// Highly susceptible to Void Stare effects (Spec 216).
    Agoraphobic,
    /// (Spec 250) Obsessed with augmenting their body.
    Transhumanist,
    /// Noble scion, refuses manual labor but pays allowance (Spec 263).
    Noble,
    /// Distrusts colony authorities, ignores placebos (Spec 256).
    Distrustful,
    /// Experiences light colors as emotional sounds.
    Synesthete,
    /// Leaves a spiteful will upon death, giving belongings to rivals or pets.
    Spiteful,
    /// Biosphere empathy link, harmonizes stress and works better near flora.
    EmpathicLink,
    /// Formal administrative capabilities. Understood the bureaucracy (Spec 464).
    Bureaucrat,
    /// (Spec 472) Basic synthetic pop. 100% work efficiency, no morale needs, apathetic to emergencies.
    Synth,
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
            Self::GreenThumb => "GreenThumb",
            Self::SilverTongue => "SilverTongue",
            Self::MoleEyes => "MoleEyes",
            Self::Hunchback => "Hunchback",
            Self::StaticSkin => "StaticSkin",
            Self::Clone => "Clone",
            Self::Soulless => "Soulless",
            Self::VoidTouched => "Void Touched",
            Self::Agoraphobic => "Agoraphobic",
            Self::Transhumanist => "Transhumanist",
            Self::Noble => "Noble",
            Self::Distrustful => "Distrustful",

            Self::Synesthete => "Synesthete",
            Self::Spiteful => "Spiteful",
            Self::EmpathicLink => "Empathic Link",
            Self::Bureaucrat => "Bureaucrat",
            Self::Synth => "Synthetic",
            Self::LogisticsExpert => "Logistics Expert",
        }
    }
}

/// Component storing a set of traits for a pop.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Traits(pub u64);

impl Traits {
    /// Checks if the pop has the given trait.
    #[must_use]
    pub fn has(&self, t: Trait) -> bool {
        (self.0 & (1 << (t as u8))) != 0
    }

    /// Adds a trait to the set.
    pub fn add(&mut self, t: Trait) {
        self.0 |= 1 << (t as u8);
    }

    /// Removes a trait from the set.
    pub fn remove(&mut self, t: Trait) {
        self.0 &= !(1 << (t as u8));
    }

    /// Iterator over the traits.
    pub fn iter(&self) -> impl Iterator<Item = Trait> {
        use strum::IntoEnumIterator;
        let mask = self.0;
        Trait::iter().filter(move |&t| (mask & (1 << (t as u8))) != 0)
    }

    /// Generates a random set of traits.
    pub fn random<R: Rng>(rng: &mut R) -> Self {
        let mut traits = Traits::default();
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
            Trait::Synesthete,
            Trait::Spiteful,
            Trait::EmpathicLink,
            Trait::Bureaucrat,
        ];

        let mut added = 0;
        while added < count {
            let t = pool[rng.gen_range(0..pool.len())];

            // Check conflicts
            if t == Trait::HardWorker && traits.has(Trait::Lazy) {
                continue;
            }
            if t == Trait::Lazy && traits.has(Trait::HardWorker) {
                continue;
            }
            if t == Trait::Glutton && traits.has(Trait::Ascetic) {
                continue;
            }
            if t == Trait::Ascetic && traits.has(Trait::Glutton) {
                continue;
            }
            if t == Trait::NightOwl && traits.has(Trait::EarlyBird) {
                continue;
            }
            if t == Trait::EarlyBird && traits.has(Trait::NightOwl) {
                continue;
            }
            if t == Trait::NativeBorn && traits.has(Trait::WeakImmunity) {
                continue;
            }
            if t == Trait::WeakImmunity && traits.has(Trait::NativeBorn) {
                continue;
            }
            if t == Trait::Optimist && traits.has(Trait::Anxious) {
                continue;
            }
            if t == Trait::Anxious && traits.has(Trait::Optimist) {
                continue;
            }
            if t == Trait::Curious && traits.has(Trait::Traditionalist) {
                continue;
            }
            if t == Trait::Traditionalist && traits.has(Trait::Curious) {
                continue;
            }
            if t == Trait::Spiteful && traits.has(Trait::Compassionate) {
                continue;
            }
            if t == Trait::Compassionate && traits.has(Trait::Spiteful) {
                continue;
            }

            if !traits.has(t) {
                traits.add(t);
                added += 1;
            }
        }

        traits
    }
}

use crate::layer1::utility_types::AssignmentType;

/// Returns the job efficiency modifier based on specialization traits.
#[must_use]
pub fn get_job_efficiency_modifier(traits: &Traits, job: AssignmentType) -> f32 {
    let mut modifier = 1.0;

    if traits.has(Trait::GreenThumb) {
        if job == AssignmentType::FarmWorker {
            modifier += 0.2;
        } else {
            modifier -= 0.2;
        }
    }

    if traits.has(Trait::SilverTongue) {
        if job == AssignmentType::Administrator {
            modifier += 0.2;
        } else {
            modifier -= 0.2;
        }
    }

    if traits.has(Trait::MoleEyes) {
        // No bonus assigned yet, apply penalty to everything
        modifier -= 0.2;
    }

    if traits.has(Trait::Hunchback) {
        // No bonus assigned yet, apply penalty to everything
        modifier -= 0.2;
    }

    if traits.has(Trait::StaticSkin) {
        // No bonus assigned yet, apply penalty to everything
        modifier -= 0.2;
    }

    modifier
}

/// Returns the work speed modifier from traits.
#[must_use]
pub fn get_trait_work_speed_modifier(traits: &Traits) -> f32 {
    let mut modifier = 1.0;
    if traits.has(Trait::HardWorker) {
        modifier += 0.2;
    }
    if traits.has(Trait::Lazy) {
        modifier -= 0.2;
    }
    modifier
}

/// Returns the hunger decay modifier from traits.
#[must_use]
pub fn get_trait_hunger_decay_modifier(traits: &Traits) -> f32 {
    let mut modifier = 1.0;
    if traits.has(Trait::Glutton) {
        modifier += 0.2;
    }
    if traits.has(Trait::Ascetic) {
        modifier -= 0.2;
    }
    modifier
}

/// Returns the leisure decay modifier from traits.
#[must_use]
pub fn get_trait_leisure_decay_modifier(traits: &Traits) -> f32 {
    let mut modifier = 1.0;
    if traits.has(Trait::Synth) {
        return 0.0;
    }
    if traits.has(Trait::Soulless) {
        modifier -= 0.5;
    }
    if traits.has(Trait::Noble) {
        modifier += 0.5;
    }
    modifier
}

/// Returns the movement speed modifier from traits.
#[must_use]
pub fn get_trait_move_speed_modifier(traits: &Traits) -> f32 {
    let mut modifier = 1.0;
    if traits.has(Trait::FastWalker) {
        modifier += 0.1;
    }
    if traits.has(Trait::Feral) {
        modifier += 0.2;
    }
    modifier
}

/// Returns the mood modifier from traits based on time of day.
#[must_use]
pub fn get_trait_mood_modifier(traits: &Traits, time_of_day: TimeOfDay) -> f32 {
    let mut modifier = 0.0;
    if traits.has(Trait::NightOwl) {
        match time_of_day {
            TimeOfDay::Night => modifier += 0.1,
            TimeOfDay::Day => modifier -= 0.05,
            TimeOfDay::Dawn | TimeOfDay::Dusk => {}
        }
    }
    if traits.has(Trait::EarlyBird) {
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

    use crate::layer1::utility_types::AssignmentType;

    #[test]
    fn test_job_efficiency_modifiers() {
        let green_thumb = Traits(1 << (Trait::GreenThumb as u8));
        let silver_tongue = Traits(1 << (Trait::SilverTongue as u8));
        let mole_eyes = Traits(1 << (Trait::MoleEyes as u8));
        let hunchback = Traits(1 << (Trait::Hunchback as u8));
        let static_skin = Traits(1 << (Trait::StaticSkin as u8));
        let normal = Traits::default();

        // Normal has no modifiers
        assert!(
            (get_job_efficiency_modifier(&normal, AssignmentType::FarmWorker) - 1.0).abs()
                < f32::EPSILON
        );

        // GreenThumb bonuses and penalties
        assert!(get_job_efficiency_modifier(&green_thumb, AssignmentType::FarmWorker) > 1.0);
        assert!(get_job_efficiency_modifier(&green_thumb, AssignmentType::Administrator) < 1.0);

        // SilverTongue bonuses and penalties
        assert!(get_job_efficiency_modifier(&silver_tongue, AssignmentType::Administrator) > 1.0);
        assert!(get_job_efficiency_modifier(&silver_tongue, AssignmentType::FarmWorker) < 1.0);

        // MoleEyes penalties (no bonus assigned to AssignmentType yet)
        assert!(get_job_efficiency_modifier(&mole_eyes, AssignmentType::FarmWorker) < 1.0);

        // Hunchback penalties (no bonus assigned to AssignmentType yet)
        assert!(get_job_efficiency_modifier(&hunchback, AssignmentType::Administrator) < 1.0);

        // StaticSkin penalties (no bonus assigned to AssignmentType yet)
        assert!(get_job_efficiency_modifier(&static_skin, AssignmentType::FarmWorker) < 1.0);
    }

    #[test]
    fn test_traits_random_generation() {
        let mut rng = rand::thread_rng();
        // Just verify it doesn't panic and produces valid Traits
        let _ = Traits::random(&mut rng);
    }

    #[test]
    fn test_work_speed_modifiers() {
        let hard_worker = Traits(1 << (Trait::HardWorker as u8));
        let lazy = Traits(1 << (Trait::Lazy as u8));
        let normal = Traits::default();

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
        let glutton = Traits(1 << (Trait::Glutton as u8));
        let ascetic = Traits(1 << (Trait::Ascetic as u8));
        let normal = Traits::default();

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
    fn test_leisure_decay_modifiers() {
        let soulless = Traits(1 << (Trait::Soulless as u8));
        let noble = Traits(1 << (Trait::Noble as u8));
        let synth = Traits(1 << (Trait::Synth as u8));
        let normal = Traits::default();

        assert!(
            get_trait_leisure_decay_modifier(&soulless) < 1.0,
            "Soulless should have reduced leisure decay"
        );
        assert!(
            get_trait_leisure_decay_modifier(&noble) > 1.0,
            "Noble should have increased leisure decay"
        );
        assert!(
            (get_trait_leisure_decay_modifier(&synth) - 0.0).abs() < f32::EPSILON,
            "Synth should have zero leisure decay"
        );
        assert!(
            (get_trait_leisure_decay_modifier(&normal) - 1.0).abs() < f32::EPSILON,
            "Normal should decay normally"
        );
    }

    #[test]
    fn test_feral_speed_modifier() {
        let feral = Traits(1 << (Trait::Feral as u8));
        let fast = Traits(1 << (Trait::FastWalker as u8));
        let both = Traits((1 << (Trait::Feral as u8)) | (1 << (Trait::FastWalker as u8)));

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
        let night_owl = Traits(1 << (Trait::NightOwl as u8));

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
            let has_lazy = traits.has(Trait::Lazy);
            let has_hard_worker = traits.has(Trait::HardWorker);
            assert!(
                !(has_lazy && has_hard_worker),
                "Should not be both Lazy and HardWorker"
            );

            let has_glutton = traits.has(Trait::Glutton);
            let has_ascetic = traits.has(Trait::Ascetic);
            assert!(
                !(has_glutton && has_ascetic),
                "Should not be both Glutton and Ascetic"
            );

            let has_night_owl = traits.has(Trait::NightOwl);
            let has_early_bird = traits.has(Trait::EarlyBird);
            assert!(
                !(has_night_owl && has_early_bird),
                "Should not be both NightOwl and EarlyBird"
            );
        }
    }
}
