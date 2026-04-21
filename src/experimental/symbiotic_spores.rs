//! The Symbiotic Spores (Nova Feature).
//!
//! # The Spark
//! We have `Mutated` pops from the Mutagenic Rain and `Fauna` roaming the map.
//! What if these mutants develop a biological connection to the alien life?
//!
//! # The Feature
//! Pops that carry mutant traits (e.g., `Photosynthesis`, `ThickSkin`) release
//! "Symbiotic Spores" when they are physically close to `Fauna`. This slow-acting
//! aura gradually heals both the Pop and the Fauna, and slightly restores the Pop's
//! rest need, turning outcasts into vital, nature-bound healers.

use crate::layer1::fauna::Fauna;
use crate::layer1::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

const SPORE_RADIUS: u32 = 3;
const HEAL_AMOUNT: f32 = 0.5;
const REST_RESTORE: f32 = 0.01;

type SymbioticFaunaQuery<'w, 's> =
    Query<'w, 's, (&'static GridPosition, &'static mut Health), (With<Fauna>, Without<Pop>)>;

/// System that applies symbiotic healing between mutant Pops and nearby Fauna.
pub fn symbiotic_spore_system(
    mut pops: Query<(&GridPosition, &Traits, &mut Health, &mut Needs), With<Pop>>,
    mut faunas: SymbioticFaunaQuery<'_, '_>,
) {
    for (pop_pos, traits, mut pop_health, mut needs) in pops.iter_mut() {
        // Check if the pop is a mutant (has a mutant trait)
        let is_mutant = traits.has(Trait::Photosynthesis)
            || traits.has(Trait::ThickSkin)
            || traits.has(Trait::BrittleBones)
            || traits.has(Trait::ExtremeHunger)
            || traits.has(Trait::Mutant);

        if !is_mutant {
            continue;
        }

        // Check for nearby fauna
        for (fauna_pos, mut fauna_health) in faunas.iter_mut() {
            if pop_pos.distance_chebyshev(*fauna_pos) <= SPORE_RADIUS {
                // Symbiosis active!

                // Heal Pop
                pop_health.current = (pop_health.current + HEAL_AMOUNT).min(pop_health.max);
                // Restore Rest
                needs.rest = (needs.rest + REST_RESTORE).min(1.0);

                // Heal Fauna
                fauna_health.current = (fauna_health.current + HEAL_AMOUNT).min(fauna_health.max);

                // One fauna is enough to trigger the pop's benefits per tick
                break;
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(symbiotic_spore_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_symbiosis_heals_mutant_and_fauna() {
        let mut world = setup_world();

        let mutant_traits = {
            let mut t = Traits::default();
            t.add(Trait::Mutant);
            t
        };

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                mutant_traits,
                Health {
                    current: 50.0,
                    max: 100.0,
                    conditions: Vec::new(),
                },
                Needs {
                    rest: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        let fauna = world
            .spawn((
                Fauna::default(),
                GridPosition { x: 6, y: 5 }, // Within radius 3
                Health {
                    current: 20.0,
                    max: 50.0,
                    conditions: Vec::new(),
                },
            ))
            .id();

        world.run_system_once(symbiotic_spore_system).unwrap();

        let pop_health = world.get::<Health>(pop).unwrap();
        let pop_needs = world.get::<Needs>(pop).unwrap();
        let fauna_health = world.get::<Health>(fauna).unwrap();

        assert_eq!(pop_health.current, 50.5);
        assert_eq!(pop_needs.rest, 0.51);
        assert_eq!(fauna_health.current, 20.5);
    }

    #[test]
    fn test_no_symbiosis_for_normal_pop() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Traits::default(), // Normal pop
                Health {
                    current: 50.0,
                    max: 100.0,
                    conditions: Vec::new(),
                },
                Needs {
                    rest: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        let fauna = world
            .spawn((
                Fauna::default(),
                GridPosition { x: 6, y: 5 },
                Health {
                    current: 20.0,
                    max: 50.0,
                    conditions: Vec::new(),
                },
            ))
            .id();

        world.run_system_once(symbiotic_spore_system).unwrap();

        let pop_health = world.get::<Health>(pop).unwrap();
        let fauna_health = world.get::<Health>(fauna).unwrap();

        assert_eq!(pop_health.current, 50.0);
        assert_eq!(fauna_health.current, 20.0);
    }

    #[test]
    fn test_no_symbiosis_out_of_range() {
        let mut world = setup_world();

        let mutant_traits = {
            let mut t = Traits::default();
            t.add(Trait::Mutant);
            t
        };

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                mutant_traits,
                Health {
                    current: 50.0,
                    max: 100.0,
                    conditions: Vec::new(),
                },
                Needs {
                    rest: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        let fauna = world
            .spawn((
                Fauna::default(),
                GridPosition { x: 20, y: 20 }, // Far away
                Health {
                    current: 20.0,
                    max: 50.0,
                    conditions: Vec::new(),
                },
            ))
            .id();

        world.run_system_once(symbiotic_spore_system).unwrap();

        let pop_health = world.get::<Health>(pop).unwrap();
        let fauna_health = world.get::<Health>(fauna).unwrap();

        assert_eq!(pop_health.current, 50.0);
        assert_eq!(fauna_health.current, 20.0);
    }
}
