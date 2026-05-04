//! The Epigenetic Crucible (Nova Feature)
//!
//! We have a `StressTracker` system measuring the mental strain of the colony,
//! and a `Traits` system managing entity characteristics.
//! What if intense, chronic stress permanently rewrote a Pop's genetics?
//!
//! The `epigenetic_mutation_system` monitors highly stressed Pops. If their
//! `accumulated_stress` crosses a massive threshold (1000.0), there is a
//! small chance they undergo a traumatic mutation. They gain the `Mutant` trait,
//! their stress is halved (catharsis), and an `AddChronicleEvent` is emitted.

use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::pop::PopName;
use crate::layer1::stress::StressTracker;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;
use rand::Rng;

/// System that allows chronic stress to permanently mutate a Pop.
pub fn epigenetic_mutation_system(
    mut pops: Query<(&mut Traits, &mut StressTracker, Option<&PopName>)>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    let mut rng = rand::thread_rng();

    for (mut traits, mut stress, name) in &mut pops {
        if stress.accumulated_stress > 1000.0 && !traits.has(Trait::Mutant) {
            // 5% chance per tick to mutate if over threshold
            if rng.gen_bool(0.05) {
                traits.add(Trait::Mutant);
                stress.accumulated_stress /= 2.0;

                let pop_name = name.map_or("A pop", |n| &n.0);
                chronicle_events.send(AddChronicleEvent {
                    text: format!(
                        "Pushed beyond their physical limits, {} has fundamentally mutated under the stress.",
                        pop_name
                    ),
                    importance: EventImportance::Major,
            ..Default::default()});
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_epigenetic_mutation_system_triggers() {
        let mut world = World::new();
        world.init_resource::<Events<AddChronicleEvent>>();

        let pop = world
            .spawn((
                Pop,
                Traits::default(),
                StressTracker {
                    accumulated_stress: 2000.0,
                },
                PopName("Nova".to_string()),
            ))
            .id();

        // Run multiple times to ensure the 5% chance triggers reliably (1000 iterations prevents flaky tests)
        for _ in 0..1000 {
            world
                .run_system_once(epigenetic_mutation_system)
                .expect("Failed to run system");
        }

        let traits = world.get::<Traits>(pop).unwrap();
        assert!(traits.has(Trait::Mutant), "Pop should have mutated");

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(
            stress.accumulated_stress <= 1000.0,
            "Stress should have been halved"
        );

        let events = world.resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let chronicle_events: Vec<_> = reader.read(events).collect();
        assert!(!chronicle_events.is_empty(), "Should emit chronicle event");
        assert_eq!(chronicle_events[0].importance, EventImportance::Major);
        assert!(
            chronicle_events[0].text.contains("Nova"),
            "Event should contain pop name"
        );
    }
}
