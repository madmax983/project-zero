//! Tectonic Prophets (Nova Feature)
//!
//! # The Spark
//! We have `TectonicStress` in the geology layer which builds towards a `MegaQuake`.
//! We also have `Trait::Prophet` and `Trait::Anxious` in the psychology system.
//!
//! # The Feature
//! When `TectonicStress` exceeds 80% of its threshold, Pops with `Trait::Prophet`
//! feel the rumbling and gain a massive morale boost ("Vibrations of the Deep"),
//! while Pops with `Trait::Anxious` suffer a severe morale penalty ("Impending Doom").
//!
//! # The Potential
//! Connects the deep crust physical simulation directly to the socio-psychological
//! state of the colonists, providing dynamic narrative tension before disaster strikes.

use bevy_ecs::prelude::*;
use crate::layer1::geology::tectonic::TectonicStress;
use crate::layer1::psychology::traits::{Trait, Traits};
use crate::layer1::social::morale::{MoodModifier, Morale};
use crate::layer1::entities::pop::Pop;

pub fn tectonic_prophets_system(
    stress: Option<Res<TectonicStress>>,
    mut pops: Query<(&Traits, &mut Morale), With<Pop>>,
) {
    let Some(stress) = stress else { return; };

    // Trigger when stress is >= 80% of threshold
    if stress.current >= stress.threshold * 0.8 {
        for (traits, mut morale) in pops.iter_mut() {
            if traits.has(Trait::Prophet) {
                morale.add_modifier(MoodModifier {
                    label: "Vibrations of the Deep".to_string(),
                    value: 0.2,
                    duration: 10,
                });
            }
            if traits.has(Trait::Anxious) {
                morale.add_modifier(MoodModifier {
                    label: "Impending Doom".to_string(),
                    value: -0.2,
                    duration: 10,
                });
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(tectonic_prophets_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::utils::HashSet;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_tectonic_prophets_apply_morale_effects() {
        let mut world = World::new();

        world.insert_resource(TectonicStress {
            current: 85.0,
            threshold: 100.0,
            dissipation_rate: 0.1,
        });

        let mut prophet_traits = Traits(HashSet::default());
        prophet_traits.add(Trait::Prophet);

        let mut anxious_traits = Traits(HashSet::default());
        anxious_traits.add(Trait::Anxious);

        let prophet_pop = world.spawn((Pop, prophet_traits, Morale::default())).id();
        let anxious_pop = world.spawn((Pop, anxious_traits, Morale::default())).id();

        world.run_system_once(tectonic_prophets_system).unwrap();

        let prophet_morale = world.get::<Morale>(prophet_pop).unwrap();
        assert!(prophet_morale.modifiers.iter().any(|m| m.label == "Vibrations of the Deep"));

        let anxious_morale = world.get::<Morale>(anxious_pop).unwrap();
        assert!(anxious_morale.modifiers.iter().any(|m| m.label == "Impending Doom"));
    }

    #[test]
    fn test_tectonic_prophets_no_effect_below_threshold() {
        let mut world = World::new();

        world.insert_resource(TectonicStress {
            current: 50.0, // Below 80%
            threshold: 100.0,
            dissipation_rate: 0.1,
        });

        let mut prophet_traits = Traits(HashSet::default());
        prophet_traits.add(Trait::Prophet);

        let prophet_pop = world.spawn((Pop, prophet_traits, Morale::default())).id();

        world.run_system_once(tectonic_prophets_system).unwrap();

        let prophet_morale = world.get::<Morale>(prophet_pop).unwrap();
        assert!(prophet_morale.modifiers.is_empty());
    }
}
