#![cfg(feature = "nova")]

use bevy_ecs::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::morale::{Morale, MoodModifier};
use crate::layer1::constellations::Sky;
use rand::Rng;

/// Component indicating the constellation a Pop was born under.
///
/// This determines their "Zodiac Resonance" - whether they are aligned or crossed
/// with the current celestial configuration.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZodiacSign {
    /// Index into the `Sky` constellations vector.
    pub index: usize,
}

/// System to assign zodiac signs to new pops.
///
/// Pops are assigned a random sign at birth (spawn).
pub fn assign_zodiac_sign_system(
    mut commands: Commands,
    query: Query<Entity, (With<Pop>, Without<ZodiacSign>)>,
    sky: Option<Res<Sky>>,
) {
    let count = sky.as_ref().map_or(12, |s| {
        if s.constellations.is_empty() {
            12
        } else {
            s.constellations.len()
        }
    });

    let mut rng = rand::thread_rng();

    for entity in query.iter() {
        let index = rng.gen_range(0..count);
        commands.entity(entity).insert(ZodiacSign { index });
    }
}

/// System to apply buffs/debuffs based on current sky alignment.
///
/// * **Stars Aligned:** +15% Morale when your birth sign is ascendant.
/// * **Stars Crossed:** -10% Morale when the opposite sign is ascendant.
pub fn zodiac_resonance_system(
    sky: Res<Sky>,
    mut query: Query<(&ZodiacSign, &mut Morale)>,
) {
    if sky.constellations.is_empty() {
        return;
    }

    let current = sky.current_index;
    let count = sky.constellations.len();
    let opposite = (current + count / 2) % count;

    let aligned_label = "Stars Aligned";
    let crossed_label = "Stars Crossed";

    for (sign, mut morale) in &mut query {
        if sign.index == current {
            // Apply or Refresh Aligned Buff
            if let Some(modifier) = morale.modifiers.iter_mut().find(|m| m.label == aligned_label) {
                modifier.duration = 20; // Refresh
            } else {
                morale.add_modifier(MoodModifier {
                    label: aligned_label.to_string(),
                    value: 0.15,
                    duration: 20,
                });
            }
        } else if sign.index == opposite {
            // Apply or Refresh Crossed Debuff
            if let Some(modifier) = morale.modifiers.iter_mut().find(|m| m.label == crossed_label) {
                modifier.duration = 20; // Refresh
            } else {
                morale.add_modifier(MoodModifier {
                    label: crossed_label.to_string(),
                    value: -0.10,
                    duration: 20,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::constellations::{Constellation, ConstellationEffect};

    #[test]
    fn test_assign_zodiac_sign() {
        let mut world = World::new();
        let entity = world.spawn(Pop).id();

        // Run system without Sky (fallback)
        let mut schedule = Schedule::default();
        schedule.add_systems(assign_zodiac_sign_system);
        schedule.run(&mut world);

        let sign = world.get::<ZodiacSign>(entity).expect("Should have assigned sign");
        assert!(sign.index < 12);
    }

    #[test]
    fn test_assign_zodiac_sign_with_sky() {
        let mut world = World::new();
        let entity = world.spawn(Pop).id();

        let sky = Sky {
            constellations: vec![
                Constellation { name: "A".into(), myth: "".into(), effect: ConstellationEffect::Calm, stars: vec![] },
                Constellation { name: "B".into(), myth: "".into(), effect: ConstellationEffect::Calm, stars: vec![] },
            ],
            current_index: 0,
        };
        world.insert_resource(sky);

        let mut schedule = Schedule::default();
        schedule.add_systems(assign_zodiac_sign_system);
        schedule.run(&mut world);

        let sign = world.get::<ZodiacSign>(entity).unwrap();
        assert!(sign.index < 2);
    }

    #[test]
    fn test_zodiac_resonance_aligned() {
        let mut world = World::new();
        let sky = Sky {
            constellations: vec![
                Constellation { name: "A".into(), myth: "".into(), effect: ConstellationEffect::Calm, stars: vec![] },
                Constellation { name: "B".into(), myth: "".into(), effect: ConstellationEffect::Calm, stars: vec![] },
            ],
            current_index: 0, // 'A' is ascendant
        };
        world.insert_resource(sky);

        let pop = world.spawn((
            Pop,
            ZodiacSign { index: 0 }, // Born under 'A'
            Morale::default(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(zodiac_resonance_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(morale.modifiers.iter().any(|m| m.label == "Stars Aligned"));
        assert!(!morale.modifiers.iter().any(|m| m.label == "Stars Crossed"));
    }

    #[test]
    fn test_zodiac_resonance_crossed() {
        let mut world = World::new();
        let sky = Sky {
            constellations: vec![
                Constellation { name: "A".into(), myth: "".into(), effect: ConstellationEffect::Calm, stars: vec![] },
                Constellation { name: "B".into(), myth: "".into(), effect: ConstellationEffect::Calm, stars: vec![] },
            ],
            current_index: 1, // 'B' is ascendant. 'A' is opposite (in 2-item list, (1+1)%2 = 0)
        };
        world.insert_resource(sky);

        let pop = world.spawn((
            Pop,
            ZodiacSign { index: 0 }, // Born under 'A', which is opposite to 'B'
            Morale::default(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(zodiac_resonance_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(morale.modifiers.iter().any(|m| m.label == "Stars Crossed"));
    }
}
