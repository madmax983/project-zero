use bevy_ecs::prelude::*;
use crate::layer1::items::ItemType;
use crate::layer1::map::GridPosition;
use crate::layer1::social_stratification::SocialClass;
use crate::layer1::morale::{Morale, MoodModifier};

/// Tracks the current "hot" item in the colony.
#[derive(Resource, Default, Debug)]
pub struct Trend {
    /// The item currently in fashion.
    pub current_item: Option<ItemType>,
    /// Strength of the trend (0.0 to 1.0). Decays over time.
    pub strength: f32,
    /// Position where the trend was started (used for proximity spread).
    pub setter_pos: GridPosition,
}

/// Component for pops indicating they want to mimic the Elite.
#[derive(Component, Default, Debug)]
pub struct SocialMimicry {
    /// The item this pop wants to consume to be trendy.
    pub desired_item: Option<ItemType>,
}

/// Component to track consumption events this tick.
/// Added by `consume_food_system` and removed by `clear_just_consumed_system`.
#[derive(Component)]
pub struct JustConsumed {
    /// The item that was just consumed.
    pub item: ItemType,
}

/// System: Elites set the trend when they consume items.
pub fn trend_setting_system(
    mut trend: ResMut<Trend>,
    query: Query<(&SocialClass, &GridPosition, &JustConsumed)>,
) {
    // Decay existing trend
    if trend.strength > 0.0 {
        trend.strength -= 0.01;
        if trend.strength <= 0.0 {
            trend.current_item = None;
            trend.strength = 0.0;
        }
    }

    // Check for new trend setters
    for (class, pos, consumed) in &query {
        if *class == SocialClass::Elite {
            // Elite sets the trend!
            trend.current_item = Some(consumed.item);
            trend.strength = 1.0; // Max strength
            trend.setter_pos = *pos;
            // Only one elite needs to set it per tick (first one wins for MVP)
            break;
        }
    }
}

/// System: Trends spread to non-elites (Labor/Middle).
pub fn trend_spread_system(
    trend: Res<Trend>,
    mut query: Query<(&GridPosition, &mut SocialMimicry), Without<crate::layer1::social_stratification::Prestige>>,
) {
    let weak_trend = trend.current_item.is_none() || trend.strength < 0.2;

    for (pos, mut mimicry) in &mut query {
        if weak_trend {
            // Trend died out, stop caring
            mimicry.desired_item = None;
            continue;
        }

        // Manhattan distance check
        let dist = (pos.x - trend.setter_pos.x).abs() + (pos.y - trend.setter_pos.y).abs();

        // If near the trend source (let's say 20 tiles is "gossip range")
        if dist < 20 {
             mimicry.desired_item = trend.current_item;
        }
    }
}

/// System: Apply mood buff if Pop consumes the trendy item.
#[allow(clippy::collapsible_if)]
pub fn trend_satisfaction_system(
    _commands: Commands,
    mut query: Query<(Entity, &SocialMimicry, &JustConsumed, Option<&mut Morale>)>,
) {
    for (_entity, mimicry, consumed, morale_opt) in &mut query {
        if let Some(desired) = mimicry.desired_item
            && consumed.item == desired
        {
            if let Some(mut morale) = morale_opt {
                morale.add_modifier(MoodModifier {
                    label: "Feeling Trendy".to_string(),
                    value: 0.1, // +10% morale
                    duration: 50, // Lasts for 50 ticks (short-ish)
                });
            }
        }
    }
}

/// System: Removes `JustConsumed` component at the end of the tick.
pub fn clear_just_consumed_system(
    mut commands: Commands,
    query: Query<Entity, With<JustConsumed>>,
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<JustConsumed>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::social_stratification::{SocialClass, Prestige};
    use crate::layer1::items::ItemType;
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;

    #[test]
    fn test_elite_sets_trend() {
        let mut world = World::new();
        world.init_resource::<Trend>();

        // Spawn Elite Pop consuming an item
        world.spawn((
            Pop,
            SocialClass::Elite,
            Prestige { value: 10 },
            GridPosition { x: 5, y: 5 },
            JustConsumed { item: ItemType::LuxuryMeal },
        ));

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
            setter_pos: GridPosition { x: 5, y: 5 },
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
            JustConsumed { item: ItemType::LuxuryMeal },
            Morale::default(),
        )).id();

        // Run satisfaction system
        let mut schedule = Schedule::default();
        schedule.add_systems(trend_satisfaction_system);
        schedule.run(&mut world);

        // Check Morale modifiers
        let morale = world.get::<Morale>(pop).unwrap();
        assert!(morale.modifiers.iter().any(|m| m.label == "Feeling Trendy"));
    }

    #[test]
    fn test_clear_just_consumed() {
        let mut world = World::new();
        let entity = world.spawn(JustConsumed { item: ItemType::Potato }).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(clear_just_consumed_system);
        schedule.run(&mut world);

        assert!(world.get::<JustConsumed>(entity).is_none());
    }
}
