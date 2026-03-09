# 128: Social Mimicry

## Overview

Introduces a dynamic fashion system where Pops copy the behaviors of high-status individuals. If a member of the Elite class consumes a specific luxury item or performs a specific leisure activity, it becomes a **Trend**. Lower-class Pops who witness this will develop a desire to mimic the behavior. Following a Trend grants a Mood bonus ("Feeling Trendy"), while inability to follow it may cause stress or envy.

This adds emergent economic pressure: the player cannot just feed the Elite efficient food; they must supply the masses with whatever the Elite are eating, or face unrest.

## Dependencies

- `113` — Social Stratification (Defines Elite/Prestige)
- `005` — Pop Needs (Consumable items)
- `031` — Pop Morale (Mood impacts)
- `047` — Pop Relationships (Proximity/Observation)

## RED Phase: Tests First

Write these tests in `src/layer1/social_mimicry_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::social_stratification::{SocialClass, Prestige};
    use crate::layer1::pop::{Pop, PopName};
    use crate::layer1::needs::{Need, NeedType}; // Assuming consumption triggers need fulfillment
    use crate::layer1::items::ItemType; // Enum for items (e.g., RatOnStick, LuxuryMeal)
    use crate::layer1::social_mimicry::{Trend, SocialMimicry, trend_setting_system, trend_spread_system, trend_satisfaction_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;

    #[test]
    fn test_elite_sets_trend() {
        let mut world = World::new();
        world.init_resource::<Trend>(); // Global trend resource or per-colony? Let's assume Global for MVP.

        // Spawn Elite Pop consuming an item
        let elite = world.spawn((
            Pop,
            SocialClass::Elite,
            Prestige { value: 10 },
            GridPosition { x: 5, y: 5 },
            // Component indicating recent consumption - normally added by action system
            crate::layer1::actions::JustConsumed { item: ItemType::LuxuryMeal },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(trend_setting_system);
        schedule.run(&mut world);

        // Check Trend resource
        let trend = world.resource::<Trend>();
        assert_eq!(trend.current_item, Some(ItemType::LuxuryMeal));
        assert!(trend.strength > 0.0);
    }

    #[test]
    fn test_trend_spreads_to_nearby_labor() {
        let mut world = World::new();
        world.insert_resource(Trend {
            current_item: Some(ItemType::LuxuryMeal),
            strength: 1.0,
            setter_pos: GridPosition { x: 5, y: 5 }, // Position of the trend setter
        });

        // Spawn Laborer nearby
        let laborer = world.spawn((
            Pop,
            SocialClass::Labor,
            GridPosition { x: 6, y: 5 }, // Adjacent
            SocialMimicry::default(),
        )).id();

        // Run spread system
        let mut schedule = Schedule::default();
        schedule.add_systems(trend_spread_system);
        schedule.run(&mut world);

        // Laborer should adopt the trend
        let mimicry = world.get::<SocialMimicry>(laborer).unwrap();
        assert_eq!(mimicry.desired_item, Some(ItemType::LuxuryMeal));
    }

    #[test]
    fn test_following_trend_grants_mood_buff() {
        let mut world = World::new();

        // Spawn Pop who wants LuxuryMeal and just ate it
        let pop = world.spawn((
            Pop,
            SocialMimicry { desired_item: Some(ItemType::LuxuryMeal) },
            crate::layer1::actions::JustConsumed { item: ItemType::LuxuryMeal },
            Morale::default(),
        )).id();

        // Run satisfaction system
        let mut schedule = Schedule::default();
        schedule.add_systems(trend_satisfaction_system);
        schedule.run(&mut world);

        // Check Morale (assuming modifier system) or Memory
        // For MVP, checking if a "Trendy" memory or buff was added
        let morale = world.get::<Morale>(pop).unwrap();
        // Assuming we can check specific modifiers or just raw value increase
        // assert!(morale.has_modifier("Trendy"));
        // OR simply:
        // assert!(world.get::<crate::layer1::memory::Memories>(pop).unwrap().contains(MemoryType::FollowedTrend));
    }

    #[test]
    fn test_trend_decay() {
        let mut world = World::new();
        world.insert_resource(Trend {
            current_item: Some(ItemType::RatOnStick),
            strength: 0.1,
            setter_pos: GridPosition::default(),
        });

        // Run decay system (part of setting system or separate)
        // crate::layer1::social_mimicry::trend_decay_system(&mut world);

        // Strength should drop to 0 and item clear
        // let trend = world.resource::<Trend>();
        // assert!(trend.strength <= 0.0);
        // assert!(trend.current_item.is_none());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer1/social_mimicry.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::items::ItemType;
use crate::layer1::map::GridPosition;
use crate::layer1::social_stratification::SocialClass;

#[derive(Resource, Default, Debug)]
pub struct Trend {
    pub current_item: Option<ItemType>,
    pub strength: f32, // 0.0 to 1.0, decays over time
    pub setter_pos: GridPosition, // Where the trend started (for proximity check)
}

#[derive(Component, Default, Debug)]
pub struct SocialMimicry {
    pub desired_item: Option<ItemType>,
}

// Temporary component to track consumption events (to be integrated with Action system)
#[derive(Component)]
pub struct JustConsumed {
    pub item: ItemType,
}

pub fn trend_setting_system(
    mut trend: ResMut<Trend>,
    query: Query<(&SocialClass, &GridPosition, &JustConsumed)>,
) {
    // Decay existing trend
    trend.strength -= 0.01;
    if trend.strength <= 0.0 {
        trend.current_item = None;
        trend.strength = 0.0;
    }

    for (class, pos, consumed) in query.iter() {
        if *class == SocialClass::Elite {
            // Elite sets the trend!
            trend.current_item = Some(consumed.item);
            trend.strength = 1.0; // Max strength
            trend.setter_pos = *pos;
            // Only one elite needs to set it per tick (or average them?)
            // First one wins for MVP.
            break;
        }
    }
}

pub fn trend_spread_system(
    trend: Res<Trend>,
    mut query: Query<(&GridPosition, &mut SocialMimicry), Without<crate::layer1::social_stratification::Prestige>>, // Only non-elites mimic? Or everyone mimics elites?
) {
    if trend.current_item.is_none() || trend.strength < 0.2 {
        return;
    }

    for (pos, mut mimicry) in query.iter_mut() {
        // Distance check
        let dist = (pos.x - trend.setter_pos.x).abs() + (pos.y - trend.setter_pos.y).abs();

        // If near the trend source (or just generically in the colony if 'Global' trend)
        // For MVP, let's say "News travels", so proximity to setter isn't strict,
        // BUT visually witnessing it is better.
        // Let's use a Colony-wide broadcast for simplicity in GREEN phase.

        if dist < 20 { // Within "gossip" range
             mimicry.desired_item = trend.current_item;
        }
    }
}

pub fn trend_satisfaction_system(
    mut commands: Commands,
    query: Query<(Entity, &SocialMimicry, &JustConsumed)>,
) {
    for (entity, mimicry, consumed) in query.iter() {
        if let Some(desired) = mimicry.desired_item {
            if consumed.item == desired {
                // Grant Mood Bonus
                // commands.entity(entity).insert(MoodModifier::new("Trendy", 10.0, 50));
                // Or Memory
                // commands.entity(entity).insert(Memory::new(MemoryType::FollowedTrend));
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Broadcasting**: Instead of `setter_pos`, trends should spread pop-to-pop via the `Rumor` system (`055`).
- **Multiple Trends**: `Trend` resource should hold a `Vec<TrendItem>` or `HashMap<ItemType, Strength>` to allow competing trends.
- **Visuals**: Pops following a trend could have a visual effect (particle?) or a thought bubble showing the item.
- **UI**: "Current Trends" panel in the faction/colony screen.
- **Resistance**: `PopTraits` (e.g., `Nonconformist`) should prevent mimicry.

## Acceptance Criteria

- [ ] `Trend` resource tracks the current "hot" item.
- [ ] Elite consumption updates the Trend.
- [ ] Pops adopt the Trend as `desired_item`.
- [ ] Consuming the `desired_item` triggers a positive effect (Mood/Memory).
- [ ] Trends decay over time if not reinforced.
- [ ] Tests pass.

## Technical Guidance

- Integrate `JustConsumed` logic into `actions::eat::finish_eating`.
- Ensure `Trend` strength decay is tuned so trends last a few days (game time), not seconds.
- Consider `SocialClass::Middle` as partial trend-setters (weaker strength).

## Questions

- *Do trends apply to clothing/activities too?* (Yes, but start with Consumables for MVP).
  - *Architect:* Yes, but limit the MVP to consumables only.
