# 441: The Sleepwalker's Guild

## 1. Overview
Extreme chronic Stress and overuse of stimulants or Hypno-Learning can cause Pops to develop "Productive Somnambulism". They continue to perform their jobs while their `Rest` need technically regenerates, but they have zero awareness of their surroundings, ignore hazards, and drop items randomly. This creates a tension between effectively doubling workforce efficiency and dealing with complete unpredictability and massive safety hazards.

## 2. Dependencies
- `009` Job System
- `127` Stress Breakdowns
- `038` Medical Care (Stimulants/Hypno-Learning context)
- `016` Utility AI System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::{Pop, Rest, Stress};
    use crate::layer1::jobs::{JobQueue, Worker};
    use crate::layer1::hazards::Hazard;

    #[test]
    fn test_sleepwalking_trigger() {
        let mut world = World::new();
        let entity = world.spawn((
            Pop,
            Stress { value: 95.0 }, // Extreme stress
            Rest { value: 10.0 }, // Very tired
            StimulantUsage { count: 5 } // High stimulant use
        )).id();

        let mut app = App::new();
        app.add_system(check_sleepwalking_trigger);
        app.update();

        assert!(world.get::<Sleepwalking>(entity).is_some());
    }

    #[test]
    fn test_sleepwalker_works_while_resting() {
        let mut world = World::new();
        let entity = world.spawn((
            Pop,
            Worker,
            Rest { value: 0.0 },
            Sleepwalking,
            CurrentJob { id: JobId(1) }
        )).id();

        // Run job processing and rest regeneration
        let mut app = App::new();
        app.add_system(process_jobs);
        app.add_system(regenerate_rest_for_sleepwalkers);
        app.update();

        // Should have progressed job AND regenerated rest
        let rest = world.get::<Rest>(entity).unwrap();
        assert!(rest.value > 0.0);
        let job = world.get::<CurrentJob>(entity).unwrap();
        assert!(job.progress > 0.0);
    }

    #[test]
    fn test_sleepwalker_ignores_hazards() {
        let mut world = World::new();
        // Create a hazard
        world.spawn((Hazard { damage: 10.0 }, GridPosition { x: 5, y: 5 }));

        let entity = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Sleepwalking,
            Health { value: 100.0 }
        )).id();

        let mut app = App::new();
        app.add_system(apply_hazard_damage);
        app.update();

        // Sleepwalker still takes damage, but their AI didn't path around it (simulated by them standing in it)
        let health = world.get::<Health>(entity).unwrap();
        assert_eq!(health.value, 90.0);

        // Test that Utility AI assigns hazard-pathing a lower penalty for sleepwalkers
        let path_cost = calculate_path_cost(&world, entity, GridPosition { x: 5, y: 5 });
        // Standard pop path cost would be high due to hazard, sleepwalker cost is normal distance
        assert_eq!(path_cost, 0); // 0 extra cost for hazard
    }

    #[test]
    fn test_sleepwalker_drops_items() {
        let mut world = World::new();
        let entity = world.spawn((
            Pop,
            Sleepwalking,
            Inventory { items: vec![Item::Ore] }
        )).id();

        let mut app = App::new();
        // Probability system for dropping items
        app.insert_resource(RandomDropChance(1.0)); // Force drop
        app.add_system(sleepwalker_drop_items);
        app.update();

        let inventory = world.get::<Inventory>(entity).unwrap();
        assert!(inventory.items.is_empty());
        // Item should be spawned on the ground
        assert_eq!(world.query::<&ItemOnGround>().iter(&world).count(), 1);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Sleepwalking;

#[derive(Component)]
pub struct StimulantUsage {
    pub count: u32,
}

pub fn check_sleepwalking_trigger(
    mut commands: Commands,
    query: Query<(Entity, &Stress, &StimulantUsage), Without<Sleepwalking>>
) {
    for (entity, stress, stims) in query.iter() {
        if stress.value > 90.0 && stims.count > 3 {
            commands.entity(entity).insert(Sleepwalking);
        }
    }
}

pub fn regenerate_rest_for_sleepwalkers(
    mut query: Query<&mut Rest, With<Sleepwalking>>,
    time: Res<Time>,
) {
    for mut rest in query.iter_mut() {
        // Regenerate rest even while active
        rest.value += 5.0 * time.delta_seconds();
        rest.value = rest.value.min(100.0);
    }
}

// In Utility AI pathfinding integration:
pub fn calculate_path_cost(world: &World, entity: Entity, pos: GridPosition) -> i32 {
    let mut cost = 0; // Base distance cost calculated elsewhere
    if world.get::<Hazard>(/* entity at pos */).is_some() {
        if world.get::<Sleepwalking>(entity).is_none() {
            cost += 1000; // Normal pops avoid hazards
        }
    }
    cost
}

pub fn sleepwalker_drop_items(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Inventory, &GridPosition), With<Sleepwalking>>,
    chance: Res<RandomDropChance>,
) {
    for (entity, mut inventory, pos) in query.iter_mut() {
        if chance.0 > 0.5 && !inventory.items.is_empty() {
            let item = inventory.items.pop().unwrap();
            commands.spawn((ItemOnGround(item), *pos));
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate `Sleepwalking` directly into the `UtilityAISystem` to alter scorer weights (e.g., set `AvoidHazardScorer` to 0.0 weight).
- The drop chance should be tied to `Time` delta to ensure framerate independence.
- Consider an event `SleepwalkingStartedEvent` to trigger notifications to the player.
- Make sure sleepwalkers cannot perform precision jobs (like surgery).

## 6. Acceptance Criteria
- [ ] `Sleepwalking` component is added to Pops with high stress and stimulant use.
- [ ] Sleepwalkers slowly regenerate `Rest` while performing regular tasks.
- [ ] Sleepwalking Pops do not avoid Hazards in pathfinding.
- [ ] Sleepwalking Pops have a random chance to drop items from their inventory onto the tile they are standing on.
- [ ] Test coverage for the new module is ≥85%.

## 7. Technical Guidance
- The pathfinding logic for ignoring hazards should hook into whatever A* or flow field system the Utility AI uses.
- Ensure that the random drop doesn't happen every frame; use a timer or a per-tick probability calculation that scales with `Time::delta_seconds()`.

## 8. Questions
