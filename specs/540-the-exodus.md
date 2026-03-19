# 540 - The Exodus

## 1. Overview
An endgame project to build an "Ark Ship". It requires massive resources, forcing you to cannibalize your own advanced infrastructure to build the engines. The colony must physically shrink and dismantle itself to leave.

## 2. Dependencies
- 004 Basic Building
- 022 Resource Stockpiles
- 157 Ship Classes & Construction

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_ark_ship_construction_progress() {
        // Arrange
        let mut world = World::new();
        let ark = world.spawn(ArkShipProject { progress: 0, target: 1000 }).id();
        let colony_stockpile = world.spawn(ColonyResources(vec![("HighTechParts".to_string(), 100)])).id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(build_ark_system);
        schedule.run(&mut world);

        // Assert
        let progress = world.get::<ArkShipProject>(ark).unwrap().progress;
        assert_eq!(progress, 100, "Ark should have consumed resources and progressed");
        let remaining_resources = world.get::<ColonyResources>(colony_stockpile).unwrap().get("HighTechParts");
        assert_eq!(remaining_resources, 0, "Resources should be consumed");
    }

    #[test]
    fn test_cannibalize_building() {
        // Arrange
        let mut world = World::new();
        let building = world.spawn((Building, Cannibalizable { yield_amount: 50 })).id();
        let colony = world.spawn(ColonyResources(vec![("HighTechParts".to_string(), 0)])).id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(cannibalize_infrastructure_system);
        schedule.run(&mut world);

        // Assert
        assert!(world.get::<Building>(building).is_none(), "Building should be dismantled");
        let remaining_resources = world.get::<ColonyResources>(colony).unwrap().get("HighTechParts");
        assert_eq!(remaining_resources, 50, "Yielded resources should go to stockpile");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct ArkShipProject {
    pub progress: u32,
    pub target: u32,
}

#[derive(Component)]
pub struct ColonyResources(pub Vec<(String, u32)>);

impl ColonyResources {
    pub fn get(&self, item: &str) -> u32 {
        self.0.iter().find(|r| r.0 == item).map(|r| r.1).unwrap_or(0)
    }

    pub fn remove(&mut self, item: &str, amount: u32) -> bool {
        if let Some(res) = self.0.iter_mut().find(|r| r.0 == item) {
            if res.1 >= amount {
                res.1 -= amount;
                return true;
            } else {
                let taken = res.1;
                res.1 = 0;
                return taken > 0;
            }
        }
        false
    }
}

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct Cannibalizable {
    pub yield_amount: u32,
}

pub fn build_ark_system(
    mut ark_projects: Query<&mut ArkShipProject>,
    mut stockpiles: Query<&mut ColonyResources>,
) {
    for mut ark in ark_projects.iter_mut() {
        if let Ok(mut stockpile) = stockpiles.get_single_mut() {
            // Take up to 100 parts per tick
            if stockpile.remove("HighTechParts", 100) {
                ark.progress += 100;
            }
        }
    }
}

pub fn cannibalize_infrastructure_system(
    mut commands: Commands,
    buildings: Query<(Entity, &Cannibalizable), With<Building>>,
    mut stockpiles: Query<&mut ColonyResources>,
) {
    for (entity, can) in buildings.iter() {
        if let Ok(mut stockpile) = stockpiles.get_single_mut() {
            if let Some(res) = stockpile.0.iter_mut().find(|r| r.0 == "HighTechParts") {
                res.1 += can.yield_amount;
            } else {
                stockpile.0.push(("HighTechParts".to_string(), can.yield_amount));
            }
            commands.entity(entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate with `BuildingDemolish` system but add special "Cannibalize" action that yields more specific endgame parts instead of standard scrap.
- Create multi-stage ark construction.
- Make sure pops migrate to the ship.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Buildings can be cannibalized for Ark materials.
- [ ] Ark consumes materials and makes progress toward launch.

## 7. Technical Guidance
- Create a special build mode or toggle for "Cannibalize" versus standard demolish.
- Only very advanced buildings (Reactors, Sci Labs) should be `Cannibalizable` for Ark parts.

## 8. Questions
*Builder: add questions here if spec is unclear.*
