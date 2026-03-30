# 762 - The Gold Rush Beacon

## 1. Overview
**Layer:** Cross-layer (2 -> 1)
**Fantasy:** Hanging out a sign that says "Open for Business" in a lawless frontier.
**Mechanic:** You can activate a "Colony Beacon" to attract Migrants and Traders rapidly. It boosts growth massively but increases the spawn rate of Pirates, Criminals, and Grifters (low-skill pops).
**Emergence:** Desperate for workers, you light the beacon. You get the workers, but your colony transforms from a disciplined outpost into a chaotic, crime-ridden boomtown.
**Tension:** Slow, safe growth vs. Fast, dangerous expansion.

## 2. Dependencies
- Layer 1 Economy and Trading (`TradeShipArrivalEvent`, `MigrantArrivalEvent`)
- Layer 1 Justice/Crime (`CrimeRecord` or similar for criminal pops)
- Layer 2 Fleet generation (for spawning pirates/traders)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::economy::trade::TradeShipArrivalEvent;
    use crate::layer1::population::MigrantArrivalEvent;
    use crate::layer1::justice::CrimeRecord;
    use crate::layer1::pop::{Pop, PopTraits, Trait};
    use crate::shared::time::SimulationTime;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(SimulationTime::default());
        app.insert_resource(ColonyBeacon::default());
        app.add_event::<TradeShipArrivalEvent>();
        app.add_event::<MigrantArrivalEvent>();
        app.add_event::<PirateRaidEvent>();
        app.add_systems(Update, process_colony_beacon_system);
        app
    }

    #[test]
    fn test_beacon_increases_arrival_rates() {
        let mut app = setup_app();

        // Activate the beacon
        app.world_mut().resource_mut::<ColonyBeacon>().is_active = true;

        // Run the simulation for several ticks
        for _ in 0..100 {
            app.update();
        }

        // Check if the number of arrivals is significantly higher than baseline
        let trade_events = app.world().get_resource::<Events<TradeShipArrivalEvent>>().unwrap();
        let migrant_events = app.world().get_resource::<Events<MigrantArrivalEvent>>().unwrap();

        assert!(!trade_events.is_empty(), "Active beacon should trigger trade ships");
        assert!(!migrant_events.is_empty(), "Active beacon should trigger migrants");
    }

    #[test]
    fn test_beacon_spawns_criminals_and_pirates() {
        let mut app = setup_app();

        app.world_mut().resource_mut::<ColonyBeacon>().is_active = true;

        for _ in 0..100 {
            app.update();
        }

        // Ensure pirate raids are triggered
        let pirate_events = app.world().get_resource::<Events<PirateRaidEvent>>().unwrap();
        assert!(!pirate_events.is_empty(), "Active beacon should attract pirates");

        // The system that spawns migrants based on MigrantArrivalEvent should have a higher chance
        // to spawn pops with criminal or low-skill traits when beacon is active.
        // This test assumes `process_colony_beacon_system` modifies a global modifier or emits
        // specific events that downstream systems use.
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy_ecs::prelude::*;
use rand::Rng;
use crate::layer1::economy::trade::TradeShipArrivalEvent;
use crate::layer1::population::MigrantArrivalEvent;
use crate::shared::time::SimulationTime;

#[derive(Resource, Default)]
pub struct ColonyBeacon {
    pub is_active: bool,
}

#[derive(Event)]
pub struct PirateRaidEvent {
    pub threat_level: u32,
}

pub fn process_colony_beacon_system(
    beacon: Res<ColonyBeacon>,
    mut trade_writer: EventWriter<TradeShipArrivalEvent>,
    mut migrant_writer: EventWriter<MigrantArrivalEvent>,
    mut pirate_writer: EventWriter<PirateRaidEvent>,
) {
    if !beacon.is_active {
        return;
    }

    let mut rng = rand::thread_rng();

    // Significantly increased chances
    if rng.gen::<f32>() < 0.05 {
        trade_writer.send(TradeShipArrivalEvent);
    }

    if rng.gen::<f32>() < 0.05 {
        migrant_writer.send(MigrantArrivalEvent {
            count: rng.gen_range(5..15),
            criminal_chance: 0.3, // High chance of criminals
            low_skill_chance: 0.5, // High chance of grifters
        });
    }

    // Risk of pirate raids
    if rng.gen::<f32>() < 0.02 {
        pirate_writer.send(PirateRaidEvent { threat_level: rng.gen_range(1..5) });
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Parameters:** Ensure `MigrantArrivalEvent` in the existing codebase is updated to accept `criminal_chance` and `low_skill_chance` modifiers (or use a global `BeaconModifier` resource that the spawning system reads).
- **Toggle Cooldown:** Activating/deactivating the beacon should probably have a cooldown or an energy cost to prevent players from micro-toggling it just before a tick.
- **Lore Hooks:** Add `AddChronicleEvent` when the beacon is first lit ("The beacon is lit. We invite the galaxy, and all its scum, to our doors.").

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] Code coverage is ≥85% for the new module.
- [ ] Turning on the beacon drastically increases the frequency of `MigrantArrivalEvent` and `TradeShipArrivalEvent`.
- [ ] Turning on the beacon introduces a chance for `PirateRaidEvent`.
- [ ] Migrants arriving while the beacon is active have a noticeably higher chance of possessing negative/low-skill traits.

## 7. Technical Guidance
- **Integration:** The `process_colony_beacon_system` should be added to the appropriate simulation schedule, likely `Economy` or `Environment`.
- **UI:** A UI toggle will need to be added to interact with the `ColonyBeacon` resource.
- **Modifiers:** Instead of hardcoding probabilities in the system, consider pulling baseline rates from a configuration file and applying a multiplier when the beacon is active.

## 8. Questions
*Builder: add questions here if spec is unclear.*
