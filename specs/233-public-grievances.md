# 233: Public Grievances

## Overview

The "Public Grievances" system introduces a **Bulletin Board** building. This structure serves as a physical location where Pops can express their feelings about the colony, specific individuals, or the leadership.

- **Posting**: Pops with extreme Morale (High or Low) or specific traits will seek out a Bulletin Board to post a "Note".
- **Reading**: Idle Pops or those taking a Leisure break may read the board.
- **Impact**: Reading a note affects the reader's opinion (Affinity) towards the subject of the note.

This adds depth to the social simulation ("Every Problem Has a Face") and gives players a way to "read the room" by inspecting the board.

## Dependencies

- `004` — Basic Building (New `BulletinBoard` type)
- `031` — Pop Morale (Triggers posting)
- `047` — Pop Relationships (Affinity changes)
- `016` — Utility AI (New actions: `PostGrievance`, `ReadBoard`)

## RED Phase: Tests First

Write these tests in `src/layer1/social/grievances_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::social::grievances::{BulletinBoard, BulletinNote, Sentiment, post_grievance_system, read_board_system};
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;
    use crate::layer1::social::{Relationships, AffinityChange};
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_bulletin_board_component() {
        let board = BulletinBoard::default();
        assert!(board.notes.is_empty());
    }

    #[test]
    fn test_post_grievance_low_morale() {
        let mut world = World::new();

        // Spawn Board
        let board_ent = world.spawn((
            BulletinBoard::default(),
            GridPosition { x: 0, y: 0 },
        )).id();

        // Spawn Unhappy Pop
        let pop = world.spawn((
            Pop,
            Needs { morale: 0.1, ..Default::default() }, // Very low morale
            GridPosition { x: 0, y: 0 },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(post_grievance_system);
        schedule.run(&mut world);

        // Check if note posted
        let board = world.get::<BulletinBoard>(board_ent).unwrap();
        assert_eq!(board.notes.len(), 1);
        assert_eq!(board.notes[0].author, pop);
        assert!(matches!(board.notes[0].sentiment, Sentiment::Negative));
    }

    #[test]
    fn test_post_commendation_high_morale() {
        let mut world = World::new();

        let board_ent = world.spawn((
            BulletinBoard::default(),
            GridPosition { x: 0, y: 0 },
        )).id();

        let pop = world.spawn((
            Pop,
            Needs { morale: 0.9, ..Default::default() }, // Very high morale
            GridPosition { x: 0, y: 0 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(post_grievance_system);
        schedule.run(&mut world);

        let board = world.get::<BulletinBoard>(board_ent).unwrap();
        assert_eq!(board.notes.len(), 1);
        assert!(matches!(board.notes[0].sentiment, Sentiment::Positive));
    }

    #[test]
    fn test_read_board_affects_affinity() {
        let mut world = World::new();
        world.init_resource::<Events<AffinityChange>>();

        let target_pop = world.spawn(Pop).id();
        let author_pop = world.spawn(Pop).id();

        // Board with a negative note about target_pop
        let note = BulletinNote {
            author: author_pop,
            target: Some(target_pop),
            sentiment: Sentiment::Negative,
            content: "They stole my lunch!".to_string(),
            timestamp: 0,
        };

        let board_ent = world.spawn((
            BulletinBoard { notes: vec![note] },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Reader Pop
        let reader_pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            // Tag component to indicate they are reading (simulating action state)
            crate::layer1::social::grievances::ReadingBoard { board_entity: board_ent },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(read_board_system);
        schedule.run(&mut world);

        // Check for AffinityChange event
        let events = world.resource::<Events<AffinityChange>>();
        let mut reader = events.get_reader();
        let emitted: Vec<_> = reader.read(events).collect();

        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].source, reader_pop);
        assert_eq!(emitted[0].target, target_pop);
        assert!(emitted[0].amount < 0.0); // Should decrease affinity
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `src/layer1/building.rs`

Add `BulletinBoard` to `BuildingType` enum.
- Cost: 20 Wood.
- Label: "Bulletin Board".
- Char: 'B'.
- Tech: Social Structures (or none/basic).

### 2. Create `src/layer1/social/grievances.rs`

```rust
use bevy_ecs::prelude::*;
use crate::layer1::needs::Needs;
use crate::layer1::social::AffinityChange;

#[derive(Debug, Clone, PartialEq)]
pub enum Sentiment {
    Positive,
    Negative,
    Neutral,
}

#[derive(Debug, Clone)]
pub struct BulletinNote {
    pub author: Entity,
    pub target: Option<Entity>, // Subject of the note
    pub sentiment: Sentiment,
    pub content: String,
    pub timestamp: u32,
}

#[derive(Component, Default)]
pub struct BulletinBoard {
    pub notes: Vec<BulletinNote>,
}

// Marker for a pop currently reading a board (assigned by Action system)
#[derive(Component)]
pub struct ReadingBoard {
    pub board_entity: Entity,
}

pub fn post_grievance_system(
    mut commands: Commands,
    mut boards: Query<&mut BulletinBoard>,
    pops: Query<(Entity, &Needs)>,
    // In real imp, would need proximity check. For MVP, find ANY board.
) {
    // Limit posting frequency in real imp.

    if let Ok(mut board) = boards.get_single_mut() {
        for (entity, needs) in pops.iter() {
            // Simplified logic: High/Low morale triggers post
            // In reality, this should be an Action via Utility AI, not a direct system effect.
            // But for this spec's scope, we simulate the "decision" here or assume the Action sets a component.

            // Let's assume this system runs when a pop *completes* a PostGrievance action.
            // But for the RED test above, we just check morale directly.

            let sentiment = if needs.morale < 0.2 {
                Some(Sentiment::Negative)
            } else if needs.morale > 0.8 {
                Some(Sentiment::Positive)
            } else {
                None
            };

            if let Some(s) = sentiment {
                let note = BulletinNote {
                    author: entity,
                    target: None, // Random target in full imp
                    sentiment: s,
                    content: "I have strong feelings!".to_string(),
                    timestamp: 0,
                };
                board.notes.push(note);
            }
        }
    }
}

pub fn read_board_system(
    mut commands: Commands,
    mut events: EventWriter<AffinityChange>,
    boards: Query<&BulletinBoard>,
    readers: Query<(Entity, &ReadingBoard)>,
) {
    for (reader, reading_state) in readers.iter() {
        if let Ok(board) = boards.get(reading_state.board_entity) {
            if let Some(note) = board.notes.last() { // Read latest
                if let Some(target) = note.target {
                    let amount = match note.sentiment {
                        Sentiment::Positive => 5.0,
                        Sentiment::Negative => -5.0,
                        Sentiment::Neutral => 0.0,
                    };

                    if amount != 0.0 {
                        events.send(AffinityChange {
                            source: reader,
                            target,
                            amount,
                        });
                    }
                }
            }
        }
        // Remove state after reading
        commands.entity(reader).remove::<ReadingBoard>();
    }
}
```

## REFACTOR Phase: Quality & Design

- **Note Decay**: Add a `decay_notes_system` to remove old notes (e.g., > 3 days old).
- **Targeting**: Use `Relationships` to pick a target. A happy pop might praise their friend. An angry pop might slander their rival.
- **UI**: Display the notes in the Building Inspector when the board is selected.
- **Capacity**: Limit board to 10 notes. Oldest removed first.

## Acceptance Criteria

- [ ] `BulletinBoard` is a buildable structure.
- [ ] Pops with extreme morale generate notes on the board.
- [ ] Reading notes triggers `AffinityChange` events.
- [ ] Tests in `grievances_tests.rs` pass.

## Technical Guidance

- Integrate with `src/layer1/social/mod.rs` for `AffinityChange`.
- Ensure `BulletinBoard` is added to `BuildingType` match arms for `cost`, `label`, `char`, etc.
