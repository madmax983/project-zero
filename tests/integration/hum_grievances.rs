use bevy_ecs::prelude::*;
use scale::layer1::map::GridPosition;
use scale::layer1::needs::Needs;
use scale::layer1::pop::Pop;
use scale::layer1::social::grievances::{post_grievance_system, BulletinBoard};
use scale::layer1::stress::StressTracker;
use scale::layer1::traits::{Trait, Traits};
use std::collections::HashSet;

#[test]
fn test_sensitive_pop_posts_hum_grievance() {
    let mut world = World::new();

    // Spawn Board
    let board_ent = world
        .spawn((BulletinBoard::default(), GridPosition { x: 0, y: 0 }))
        .id();

    // Spawn Sensitive Pop with High Stress
    let mut traits = HashSet::new();
    traits.insert(Trait::Sensitive);

    let _pop = world
        .spawn((
            Pop,
            Needs {
                hunger: 0.1,
                rest: 0.1,
                leisure: 0.1,
                hygiene: 0.1,
            },
            GridPosition { x: 0, y: 0 },
            Traits(traits),
            StressTracker {
                accumulated_stress: 80.0, // High stress (> 50.0)
            },
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(post_grievance_system);

    // Run until a note is posted (dealing with 1% chance)
    let mut note_posted = false;
    for _ in 0..500 {
        schedule.run(&mut world);
        let board = world.get::<BulletinBoard>(board_ent).unwrap();
        if !board.notes.is_empty() {
            note_posted = true;
            break;
        }
    }

    assert!(note_posted, "Pop should have posted a note");

    let board = world.get::<BulletinBoard>(board_ent).unwrap();
    let note = &board.notes[0];

    // Verify content is Hum-related
    // Currently (RED Phase), it will be "I blame X" or "I have strong feelings!"
    // We want it to be about the Hum.
    let content = &note.content;
    let hum_phrases = ["The Hum", "singing", "vibration", "loud today"];

    let found_hum_reference = hum_phrases.iter().any(|phrase| content.contains(phrase));
    assert!(
        found_hum_reference,
        "Note content '{}' should reference The Hum",
        content
    );
}
