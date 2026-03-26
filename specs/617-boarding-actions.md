# 617: Boarding Actions

## 1. Overview
Close-quarters combat in the corridors of a dying ship. When an enemy ship is disabled in Layer 2 (System map), players can launch a "Breaching Pod". This pod transports a squad of Layer 1 units into a small, temporary procedural Layer 1 map representing the interior of the disabled ship. Once boarded, players must clear the ship of defenders to secure unique loot or face the consequences of discovering refugees or a trap.

## 2. Dependencies
- Layer 2 Fleet Combat System
- Layer 1 Combat System (for squad mechanics)
- Layer 1 Procedural Map Generation System (for ship interiors)
- Cross-Layer Communication (Event/Message passing)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use layer2::combat::ShipDisabledEvent;
    use layer1::squads::{BreachingPod, Squad};
    use layer1::map::ProceduralMapGenEvent;

    #[test]
    fn test_boarding_pod_launches_on_disabled_ship() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<ShipDisabledEvent>();
        app.add_event::<ProceduralMapGenEvent>();
        app.add_systems(Update, handle_boarding_action_system);

        // Disable a ship
        let ship_entity = app.world_mut().spawn(()).id();
        app.world_mut().send_event(ShipDisabledEvent { ship: ship_entity });

        let squad_entity = app.world_mut().spawn((Squad, BreachingPod)).id();

        app.update();

        // Ensure a procedural map generation event was triggered for the squad
        let map_events = app.world().resource::<Events<ProceduralMapGenEvent>>();
        assert!(map_events.get_reader().read(&map_events).next().is_some(), "Procedural map generation should be triggered for boarding action");
    }

    #[test]
    fn test_boarding_success_yields_loot() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<BoardingCompletedEvent>();
        app.add_systems(Update, handle_boarding_loot_system);

        let loot_stash = app.world_mut().spawn(LootStash { amount: 0 }).id();
        app.world_mut().send_event(BoardingCompletedEvent { success: true, reward_amount: 50 });

        app.update();

        let amount = app.world().get::<LootStash>(loot_stash).unwrap().amount;
        assert_eq!(amount, 50, "Successful boarding should yield the reward amount");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Event)]
pub struct ShipDisabledEvent {
    pub ship: Entity,
}

#[derive(Event)]
pub struct ProceduralMapGenEvent {
    pub target: Entity,
    pub seed: u64,
}

#[derive(Component)]
pub struct Squad;

#[derive(Component)]
pub struct BreachingPod;

#[derive(Event)]
pub struct BoardingCompletedEvent {
    pub success: bool,
    pub reward_amount: u32,
}

#[derive(Component)]
pub struct LootStash {
    pub amount: u32,
}

pub fn handle_boarding_action_system(
    mut disabled_events: EventReader<ShipDisabledEvent>,
    mut map_events: EventWriter<ProceduralMapGenEvent>,
    squad_query: Query<Entity, (With<Squad>, With<BreachingPod>)>,
) {
    for event in disabled_events.read() {
        if let Ok(squad) = squad_query.get_single() {
            // Trigger a procedural ship map generation
            map_events.send(ProceduralMapGenEvent { target: squad, seed: 12345 });
        }
    }
}

pub fn handle_boarding_loot_system(
    mut boarding_events: EventReader<BoardingCompletedEvent>,
    mut loot_query: Query<&mut LootStash>,
) {
    for event in boarding_events.read() {
        if event.success {
            for mut stash in loot_query.iter_mut() {
                stash.amount += event.reward_amount;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate map generation seamlessly, possibly switching the active camera/viewport to the newly generated Layer 1 ship map.
- Handle varying loot types instead of a generic `u32` integer amount.
- Ensure squad entities are moved effectively from their origin map to the target ship map and back.
- Properly despawn the temporary Layer 1 map upon completion (success or squad wipe).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Disabling a ship correctly fires an event leading to procedural map generation.
- [ ] Completing the boarding action successfully increments colonial loot stashes.

## 7. Technical Guidance
- **Cross-Layer Data:** Passing state between Layer 2 combat and the Layer 1 instance requires careful tracking of entity IDs so that returning squads carry the correct state (wounds, ammo, loot).
- **Procedural Generation:** Use existing procedural room/dungeon generation components to create tight, modular corridors reminiscent of a ship interior.
- **Victory Condition:** Ensure a reliable system determines when the ship is "cleared" (e.g., all hostiles dead, core hacked, specific cargo grabbed).

## 8. Questions
*Builder: add questions here if spec is unclear.*
