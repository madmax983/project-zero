# 1203: Gravity-Tethered Inheritance

## 1. Overview
**Layer:** 2 -> 1
**Fantasy:** Physical possessions and inherited wealth literally raining down from the sky upon the death of an orbital aristocrat.
**Mechanic:** In systems with high-wealth Orbital Rings (Layer 2) and impoverished surface colonies (Layer 1), the death of a "Ring-Born" noble triggers a "Gravity Inheritance" event. Their physical belongings—often obsolete luxury goods or strange tech—are physically dropped via cargo pod to the surface to be claimed by their distant "Gravity-Bound" relatives.
**Emergence:** An orbital duke dies. His inheritance pod, filled with wildly inappropriate luxury goods (like zero-G art sculptures and exotic pets), crashes into a starving surface mining town. The miners fight violently over the useless art because it represents unimaginable wealth, completely ignoring the fact that they still have no food.
**Tension:** Dealing with the chaotic economic and social disruption caused by massive, unpredictable influxes of alien wealth dropping onto desperate, impoverished populations.

## 2. Dependencies
- Layer 2 Node Wealth levels / Noble deaths
- Layer 1 Drop Pod mechanics
- Layer 1 Item Economy (Luxury goods vs Survival goods)
- Pop Utility AI (Greed / Looting behaviors)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_noble_death_triggers_inheritance_drop() {
        let mut app = App::new();
        app.add_systems(Update, process_noble_death_system);

        app.world.insert_resource(Events::<NobleDeathEvent>::default());
        app.world.insert_resource(Events::<SpawnDropPodEvent>::default());

        // Send a noble death event
        let mut death_events = app.world.get_resource_mut::<Events<NobleDeathEvent>>().unwrap();
        death_events.send(NobleDeathEvent {
            wealth_level: 1000,
            target_surface_node: Entity::from_raw(1),
        });

        app.update();

        // Should have generated a drop pod event
        let pod_events = app.world.get_resource::<Events<SpawnDropPodEvent>>().unwrap();
        let mut reader = pod_events.get_cursor();
        let events: Vec<_> = reader.read(pod_events).collect();

        assert_eq!(events.len(), 1, "Death of a noble should trigger an inheritance drop pod");
        assert_eq!(events[0].contents.item_type, ItemType::LuxuryGoods, "Inheritance should contain luxury items");
    }

    #[test]
    fn test_pops_prioritize_luxury_loot_over_needs() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_loot_utility_system);

        // Pop with high hunger
        let pop_id = app.world.spawn((Pop, Hunger(80.0), ActionQueue::default())).id();

        // A drop pod with luxury goods lands nearby
        let drop_pod = app.world.spawn((DropPod, Item { item_type: ItemType::LuxuryGoods, value: 5000 })).id();

        app.update();

        // Pop should enqueue a Loot action despite being hungry, due to immense wealth value
        let action_queue = app.world.get::<ActionQueue>(pop_id).unwrap();
        assert!(
            action_queue.actions.iter().any(|a| matches!(a, Action::Loot(_))),
            "Pops should prioritize looting extreme wealth over basic needs"
        );
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Event)]
pub struct NobleDeathEvent {
    pub wealth_level: u32,
    pub target_surface_node: Entity,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ItemType {
    Food,
    LuxuryGoods,
}

#[derive(Clone)]
pub struct ItemStack {
    pub item_type: ItemType,
    pub value: u32,
}

#[derive(Event)]
pub struct SpawnDropPodEvent {
    pub target: Entity,
    pub contents: ItemStack,
}

#[derive(Component)]
pub struct DropPod;

#[derive(Component)]
pub struct Item {
    pub item_type: ItemType,
    pub value: u32,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Hunger(pub f32);

#[derive(Clone)]
pub enum Action {
    Idle,
    Loot(Entity),
    Eat,
}

#[derive(Component, Default)]
pub struct ActionQueue {
    pub actions: Vec<Action>,
}

pub fn process_noble_death_system(
    mut death_events: EventReader<NobleDeathEvent>,
    mut drop_pod_events: EventWriter<SpawnDropPodEvent>,
) {
    for event in death_events.read() {
        drop_pod_events.send(SpawnDropPodEvent {
            target: event.target_surface_node,
            contents: ItemStack {
                item_type: ItemType::LuxuryGoods,
                value: event.wealth_level * 10,
            },
        });
    }
}

pub fn evaluate_loot_utility_system(
    mut pops: Query<(&mut ActionQueue, &Hunger), With<Pop>>,
    loot: Query<(Entity, &Item), With<DropPod>>,
) {
    for (mut queue, hunger) in pops.iter_mut() {
        for (loot_entity, item) in loot.iter() {
            // Simplified utility check: If loot is extremely valuable, drop everything and loot it
            if item.value > 1000 && hunger.0 < 95.0 {
                queue.actions.push(Action::Loot(loot_entity));
                break;
            } else if hunger.0 >= 95.0 {
                // Only eat if literally starving to death
                queue.actions.push(Action::Eat);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate Drop Trajectory: Drop pods shouldn't instantly appear. They should have a travel time, allowing the player to intercept them or Pops to gather at the impact site.
- Combine the utility evaluation into the core `src/layer1/utility_ai.rs` instead of a standalone system, weighting the `Item.value` against the current `Hunger` score.
- Introduce `SocialConflictEvent`s if multiple Pops attempt to loot the same highly valuable drop pod.

## 6. Acceptance Criteria
- [ ] `NobleDeathEvent` successfully triggers a luxury `SpawnDropPodEvent`.
- [ ] Pops prioritize looting extreme wealth over mid-level survival needs.
- [ ] All RED phase tests pass.
- [ ] Coverage >= 85%.
- [ ] 0 clippy warnings.

## 7. Technical Guidance
- Be sure to update `Events::get_cursor()` for Bevy 0.15 compat in tests.
- This creates a massive spike in colony wealth. Ensure the market simulation can handle rapid inflation or that `LuxuryGoods` can be sold.

## 8. Questions
*Builder: Add any questions here.*
