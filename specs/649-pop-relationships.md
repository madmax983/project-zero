# Pop Relationships

## 1. Overview
**Layer:** 1
**Fantasy:** Watching a colony develop social fabric—friendships, rivalries, families.
**Mechanic:** Pops build relationship scores with pops they work alongside. High relationship = mood bonus. Low relationship = conflicts, productivity loss.

## 2. Dependencies
- Workplaces/Jobs, Needs, and Utility AI.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_pops_develop_relationships_working_together() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, update_workplace_relationships_system);

        let entity1 = app.world.spawn((
            Pop,
            CurrentJob { location_id: 1 },
            Relationships { bonds: Vec::new() },
        )).id();

        let entity2 = app.world.spawn((
            Pop,
            CurrentJob { location_id: 1 }, // Same location
            Relationships { bonds: Vec::new() },
        )).id();

        // Act
        app.update();

        // Assert
        let bonds1 = app.world.get::<Relationships>(entity1).unwrap();
        assert_eq!(bonds1.bonds.len(), 1);
        assert_eq!(bonds1.bonds[0].target, entity2);
        assert!(bonds1.bonds[0].score > 0);
    }

    #[test]
    fn test_high_relationships_boost_mood() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, calculate_relationship_mood_buff_system);

        let entity2 = app.world.spawn(Pop).id();

        let mut bonds = Vec::new();
        bonds.push(Bond { target: entity2, score: 50 }); // Friendly

        let entity1 = app.world.spawn((
            Pop,
            Relationships { bonds },
            Mood { current: 50, buff: 0 },
        )).id();

        // Act
        app.update();

        // Assert
        let mood = app.world.get::<Mood>(entity1).unwrap();
        assert!(mood.buff > 0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct CurrentJob {
    pub location_id: u32,
}

#[derive(Clone)]
pub struct Bond {
    pub target: Entity,
    pub score: i32,
}

#[derive(Component)]
pub struct Relationships {
    pub bonds: Vec<Bond>,
}

#[derive(Component)]
pub struct Mood {
    pub current: i32,
    pub buff: i32,
}

pub fn update_workplace_relationships_system(
    mut query: Query<(Entity, &CurrentJob, &mut Relationships)>
) {
    // Collect all workers and their locations
    let mut location_map = std::collections::HashMap::new();
    for (entity, job, _) in query.iter() {
        location_map.entry(job.location_id).or_insert_with(Vec::new).push(entity);
    }

    // Build relationships based on shared locations
    for (_, workers) in location_map.iter() {
        if workers.len() > 1 {
            for &worker in workers {
                for &peer in workers {
                    if worker != peer {
                        // Apply relationship gain
                        if let Ok((_, _, mut rels)) = query.get_mut(worker) {
                            let mut found = false;
                            for bond in rels.bonds.iter_mut() {
                                if bond.target == peer {
                                    bond.score += 1;
                                    found = true;
                                }
                            }
                            if !found {
                                rels.bonds.push(Bond { target: peer, score: 1 });
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn calculate_relationship_mood_buff_system(
    mut query: Query<(&Relationships, &mut Mood)>
) {
    for (rels, mut mood) in query.iter_mut() {
        let mut total_buff = 0;
        for bond in rels.bonds.iter() {
            if bond.score >= 50 {
                total_buff += 5; // Friendly buff
            } else if bond.score <= -50 {
                total_buff -= 5; // Rivalry debuff
            }
        }
        mood.buff = total_buff;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Re-use the existing spatial grid/zone system for job locations.
- Replace the expensive `O(n^2)` checking if thousands of Pops work together by creating an event-driven `ShiftEndEvent` that applies the relationship delta once per cycle, instead of every frame.
- Add negative bonds (Rivalries) that might trigger `FightEvent` or `Arguments`.
- Extract relationship thresholds to an Enum (e.g., `Friend`, `Rival`, `Acquaintance`).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops at the same workplace slowly increase their relationship score over time.
- [ ] High relationship scores grant a mood buff; low scores grant a debuff.

## 7. Technical Guidance
- Be careful with the `Relationships` vector growing too large. Consider capping the max number of bonds a Pop can maintain (e.g., Dunbar's number) to keep memory overhead low.

## 8. Questions
*Builder: add questions here if spec is unclear.*
