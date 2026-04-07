# Specification: 864 Xeno-Zoo

## 1. Overview
**Layer:** 1
**Fantasy:** Displaying the terrifying beasts of the frontier for a profit.
**Mechanic:** Capturing live fauna allows you to construct a "Xeno-Zoo". This building generates "Leisure" and "Morale" based on the danger level of the creatures contained. However, a containment breach releases angry, confined predators into the heart of your colony.

## 2. Dependencies
- Layer 1 Buildings (`src/layer1/building.rs`)
- Layer 1 Fauna (`src/layer1/fauna.rs`)
- Needs System (Leisure)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_xeno_zoo_generates_leisure_value() {
        let mut app = App::new();
        app.add_plugins(XenoZooPlugin);

        // Spawn a zoo with a dangerous creature
        let beast = app.world_mut().spawn(DangerousFauna { danger_level: 5 }).id();
        let zoo = app.world_mut().spawn((
            Building,
            XenoZoo { contained_beasts: vec![beast], base_leisure: 2 },
            LeisureProvider { value: 0 },
        )).id();

        app.update();

        // System should update the leisure value based on the beast
        let provider = app.world().get::<LeisureProvider>(zoo).unwrap();
        assert_eq!(provider.value, 7, "Zoo leisure value should equal base + danger_level");
    }

    #[test]
    fn test_containment_breach_releases_beasts() {
        let mut app = App::new();
        app.add_plugins(XenoZooPlugin);

        let beast = app.world_mut().spawn(DangerousFauna { danger_level: 5 }).id();
        let zoo = app.world_mut().spawn((
            Building,
            XenoZoo { contained_beasts: vec![beast], base_leisure: 2 },
            Position(UVec2::new(10, 10)),
        )).id();

        app.world_mut().send_event(ContainmentBreachEvent { zoo_entity: zoo });
        app.update();

        let zoo_comp = app.world().get::<XenoZoo>(zoo).unwrap();
        assert!(zoo_comp.contained_beasts.is_empty(), "Zoo should be empty after breach");

        // Ensure beast is now active on the map
        let beast_pos = app.world().get::<Position>(beast).expect("Beast should have a position");
        assert_eq!(beast_pos.0, UVec2::new(10, 10), "Beast should spawn at zoo location");
        assert!(app.world().get::<ActivePredator>(beast).is_some(), "Beast should become an active predator");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Position(pub UVec2);

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct DangerousFauna {
    pub danger_level: i32,
}

#[derive(Component)]
pub struct ActivePredator;

#[derive(Component)]
pub struct XenoZoo {
    pub contained_beasts: Vec<Entity>,
    pub base_leisure: i32,
}

#[derive(Component)]
pub struct LeisureProvider {
    pub value: i32,
}

#[derive(Event)]
pub struct ContainmentBreachEvent {
    pub zoo_entity: Entity,
}

pub struct XenoZooPlugin;

impl Plugin for XenoZooPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ContainmentBreachEvent>()
           .add_systems(Update, (
               calculate_zoo_leisure_system,
               handle_containment_breach_system,
           ));
    }
}

fn calculate_zoo_leisure_system(
    mut zoos: Query<(&XenoZoo, &mut LeisureProvider)>,
    beasts: Query<&DangerousFauna>,
) {
    for (zoo, mut provider) in zoos.iter_mut() {
        let mut total_danger = 0;
        for &beast_entity in &zoo.contained_beasts {
            if let Ok(beast) = beasts.get(beast_entity) {
                total_danger += beast.danger_level;
            }
        }
        provider.value = zoo.base_leisure + total_danger;
    }
}

fn handle_containment_breach_system(
    mut events: EventReader<ContainmentBreachEvent>,
    mut zoos: Query<(&mut XenoZoo, &Position)>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok((mut zoo, pos)) = zoos.get_mut(event.zoo_entity) {
            let escaped_beasts = std::mem::take(&mut zoo.contained_beasts);

            for beast_entity in escaped_beasts {
                commands.entity(beast_entity)
                    .insert(Position(pos.0))
                    .insert(ActivePredator);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- `calculate_zoo_leisure_system` currently recalculates every frame. This should only recalculate when a beast is added or removed, or when a breach occurs. Add a dirty flag or listen to an event.
- Ensure the AI utility systems prioritize fleeing from `ActivePredator` entities.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the module.
- [ ] A zoo's leisure value scales with the `danger_level` of its contained beasts.
- [ ] A `ContainmentBreachEvent` empties the zoo and places the beasts onto the map as `ActivePredator`s.

## 7. Technical Guidance
- The actual trigger for `ContainmentBreachEvent` could be tied to power outages, unrest/riots (sabotage), or simply poor maintenance.
- Be careful with `ActivePredator` AI state initialization to ensure they immediately begin pathfinding or attacking when released.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
