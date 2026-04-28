# 1245: Chronological Stutter

## 1. Overview
**Layer:** 2
**Fantasy:** Moving through space where time itself skips like a scratched record.
**Mechanic:** Passing through certain unstable hyperlanes causes fleets to experience "Chronological Stutter." They arrive at their destination physically fine, but their internal clocks are scrambled. Some ships arrive weeks before they left, while others arrive years late. This completely disrupts coordinated fleet actions.

## 2. Dependencies
- Layer 2 Fleet Movement System
- Layer 2 Hyperlane/Map System
- Time/Chronicle System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_chronological_stutter_delays_arrival() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<SimulationTime>();

        let fleet = app.world_mut().spawn(Fleet { arrival_time: 100 }).id();

        app.add_event::<HyperlaneTransitEvent>();
        app.world_mut().send_event(HyperlaneTransitEvent {
            fleet,
            is_unstable: true,
        });

        // Act
        app.add_systems(Update, apply_chronological_stutter_system);
        app.update();

        // Assert
        let updated_fleet = app.world().get::<Fleet>(fleet).unwrap();
        assert_ne!(updated_fleet.arrival_time, 100, "Unstable transit should scramble arrival time");
    }

    #[test]
    fn test_stable_transit_preserves_arrival() {
        // Arrange
        let mut app = App::new();

        let fleet = app.world_mut().spawn(Fleet { arrival_time: 100 }).id();

        app.add_event::<HyperlaneTransitEvent>();
        app.world_mut().send_event(HyperlaneTransitEvent {
            fleet,
            is_unstable: false, // Stable lane
        });

        // Act
        app.add_systems(Update, apply_chronological_stutter_system);
        app.update();

        // Assert
        let updated_fleet = app.world().get::<Fleet>(fleet).unwrap();
        assert_eq!(updated_fleet.arrival_time, 100, "Stable transit should preserve exact arrival time");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use rand::Rng;

#[derive(Resource, Default)]
pub struct SimulationTime {
    pub tick: u64,
}

#[derive(Component)]
pub struct Fleet {
    pub arrival_time: u64,
}

#[derive(Event)]
pub struct HyperlaneTransitEvent {
    pub fleet: Entity,
    pub is_unstable: bool,
}

pub fn apply_chronological_stutter_system(
    mut events: EventReader<HyperlaneTransitEvent>,
    mut fleets: Query<&mut Fleet>,
) {
    let mut rng = rand::thread_rng();

    for ev in events.read() {
        if ev.is_unstable {
            if let Ok(mut fleet) = fleets.get_mut(ev.fleet) {
                // Scramble arrival time randomly (could be early or very late)
                let stutter_amount: i64 = rng.gen_range(-50..500);

                // Ensure we don't go backwards past 0 tick
                let new_time = (fleet.arrival_time as i64 + stutter_amount).max(0) as u64;
                fleet.arrival_time = new_time;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Currently, fleets are moved as a single unit. Chronological stutter implies individual *ships* within the fleet arrive at different times. Refactor fleet structures so that ships hold their own local `arrival_time`, effectively scattering the armada upon exit.
- Integrate with the Chronicle system: arriving before you left is a paradox that should be logged as a major lore event.
- Allow high-tech "Temporal Anchors" on ships to mitigate the stutter effect.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Fleets transiting unstable lanes have their `arrival_time` mutated randomly.
- [ ] Stable lanes do not affect arrival times.

## 7. Technical Guidance
- **Arrival Logic:** Ensure whatever system processes `Fleet` arrivals checks `SimulationTime.tick >= arrival_time`. Do not use absolute equality `==`, as time skipping might miss it.
- **RNG:** Use `rand::thread_rng()` for the MVP, but be aware this makes tests non-deterministic without seed injection if you want to test exact ranges.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
