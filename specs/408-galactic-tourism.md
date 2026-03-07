# 408: Galactic Tourism

## 1. Overview
If you build a destination, they will come. If a colony achieves a high "Beauty" or "Wonder" score (via art, high-tier rooms, or monuments), it begins attracting Visitor Ships from Layer 3.

Tourists pay massive amounts of Credits upon landing. However, they consume the colony's Food and Services and occupy physical space. If a food shortage occurs while a luxury liner is docked, the player faces a terrible choice: deny the tourists food (tanking diplomatic reputation and ending the tourism boom) or starve their own workers to feed the rich.

## 2. Dependencies
- `074-visitor-system.md` (Base mechanics for non-colony Pops arriving)
- `044-horticulture-beauty.md` (To calculate the colony's Beauty score)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::visitor::{Visitor, VisitorSpawning};
    use crate::layer1::colony::ColonyStats;

    #[test]
    fn test_high_beauty_triggers_tourist_spawn() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_tourist_attraction_system);

        // High beauty colony
        app.insert_resource(ColonyStats { global_beauty: 500.0, credits: 0 });
        app.insert_resource(VisitorSpawning { pending_tourists: 0 });

        app.update();

        let spawning = app.world().get_resource::<VisitorSpawning>().unwrap();
        assert!(spawning.pending_tourists > 0, "High beauty should queue tourists to spawn");
    }

    #[test]
    fn test_tourist_arrival_grants_credits() {
        let mut app = App::new();
        app.add_event::<TouristArrivalEvent>();
        app.add_systems(Update, process_tourist_arrival_system);

        app.insert_resource(ColonyStats { global_beauty: 500.0, credits: 100 });

        let tourist_ent = app.world_mut().spawn(Tourist { wealth: 500 }).id();

        app.world_mut().send_event(TouristArrivalEvent { tourist: tourist_ent });

        app.update();

        let stats = app.world().get_resource::<ColonyStats>().unwrap();
        assert_eq!(stats.credits, 600, "Colony should receive credits upon tourist arrival");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::colony::ColonyStats;

#[derive(Component)]
pub struct Tourist {
    pub wealth: u32,
}

#[derive(Resource, Default)]
pub struct VisitorSpawning {
    pub pending_tourists: u32,
}

#[derive(Event)]
pub struct TouristArrivalEvent {
    pub tourist: Entity,
}

pub fn evaluate_tourist_attraction_system(
    stats: Res<ColonyStats>,
    mut spawning: ResMut<VisitorSpawning>,
) {
    if stats.global_beauty > 300.0 {
        // Mock random chance or accumulation
        spawning.pending_tourists += 1;
    }
}

pub fn process_tourist_arrival_system(
    mut events: EventReader<TouristArrivalEvent>,
    tourist_query: Query<&Tourist>,
    mut stats: ResMut<ColonyStats>,
) {
    for event in events.read() {
        if let Ok(tourist) = tourist_query.get(event.tourist) {
            stats.credits += tourist.wealth;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Consumption:** Tourists need to actually be spawned as `Pop` entities with an override `UtilityAI` that heavily prioritizes `Leisure` and eating the highest quality `Food` available.
- **Anger:** If a Tourist's needs drop too low (no food, no beautiful places to sit), they should depart early and trigger a `ReputationPenalty` for the colony.
- **Quarantine Check:** Tourists bypass the standard immigrant screening process because of their wealth, introducing a risk of bringing alien diseases (integrates with Quarantine protocols).

## 6. Acceptance Criteria (Testable!)
- [ ] `ColonyStats` global beauty score influences tourist attraction rates.
- [ ] Tourists grant Credits upon arriving at the colony.
- [ ] Tests pass.

## 7. Technical Guidance
- Make sure `Tourist` entities use the `Visitor` tag from `074-visitor-system.md` so they aren't assigned standard colony jobs like mining.

## 8. Questions
*Builder: add questions here if spec is unclear.*
