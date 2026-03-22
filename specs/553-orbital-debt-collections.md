# Spec 553: Orbital Debt Collections

## 1. Overview
If a colony defaults on a massive loan from a Layer 3 megacorporation, they don't declare war. Instead, they deploy "Repo Drones" in Layer 2 orbit. These drones use precision tractor beams to selectively lift high-value manufactured goods, or even entire small modular buildings, straight off the Layer 1 surface, bypassing ground defenses until the debt is paid.

## 2. Dependencies
- `039` Trade System (Contracts/Loans)
- `094` System View Architecture (Layer 2 -> Layer 1 interaction)
- Entity destruction/removal logic in Layer 1.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::buildings::Building;
    use crate::layer1::resources::ResourceStash;

    #[test]
    fn test_defaulting_on_loan_triggers_repo_drones() {
        let mut app = App::new();
        app.add_systems(Update, process_loan_default_system);

        let colony_finances = app.world_mut().insert_resource(ColonyFinances {
            debt: 100_000.0,
            defaulted: true,
        });

        app.update();

        let repo_events = app.world().resource::<Events<RepoDroneArrivalEvent>>();
        let mut reader = repo_events.get_reader();
        assert!(reader.read(repo_events).len() > 0, "Loan default should spawn repo drones in orbit");
    }

    #[test]
    fn test_repo_drones_remove_high_value_buildings_from_layer_1() {
        let mut app = App::new();
        app.add_systems(Update, execute_repo_drone_extraction_system);

        let hospital = app.world_mut().spawn((
            Building { value: 50_000.0 }, // High value
        )).id();

        app.world_mut().spawn(RepoDrone { target: hospital });

        app.update();

        assert!(app.world().get::<Building>(hospital).is_none(), "Repo drone should extract (despawn) the high-value building");
    }

    #[test]
    fn test_repo_drones_reduce_colony_debt_upon_extraction() {
        let mut app = App::new();
        app.add_systems(Update, execute_repo_drone_extraction_system);

        app.world_mut().insert_resource(ColonyFinances { debt: 100_000.0, defaulted: true });
        let generator = app.world_mut().spawn((Building { value: 40_000.0 })).id();
        app.world_mut().spawn(RepoDrone { target: generator });

        app.update();

        let finances = app.world().resource::<ColonyFinances>();
        assert_eq!(finances.debt, 60_000.0, "Extracting a building should reduce the debt by its value");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct ColonyFinances {
    pub debt: f32,
    pub defaulted: bool,
}

#[derive(Component)]
pub struct Building {
    pub value: f32,
}

#[derive(Event)]
pub struct RepoDroneArrivalEvent;

#[derive(Component)]
pub struct RepoDrone {
    pub target: Entity,
}

pub fn process_loan_default_system(
    finances: Res<ColonyFinances>,
    mut events: EventWriter<RepoDroneArrivalEvent>,
) {
    if finances.defaulted && finances.debt > 0.0 {
        events.send(RepoDroneArrivalEvent);
    }
}

pub fn execute_repo_drone_extraction_system(
    mut commands: Commands,
    mut finances: ResMut<ColonyFinances>,
    repo_query: Query<(Entity, &RepoDrone)>,
    building_query: Query<&Building>,
) {
    for (drone_entity, drone) in repo_query.iter() {
        if let Ok(building) = building_query.get(drone.target) {
            finances.debt -= building.value; // Pay off debt with the building's value
            if finances.debt <= 0.0 {
                finances.defaulted = false;
                finances.debt = 0.0;
            }
            commands.entity(drone.target).despawn_recursive(); // Extract building
            commands.entity(drone_entity).despawn(); // Drone leaves
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an algorithm for the `RepoDrone` to target the most valuable assets first (highest `value` ratio to the remaining debt), prioritizing finished goods in stockpiles before resorting to tearing down critical infrastructure like Hospitals or Power Plants.
- Add visual indicators/events to Layer 1 (a tractor beam effect or warning notification) so the player knows what is about to be stolen.
- The `RepoDrone` should be an entity in Layer 2 orbit that must take time to extract the target, giving the player a brief window to perhaps frantically pay off the debt or deploy a desperate anti-orbital defense (if they have one).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Defaulting on a loan spawns a `RepoDroneArrivalEvent`.
- [ ] `RepoDrone` entities target and despawn `Building` (or high-value `Resource`) entities on Layer 1.
- [ ] The value of the extracted asset is subtracted from the `ColonyFinances` debt.

## 7. Technical Guidance
- The `RepoDroneArrivalEvent` will serve as the bridge between the L3 Megacorp's decision to collect and the L2/L1 execution.
- If a building is extracted, ensure all Pops inside are safely "dropped" (or perhaps they are taken as collateral too, creating a hostage situation). For MVP, just despawn the building and un-assign the Pops working there.

## 8. Questions
*Builder: add questions here if spec is unclear.*
