# 640 - Psychic Insulation

## 1. Overview
**Layer:** 1
**Fantasy:** Some thoughts are too loud, and some memories are too sharp. Sometimes, the only way to stay sane is to build a wall against the minds of others.
**Mechanic:** Certain rare materials (e.g., "Null-Stone") can be used to construct rooms that block all "Resonance," "Empathy," and "Rumor" mechanics. Pops inside these rooms cannot be affected by the moods, memories, or telepathic traits of others, but they also cannot gain positive social buffs.

## 2. Dependencies
- Layer 1 Room/Zone system.
- Pop `Mood`, `Memory`, or `Rumor` propagation systems.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_null_stone_blocks_negative_rumors() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, spread_rumor_system);

        // Spawn a pop inside a Null-Stone room
        let protected_pop = app.world.spawn((
            Pop,
            Mood { current: 50 },
            Position { x: 5, y: 5, z: 0 },
            InZone(ZoneType::NullStone),
        )).id();

        // Spawn an adjacent pop with a terrible rumor
        let _gossiper = app.world.spawn((
            Pop,
            HasRumor(RumorType::Despair),
            Position { x: 6, y: 5, z: 0 },
        )).id();

        // Act
        app.update();

        // Assert: Protected pop did NOT receive the rumor/mood debuff
        assert!(app.world.get::<HasRumor>(protected_pop).is_none());
        let mood = app.world.get::<Mood>(protected_pop).unwrap();
        assert_eq!(mood.current, 50); // Mood unchanged
    }

    #[test]
    fn test_null_stone_blocks_positive_empathy() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_empathy_buff_system);

        let protected_pop = app.world.spawn((
            Pop,
            Mood { current: 50 },
            Position { x: 5, y: 5, z: 0 },
            InZone(ZoneType::NullStone),
        )).id();

        let _empath = app.world.spawn((
            Pop,
            Traits { traits: vec!["Empathic".to_string()] },
            Position { x: 6, y: 5, z: 0 },
        )).id();

        // Act
        app.update();

        // Assert: Protected pop did NOT receive the empathy buff
        let mood = app.world.get::<Mood>(protected_pop).unwrap();
        assert_eq!(mood.current, 50); // Mood unchanged
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component, PartialEq, Eq)]
pub enum ZoneType {
    Normal,
    NullStone,
}

#[derive(Component)]
pub struct InZone(pub ZoneType);

// Modify existing systems to check for NullStone
pub fn spread_rumor_system(
    mut commands: Commands,
    gossiper_query: Query<(&Position, &HasRumor)>,
    target_query: Query<(Entity, &Position, Option<&InZone>), Without<HasRumor>>,
) {
    for (g_pos, rumor) in gossiper_query.iter() {
        for (t_entity, t_pos, in_zone) in target_query.iter() {
            if g_pos.distance(t_pos) <= 1 {
                // Check for Psychic Insulation
                if let Some(zone) = in_zone {
                    if zone.0 == ZoneType::NullStone {
                        continue;
                    }
                }
                commands.entity(t_entity).insert(HasRumor(rumor.0.clone()));
            }
        }
    }
}

pub fn apply_empathy_buff_system(
    empath_query: Query<&Position, With<Traits>>, // Assume Traits has Empathic
    mut target_query: Query<(&mut Mood, &Position, Option<&InZone>)>,
) {
    for e_pos in empath_query.iter() {
        for (mut mood, t_pos, in_zone) in target_query.iter_mut() {
            if e_pos.distance(t_pos) <= 5 {
                if let Some(zone) = in_zone {
                    if zone.0 == ZoneType::NullStone {
                        continue;
                    }
                }
                mood.current = (mood.current + 5).min(100);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells:** We shouldn't scatter `ZoneType::NullStone` checks across all social systems. Instead, we should give protected pops a generic `PsychicallyInsulated` component when they enter the zone, and check for that instead.
- **Performance:** `spread_rumor_system` is O(N^2) if not careful. Should use spatial hashing if the pop count is high.
- **API Improvements:** Create a centralized `can_receive_social_interaction(entity)` function.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops in Null-Stone zones ignore both positive and negative psychic/social effects.

## 7. Technical Guidance
- Add the new ZoneType to `src/layer1/zones.rs` or `rooms.rs`.
- Check where `Rumor` and `Mood` are updated. You may need to intercept events rather than just modifying queries if the current system uses Bevy Events for propagation.

## 8. Questions
*Builder: add questions here if spec is unclear.*
