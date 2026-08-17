# 1373: The 'Recalled' Product

## 1. Overview
**Layer:** 3 -> 1

**Fantasy:** Corporate negligence on a galactic scale.

**Mechanic:** A product you rely on (e.g., "Standard Rations", "Fusion Cells") is issued a "Recall Notice" by the manufacturer due to defects. Using it has a 5% chance of critical failure/poisoning. Returning it grants Credits but leaves you without stock.

**Emergence:** You get the recall notice for your ammo during a siege. You have to choose: shoot the defective rounds (gun might explode) or fight with knives.

**Tension:** Safety (Recall) vs. Necessity (Use it anyway).

## 2. Dependencies
- Base ECS system
- Resource/Inventory system
- Item usage/consumption events
- Event generation/notification system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<UseItemEvent>();
        app.add_event::<ReturnRecalledItemEvent>();
        app.add_systems(Update, (
            process_item_usage_system,
            process_item_return_system,
        ));
        app.insert_resource(Economy { credits: 0.0 });
        app
    }

    #[test]
    fn test_using_recalled_item_has_failure_chance() {
        let mut app = setup_app();

        // Spawn a pop using the item
        let pop = app.world_mut().spawn((
            Pop,
            Health { current: 100.0, max: 100.0 }
        )).id();

        // Mock randomness or seed it to ensure deterministic testing in a real scenario
        // For simplicity, we trigger enough events to statically test the failure threshold.
        // We'll use 500 usages to make failure statistically certain for this basic test.
        for _ in 0..500 {
            app.world_mut().send_event(UseItemEvent {
                user: pop,
                item_type: ItemType::FusionCell,
                is_recalled: true,
            });
        }

        app.update();

        let health = app.world().get::<Health>(pop).unwrap();

        // Health should be reduced due to at least one critical failure
        assert!(health.current < 100.0);
    }

    #[test]
    fn test_returning_recalled_item_grants_credits() {
        let mut app = setup_app();

        app.world_mut().send_event(ReturnRecalledItemEvent {
            item_type: ItemType::FusionCell,
            quantity: 10,
        });

        app.update();

        let economy = app.world().resource::<Economy>();

        // Credits should increase (e.g. 5 credits per item)
        assert_eq!(economy.credits, 50.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use rand::Rng;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    StandardRation,
    FusionCell,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Resource)]
pub struct Economy {
    pub credits: f32,
}

#[derive(Event)]
pub struct UseItemEvent {
    pub user: Entity,
    pub item_type: ItemType,
    pub is_recalled: bool,
}

#[derive(Event)]
pub struct ReturnRecalledItemEvent {
    pub item_type: ItemType,
    pub quantity: u32,
}

pub fn process_item_usage_system(
    mut events: EventReader<UseItemEvent>,
    mut health_query: Query<&mut Health>,
) {
    let mut rng = rand::thread_rng();

    for ev in events.read() {
        if ev.is_recalled {
            // 5% chance of critical failure
            if rng.gen_bool(0.05) {
                if let Ok(mut health) = health_query.get_mut(ev.user) {
                    health.current -= 25.0; // Explosion/poison damage
                }
            }
        }
    }
}

pub fn process_item_return_system(
    mut events: EventReader<ReturnRecalledItemEvent>,
    mut economy: ResMut<Economy>,
) {
    for ev in events.read() {
        // Grant 5 credits per returned item
        economy.credits += (ev.quantity * 5) as f32;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Implement a global `RecallManager` resource that tracks which `ItemType`s are currently recalled.
- Adjust item consumption logic to check the `RecallManager` rather than relying on the event to pass `is_recalled`.
- Ensure the failure effects differ by item type (e.g. rations poison health, ammo damages weapons or explodes).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Using an item marked as recalled applies a random failure consequence.
- [ ] Processing a `ReturnRecalledItemEvent` correctly increments `Economy.credits`.

## 7. Technical Guidance
- The random failure should hook into existing event streams (like `DamageEvent` or `SicknessEvent`) rather than directly mutating health if possible.
- Ensure the UI prompts the player when a recall happens, giving them a distinct action button to "Return All Stock".

## 8. Questions
*Builder: add questions here if spec is unclear.*
