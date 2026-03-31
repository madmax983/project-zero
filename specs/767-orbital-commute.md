# Orbital Commute

## 1. Overview
**Layer:** Cross-layer

**Fantasy:** The daily grind, in space. Living in the suburbs (orbit) and working in the city (surface).

**Mechanic:** Pops can have homes on one Layer 1 map (Planet) and jobs on another (Orbital Station) connected by "Shuttle Routes". Commute time consumes "Free Time" and requires fuel.

**Emergence:** A fuel shortage grounds the shuttles. Your orbital refinery shuts down because the workers are stuck on the ground playing cards.

**Tension:** Centralized housing (Efficiency/Happiness) vs. Distributed specialized work (Logistics cost).

## 2. Dependencies
- Needs and Actions systems (`src/layer1/needs.rs`, `src/layer1/actions.rs`)
- Fleet/Shuttle movement across boundaries
- Colony resources (Fuel)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_orbital_commute_consumes_fuel_and_updates_location() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ColonyResources>();
        app.resource_mut::<ColonyResources>().fuel = 10;
        app.add_systems(Update, process_orbital_commutes);

        let planet_entity = app.world_mut().spawn(SystemNode).id();
        let station_entity = app.world_mut().spawn(SystemNode).id();

        let pop = app.world_mut().spawn((
            Pop,
            CurrentLocation(planet_entity),
            CommuteAction {
                destination: station_entity,
                fuel_cost: 2,
                duration: 3,
                progress: 0,
            }
        )).id();

        // Tick 1
        app.update();
        let commute = app.world().get::<CommuteAction>(pop).unwrap();
        assert_eq!(commute.progress, 1);
        assert_eq!(app.world().resource::<ColonyResources>().fuel, 8);
        assert_eq!(app.world().get::<CurrentLocation>(pop).unwrap().0, planet_entity); // Still en route

        // Tick 2
        app.update();

        // Tick 3 - Commute complete
        app.update();

        let loc = app.world().get::<CurrentLocation>(pop).unwrap();
        assert_eq!(loc.0, station_entity);
        assert!(app.world().get::<CommuteAction>(pop).is_none()); // Action removed
    }

    #[test]
    fn test_commute_stalls_without_fuel() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ColonyResources>();
        app.resource_mut::<ColonyResources>().fuel = 0; // No fuel
        app.add_systems(Update, process_orbital_commutes);

        let planet_entity = app.world_mut().spawn(SystemNode).id();
        let station_entity = app.world_mut().spawn(SystemNode).id();

        let pop = app.world_mut().spawn((
            Pop,
            CurrentLocation(planet_entity),
            CommuteAction {
                destination: station_entity,
                fuel_cost: 2,
                duration: 3,
                progress: 0,
            }
        )).id();

        app.update();

        let commute = app.world().get::<CommuteAction>(pop).unwrap();
        assert_eq!(commute.progress, 0); // No progress
        assert_eq!(app.world().get::<CurrentLocation>(pop).unwrap().0, planet_entity);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct SystemNode;

#[derive(Component)]
pub struct CurrentLocation(pub Entity);

#[derive(Component)]
pub struct CommuteAction {
    pub destination: Entity,
    pub fuel_cost: u32,
    pub duration: u32,
    pub progress: u32,
}

#[derive(Resource, Default)]
pub struct ColonyResources {
    pub fuel: u32,
}

pub fn process_orbital_commutes(
    mut commands: Commands,
    mut resources: ResMut<ColonyResources>,
    mut query: Query<(Entity, &mut CommuteAction, &mut CurrentLocation)>,
) {
    for (entity, mut commute, mut loc) in query.iter_mut() {
        if resources.fuel >= commute.fuel_cost {
            resources.fuel -= commute.fuel_cost;
            commute.progress += 1;

            if commute.progress >= commute.duration {
                loc.0 = commute.destination;
                commands.entity(entity).remove::<CommuteAction>();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: `CommuteAction` takes fuel *per tick* in this minimal implementation, which might lead to weird states where a pop runs out of fuel halfway to the station. A better design might be an upfront fuel cost when the commute *starts*, or relying on dedicated "Shuttle" entities that batch pops and handle fuel logic collectively.
- **Performance Considerations**: Querying all commuting pops is fine, but tracking per-pop cross-layer location requires careful sync. Ensure rendering/logic maps correctly update when a `CurrentLocation` changes between Layer 1 map and Layer 2 station.
- **API Improvements**: `CurrentLocation` should likely be an Enum (e.g. `Location::Planet(Entity)`, `Location::Transit(FleetEntity)`, `Location::Orbital(Entity)`) to better represent that they are un-targetable while in transit.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops successfully transition between location entities after consuming the required fuel and time.
- [ ] Commutes correctly stall if insufficient fuel is available.

## 7. Technical Guidance
- Add to a new module `src/layer1/logistics/commute.rs`.
- The Utility AI will need to be updated to score "Commute" actions if a Pop's assigned job and home are on different nodes.
- Make sure to consider edge cases where the destination node is destroyed mid-commute.

## 8. Questions
*Builder: add questions here if spec is unclear.*
