use crate::layer1::inventory::{Inventory, InventoryItem};
use crate::layer1::items::ItemType;
use crate::layer1::stress::StressTracker;
use crate::layer1::traits::Trait;
use crate::layer1::utility_ai::{ActionType, PopAction};
use crate::layer1::utility_eval_types::PopEvalData;
use crate::layer1::utility_types::HobbyType;
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Component, Debug, Clone)]
pub struct Hobby {
    pub hobby_type: HobbyType,
}

// Logic to pick hobby from Traits component
fn pick_hobby_for_traits(traits: &crate::layer1::traits::Traits, rng: &mut impl Rng) -> HobbyType {
    // Priority check
    if traits.0.contains(&Trait::HardWorker) || traits.0.contains(&Trait::Curious) {
        return HobbyType::Tinkering;
    }
    if traits.0.contains(&Trait::Lazy) || traits.0.contains(&Trait::NativeBorn) {
        return HobbyType::CloudWatching;
    }
    if traits.0.contains(&Trait::Ascetic) || traits.0.contains(&Trait::Traditionalist) {
        return HobbyType::Meditation;
    }
    if traits.0.contains(&Trait::Greedy) || traits.0.contains(&Trait::Optimist) {
        // Mapping Social to Gossip?
        return HobbyType::Gossip;
    }

    // Creative? `Trait::Creative` doesn't exist in `traits.rs`.
    // I see `HardWorker`, `Lazy`, `Glutton`, `Ascetic`, `NightOwl`, `EarlyBird`, `FastWalker`, `Greedy`, `Anxious`, `NativeBorn`, `WeakImmunity`, `Pyromaniac`, `Feral`, `Optimist`, `Curious`, `Traditionalist`.

    // Mapping:
    // Tinkering: Curious, HardWorker
    // CloudWatching: Lazy, Feral
    // Meditation: Ascetic, Traditionalist
    // Gossip: Greedy, Anxious (Sharing fears/greed?)
    // Whittling: Default / Optimist?

    let candidates = [
        HobbyType::Whittling,
        HobbyType::CloudWatching,
        HobbyType::Meditation,
        HobbyType::Gossip,
        HobbyType::Tinkering,
    ];

    candidates[rng.gen_range(0..candidates.len())]
}

pub fn assign_hobby_system(
    mut commands: Commands,
    query: Query<(Entity, &crate::layer1::traits::Traits), Without<Hobby>>,
) {
    let mut rng = rand::thread_rng();
    for (entity, traits) in &query {
        let hobby_type = pick_hobby_for_traits(traits, &mut rng);
        commands.entity(entity).insert(Hobby { hobby_type });
    }
}

pub fn evaluate_hobby(data: &PopEvalData, _hobby_type: HobbyType) -> f32 {
    let base_score = 0.1;
    let stress_factor = data.stress * 0.8; // Up to 0.8 from stress

    // Idle bonus? `PopEvalData` has `action: PopAction`.
    let is_idle = data.action.current == ActionType::Idle;
    let idle_bonus = if is_idle { 0.3 } else { 0.0 };

    (base_score + stress_factor + idle_bonus).min(1.0)
}

pub fn execute_hobby_system(
    mut query: Query<(
        &mut StressTracker,
        &Hobby,
        Option<&mut Inventory>,
        &PopAction,
    )>,
) {
    let mut rng = rand::thread_rng();

    for (mut stress, hobby, inventory, action) in &mut query {
        if action.current != ActionType::Hobby {
            continue;
        }

        // Reduce stress
        stress.accumulated_stress = (stress.accumulated_stress - 0.5).max(0.0); // 0.5 per tick is fast? Spec said 0.005.
        // Spec: `stress.value = (stress.value - 0.005).max(0.0);` where stress was 0-1.
        // `StressTracker` is 0-100. So 0.5 is 0.5%.
        // Let's use 0.5.

        // Chance to produce item (e.g., 1% per tick)
        if rng.gen_bool(0.01) {
            if let Some(mut inv) = inventory {
                let item_type = match hobby.hobby_type {
                    HobbyType::Whittling => Some(ItemType::Curio("Wooden Duck".to_string())),
                    HobbyType::Tinkering => Some(ItemType::Curio("Bent Gear".to_string())),
                    _ => None,
                };

                if let Some(t) = item_type {
                    inv.add(InventoryItem { item_type: t });
                }
            }
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::inventory::{Inventory, InventoryItem};
    use crate::layer1::items::ItemType;
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::utility_ai::UtilityWeights;
    use crate::layer1::utility_ai::{ActionType, PopAction};
    use crate::layer1::utility_eval_types::PopEvalData;
    use bevy_ecs::prelude::*;
    use std::collections::HashSet;

    #[test]
    fn test_pop_assigned_hobby_based_on_trait() {
        let mut world = World::new();

        // Spawn pop with Curious trait (should get Tinkering)
        let pop = world
            .spawn((Pop, Traits(HashSet::from([Trait::Curious]))))
            .id();

        // Run assignment system
        let mut schedule = Schedule::default();
        schedule.add_systems(assign_hobby_system);
        schedule.run(&mut world);

        // Check hobby
        let hobby = world.get::<Hobby>(pop).unwrap();
        assert_eq!(hobby.hobby_type, HobbyType::Tinkering);
    }

    #[test]
    fn test_evaluate_hobby_high_when_stressed() {
        let mut eval_data = PopEvalData {
            entity: Entity::from_raw(0),
            pos: GridPosition { x: 0, y: 0 },
            needs: Needs::default(),
            weights: UtilityWeights::default(),
            action: PopAction {
                current: ActionType::Idle,
                ..Default::default()
            }, // Idle
            equipment: None,
            carrying: None,
            mental_state: None,
            drafted: None,
            faction_member: None,
            penal_labor: None,
            breakdown: None,
            traits: None,
            stress: 0.8, // High stress
            hobby_type: Some(HobbyType::CloudWatching),
        };

        let score = evaluate_hobby(&eval_data, HobbyType::CloudWatching);
        assert!(score > 0.6, "Stressed pop should want to do hobby");
    }

    #[test]
    fn test_hobby_execution_reduces_stress() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 50.0,
                },
                Hobby {
                    hobby_type: HobbyType::Meditation,
                },
                PopAction {
                    current: ActionType::Hobby,
                    ..Default::default()
                },
            ))
            .id();

        // Simulate execution tick
        let mut schedule = Schedule::default();
        schedule.add_systems(execute_hobby_system);
        schedule.run(&mut world);

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(
            stress.accumulated_stress < 50.0,
            "Hobby should reduce stress"
        );
    }

    #[test]
    fn test_hobby_produces_curio() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                Hobby {
                    hobby_type: HobbyType::Whittling,
                },
                Inventory::default(),
                StressTracker::default(),
                PopAction {
                    current: ActionType::Hobby,
                    ..Default::default()
                },
            ))
            .id();

        // Force production trigger by running enough times or mocking RNG?
        // Since we can't easily mock RNG in the system without DI, we loop.
        // 1% chance -> 500 loops should trigger it.
        let mut schedule = Schedule::default();
        schedule.add_systems(execute_hobby_system);

        let mut produced = false;
        for _ in 0..1000 {
            schedule.run(&mut world);
            let inventory = world.get::<Inventory>(pop).unwrap();
            if inventory
                .items
                .iter()
                .any(|i| matches!(i.item_type, ItemType::Curio(_)))
            {
                produced = true;
                break;
            }
        }

        assert!(produced, "Whittling should produce a Curio eventually");
    }
}
