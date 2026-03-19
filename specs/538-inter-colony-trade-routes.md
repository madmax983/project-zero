# 538 - Inter-Colony Trade Routes

## 1. Overview
Space trucking. The lifeblood of an empire. Automated ships travel between your colonies on Layer 2, moving resources (e.g., Colony A makes Food, Colony B makes Metal). Routes take time and can be intercepted by pirates/storms.

## 2. Dependencies
- 039 Trade System
- 099 Fleet Movement
- 094 System View Architecture

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_trade_route_creation() {
        // Arrange
        let mut world = World::new();
        let colony_a = world.spawn(Colony { name: "Earth".to_string() }).id();
        let colony_b = world.spawn(Colony { name: "Mars".to_string() }).id();

        // Act
        let route = TradeRoute {
            source: colony_a,
            destination: colony_b,
            item_type: "Food".to_string(),
            amount: 100,
            interval: 10,
        };
        let route_entity = world.spawn(route.clone()).id();

        // Assert
        let stored_route = world.get::<TradeRoute>(route_entity).unwrap();
        assert_eq!(stored_route.source, colony_a);
        assert_eq!(stored_route.item_type, "Food");
    }

    #[test]
    fn test_trade_route_execution() {
        // Arrange
        let mut world = World::new();
        let colony_a = world.spawn(Colony { name: "Earth".to_string(), resources: vec![("Food".to_string(), 500)] }).id();
        let colony_b = world.spawn(Colony { name: "Mars".to_string(), resources: vec![("Food".to_string(), 0)] }).id();

        let route = TradeRoute {
            source: colony_a,
            destination: colony_b,
            item_type: "Food".to_string(),
            amount: 100,
            interval: 1,
        };
        world.spawn((route, Timer(1)));

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(execute_trade_routes_system);
        schedule.run(&mut world);

        // Assert
        let a_res = world.get::<Colony>(colony_a).unwrap().get_resource("Food");
        let b_res = world.get::<Colony>(colony_b).unwrap().get_resource("Food");
        assert_eq!(a_res, 400, "Source colony should have sent 100 Food");
        assert_eq!(b_res, 100, "Destination colony should have received 100 Food");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Colony {
    pub name: String,
    pub resources: Vec<(String, u32)>,
}

impl Colony {
    pub fn get_resource(&self, item: &str) -> u32 {
        self.resources.iter().find(|r| r.0 == item).map(|r| r.1).unwrap_or(0)
    }

    pub fn add_resource(&mut self, item: String, amount: u32) {
        if let Some(res) = self.resources.iter_mut().find(|r| r.0 == item) {
            res.1 += amount;
        } else {
            self.resources.push((item, amount));
        }
    }

    pub fn remove_resource(&mut self, item: &str, amount: u32) -> bool {
        if let Some(res) = self.resources.iter_mut().find(|r| r.0 == item) {
            if res.1 >= amount {
                res.1 -= amount;
                return true;
            }
        }
        false
    }
}

#[derive(Component, Clone)]
pub struct TradeRoute {
    pub source: Entity,
    pub destination: Entity,
    pub item_type: String,
    pub amount: u32,
    pub interval: u32,
}

#[derive(Component)]
pub struct Timer(pub u32);

pub fn execute_trade_routes_system(
    mut routes: Query<(&TradeRoute, &mut Timer)>,
    mut colonies: Query<&mut Colony>,
) {
    for (route, mut timer) in routes.iter_mut() {
        if timer.0 > 0 {
            timer.0 -= 1;
        }
        if timer.0 == 0 {
            // Very naive execution assuming source has items
            if let Ok(mut source_colony) = colonies.get_mut(route.source) {
                if source_colony.remove_resource(&route.item_type, route.amount) {
                    if let Ok(mut dest_colony) = colonies.get_mut(route.destination) {
                        dest_colony.add_resource(route.item_type.clone(), route.amount);
                    }
                }
            }
            timer.0 = route.interval;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create actual ship entities that move between colonies instead of instantaneous teleportation.
- Add piracy/hazard interception mechanics to the ship travel.
- Use `Layer2` fleet movement systems.
- Proper inventory management instead of strings.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Items move between colonies reliably.

## 7. Technical Guidance
- `TradeRoute` component should spawn an active `TradeShip` fleet when interval hits.
- The `TradeShip` moves along the path using `FleetMovement` systems.

## 8. Questions
*Builder: add questions here if spec is unclear.*
