# 351: The Long-Haulers

## 1. Overview

Your colony is just a stopover for some. Not everyone calls this rock "home".

**The Long-Haulers** mechanic introduces temporary colonists. Some pops arrive with the `Traveler` trait. They function as normal workers, gaining XP and contributing to the colony, but they have a `DepartureDate`. When an outbound ship arrives (or launches) near that date, they leave the colony permanently, taking their accumulated skills with them.

This mechanic challenges the player to decide whether to invest heavily in a high-skilled worker who is guaranteed to leave, or focus on permanent but less skilled residents.

## 2. Dependencies

- `004` Pop Entity (for traits and pop management)
- `009` Job System (for assigning jobs to Travelers)
- `099` Fleet Movement / Spaceport Mechanics (to trigger the departure event)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{PopBundle, Traits, Trait};
    use crate::layer1::time::SimulationTime;
    use crate::layer2::trade::{ShipArrivalEvent, ShipType};

    #[test]
    fn test_traveler_departs_when_ship_arrives() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(process_traveler_departures_system);

        // Setup time
        world.insert_resource(SimulationTime { tick: 1000, ..Default::default() });

        let mut events = Events::<ShipArrivalEvent>::default();
        events.send(ShipArrivalEvent { ship_type: ShipType::Transport });
        world.insert_resource(events);

        let pop_entity = world.spawn((
            PopBundle::default(),
            Traveler { departure_tick: 950 }, // Departure date has passed
        )).id();

        schedule.run(&mut world);

        // Pop should be despawned or marked as departed
        assert!(world.get_entity(pop_entity).is_err() || world.get_entity(pop_entity).unwrap().is_despawned());
    }

    #[test]
    fn test_traveler_stays_if_departure_date_not_reached() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(process_traveler_departures_system);

        world.insert_resource(SimulationTime { tick: 1000, ..Default::default() });

        let mut events = Events::<ShipArrivalEvent>::default();
        events.send(ShipArrivalEvent { ship_type: ShipType::Transport });
        world.insert_resource(events);

        let pop_entity = world.spawn((
            PopBundle::default(),
            Traveler { departure_tick: 1500 }, // Departure date is in the future
        )).id();

        schedule.run(&mut world);

        // Pop should still exist
        assert!(world.get_entity(pop_entity).is_ok());
    }

    #[test]
    fn test_traveler_stays_if_no_ship_arrives() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(process_traveler_departures_system);

        world.insert_resource(SimulationTime { tick: 1000, ..Default::default() });

        // No ships arriving
        world.insert_resource(Events::<ShipArrivalEvent>::default());

        let pop_entity = world.spawn((
            PopBundle::default(),
            Traveler { departure_tick: 950 }, // Past departure date
        )).id();

        schedule.run(&mut world);

        // Pop should still exist because no ship arrived to take them
        assert!(world.get_entity(pop_entity).is_ok());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::time::SimulationTime;
use crate::layer2::trade::{ShipArrivalEvent, ShipType};
use crate::layer1::pop::Pop;

#[derive(Component)]
pub struct Traveler {
    pub departure_tick: u64,
}

pub fn process_traveler_departures_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    mut ship_events: EventReader<ShipArrivalEvent>,
    traveler_query: Query<(Entity, &Traveler), With<Pop>>,
) {
    // Only process if a transport ship actually arrived
    let ship_arrived = ship_events.read().any(|e| matches!(e.ship_type, ShipType::Transport | ShipType::Passenger));

    if !ship_arrived {
        return;
    }

    for (entity, traveler) in traveler_query.iter() {
        if time.tick >= traveler.departure_tick {
            commands.entity(entity).despawn_recursive();
            // In a real implementation, you might spawn a "Departed" chronicle event here
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Chronicle Integration:** Emitting an `AddChronicleEvent` when a highly skilled Traveler leaves would add narrative weight to their departure.
- **Mood Impact:** Other Pops who formed relationships with the departing Traveler should suffer a temporary `Sadness` or `Misses Friend` mood penalty.
- **UI Warning:** Add a notification system hook to warn the player when a Traveler is nearing their departure date, allowing them to extract tools or reassign critical jobs before they suddenly vanish.
- **Detention Mechanics:** Consider allowing the player to "Detain" a Traveler. This would prevent them from leaving but cause a massive mood penalty, potential factional unrest, and a hit to diplomatic standing with the transport ship's faction.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code
- [ ] Travelers properly despawn (or are transferred off-map) when their `departure_tick` has passed AND a relevant ship arrives.
- [ ] Travelers do not leave before their departure tick or if no ship is present.

## 7. Technical Guidance

- Place `Traveler` in a new module, e.g., `src/layer1/pop/traveler.rs` or alongside traits.
- The `ShipArrivalEvent` check should ideally differentiate between ships that can carry passengers and those that cannot (like automated cargo drones).
- Make sure that destroying a Pop entity handles cleanup correctly, such as un-assigning them from their current `Job` or `Bed` to prevent ghost references.

## 8. Questions

*Builder: add questions here if spec is unclear.*
