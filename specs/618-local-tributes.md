# 618: Local Tributes

## 1. Overview
We are not the masters of this world; we are guests. And the landlord is hungry. A massive local entity (Leviathan, Ancient AI, Hive Mind) demands periodic "Tribute" (Food, Energy, Pops). Appeasement grants protection/buffs. Refusal triggers attacks or disasters. The tension lies in fighting the monster (high risk/cost) versus feeding the monster (moral/resource cost). This feature implements the `LocalTributeSystem` and `LeviathanEntity` in Layer 1.

## 2. Dependencies
- Base resource systems (Food, Energy).
- Pop management system.
- Event/Disaster spawning system.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_leviathan_demands_tribute_on_timer() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, leviathan_tribute_system);

        let leviathan = app.world_mut().spawn(Leviathan {
            tribute_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
            current_demand: None,
            anger_level: 0,
        }).id();

        // Act
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs(11));
        app.update();

        // Assert
        let leviathan_comp = app.world().get::<Leviathan>(leviathan).unwrap();
        assert!(leviathan_comp.current_demand.is_some(), "Leviathan should have generated a demand after the timer elapsed");
    }

    #[test]
    fn test_appeasing_leviathan_grants_buff_and_resets_anger() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, leviathan_appeasement_system);

        let mut inventory = Inventory::new();
        inventory.add(ItemType::Food, 500);
        app.insert_resource(inventory);

        let leviathan = app.world_mut().spawn(Leviathan {
            tribute_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
            current_demand: Some(TributeDemand { item: ItemType::Food, amount: 500 }),
            anger_level: 50,
        }).id();

        // Act
        // Simulate player clicking "Pay Tribute"
        app.world_mut().send_event(PayTributeEvent { leviathan_id: leviathan });
        app.update();

        // Assert
        let inventory = app.world().resource::<Inventory>();
        assert_eq!(inventory.get(ItemType::Food), 0, "Inventory should be depleted by tribute amount");

        let leviathan_comp = app.world().get::<Leviathan>(leviathan).unwrap();
        assert!(leviathan_comp.current_demand.is_none(), "Demand should be cleared");
        assert_eq!(leviathan_comp.anger_level, 0, "Anger should be reset to 0 upon appeasement");

        // Check for buff (e.g., global morale boost or protection aura)
        assert!(app.world().get_resource::<LeviathanProtectionBuff>().is_some(), "Appeasement should grant a protection buff");
    }

    #[test]
    fn test_refusing_leviathan_increases_anger_and_triggers_disaster() {
        // Arrange
        let mut app = App::new();
        app.add_event::<DisasterEvent>();
        app.add_systems(Update, leviathan_refusal_system);

        let leviathan = app.world_mut().spawn(Leviathan {
            tribute_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
            current_demand: Some(TributeDemand { item: ItemType::Food, amount: 500 }),
            anger_level: 90, // Close to threshold
        }).id();

        // Act
        app.world_mut().send_event(RefuseTributeEvent { leviathan_id: leviathan });
        app.update();

        // Assert
        let leviathan_comp = app.world().get::<Leviathan>(leviathan).unwrap();
        assert_eq!(leviathan_comp.anger_level, 100, "Refusing should increase anger");
        assert!(leviathan_comp.current_demand.is_none(), "Demand should be cleared after refusal");

        let disaster_events = app.world().resource::<Events<DisasterEvent>>();
        let mut reader = disaster_events.get_reader();
        assert!(reader.read(disaster_events).next().is_some(), "Reaching max anger should trigger a disaster event");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use std::time::Duration;

#[derive(Component)]
pub struct Leviathan {
    pub tribute_timer: Timer,
    pub current_demand: Option<TributeDemand>,
    pub anger_level: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct TributeDemand {
    pub item: ItemType, // Assuming ItemType exists in the codebase
    pub amount: u32,
}

#[derive(Resource)]
pub struct LeviathanProtectionBuff {
    pub duration: Timer,
}

#[derive(Event)]
pub struct PayTributeEvent {
    pub leviathan_id: Entity,
}

#[derive(Event)]
pub struct RefuseTributeEvent {
    pub leviathan_id: Entity,
}

#[derive(Event)]
pub struct DisasterEvent {
    pub source: Entity,
    pub severity: u32,
}

// Dummy ItemType and Inventory for compilation of example
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemType { Food, Energy }
#[derive(Resource)]
pub struct Inventory { items: std::collections::HashMap<ItemType, u32> }
impl Inventory {
    pub fn new() -> Self { Self { items: std::collections::HashMap::new() } }
    pub fn add(&mut self, item: ItemType, amount: u32) { *self.items.entry(item).or_insert(0) += amount; }
    pub fn get(&self, item: ItemType) -> u32 { *self.items.get(&item).unwrap_or(&0) }
    pub fn remove(&mut self, item: ItemType, amount: u32) -> bool {
        let current = self.get(item);
        if current >= amount {
            self.items.insert(item, current - amount);
            true
        } else {
            false
        }
    }
}

pub fn leviathan_tribute_system(
    time: Res<Time>,
    mut query: Query<&mut Leviathan>,
) {
    for mut leviathan in query.iter_mut() {
        if leviathan.current_demand.is_none() {
            leviathan.tribute_timer.tick(time.delta());
            if leviathan.tribute_timer.just_finished() {
                // Generate random demand (hardcoded for minimal implementation)
                leviathan.current_demand = Some(TributeDemand { item: ItemType::Food, amount: 500 });
            }
        }
    }
}

pub fn leviathan_appeasement_system(
    mut commands: Commands,
    mut events: EventReader<PayTributeEvent>,
    mut query: Query<&mut Leviathan>,
    mut inventory: ResMut<Inventory>,
) {
    for event in events.read() {
        if let Ok(mut leviathan) = query.get_mut(event.leviathan_id) {
            if let Some(demand) = leviathan.current_demand {
                if inventory.remove(demand.item, demand.amount) {
                    leviathan.current_demand = None;
                    leviathan.anger_level = 0;
                    commands.insert_resource(LeviathanProtectionBuff {
                        duration: Timer::from_seconds(60.0, TimerMode::Once),
                    });
                }
            }
        }
    }
}

pub fn leviathan_refusal_system(
    mut events: EventReader<RefuseTributeEvent>,
    mut disaster_writer: EventWriter<DisasterEvent>,
    mut query: Query<&mut Leviathan>,
) {
    for event in events.read() {
        if let Ok(mut leviathan) = query.get_mut(event.leviathan_id) {
            leviathan.current_demand = None;
            leviathan.anger_level += 10;
            if leviathan.anger_level >= 100 {
                disaster_writer.send(DisasterEvent {
                    source: event.leviathan_id,
                    severity: leviathan.anger_level,
                });
                leviathan.anger_level = 0; // Reset after disaster
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Procedural Demands:** Move the hardcoded `TributeDemand` generation into a configurable, weighted loot-table system that scales with colony wealth.
- **Anger Mechanics:** Have anger passively decay if tributes are paid consistently, and passively rise if ignored (timer expiry instead of explicit refusal).
- **Buff Variety:** Instead of a generic `LeviathanProtectionBuff`, link the buff type to the type of resource sacrificed (e.g., sacrificing Energy grants a shield boost, Food grants crop growth).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Leviathan correctly demands resources on a timer, accepts payments (deducting inventory), and punishes refusals via Disaster events.

## 7. Technical Guidance
- **Integration:** Hook `PayTributeEvent` and `RefuseTributeEvent` into the UI layer so the player gets a pop-up when a demand is active.
- **Gotchas:** Ensure the `tribute_timer` pauses while a demand is active, otherwise the Leviathan might generate overlapping demands or instantly generate a new one after payment. (The minimal implementation handles this by checking `is_none()`).

## 8. Questions
*Builder: Add questions here if spec is unclear.*
