# 788: The Chrono-Schism

## 1. Overview
**Layer:** 2 -> 3
**Fantasy:** Accidentally fracturing time through excessive FTL usage.
**Mechanic:** Using unregulated experimental FTL drives in a specific system too frequently creates a localized "Chrono-Schism." Fleets that enter the system might arrive years before they left, or years after, causing them to fight alongside past versions of themselves or arrive to find the war already lost.
**Emergence:** You desperately need reinforcements in a losing war, so you risk jumping through a Chrono-Schism. Your fleet arrives a decade late, finding your empire already conquered, forcing you to play as a lone, rogue armada trying to liberate your own descendants.
**Tension:** The immediate tactical need for rapid deployment vs. the risk of temporally misplacing your entire armada.

## 2. Dependencies
- Layer 2 Fleet Movement System (`Fleet`, `Navigation`)
- Layer 2 FTL Drive System (`FTLDrive`, `DriveType`)
- Layer 3 System Status (`StarSystem`, `SystemHazards`)
- Layer 3 Chronicle System (to record the temporal displacement)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer2::fleet::Fleet;
    use crate::layer3::map::StarSystem;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            track_experimental_ftl_usage_system,
            apply_chrono_schism_displacement_system
        ));
        app.init_resource::<GlobalTime>();
        app
    }

    #[test]
    fn test_experimental_ftl_increases_schism_risk() {
        let mut app = setup_app();

        let system_entity = app.world_mut().spawn((
            StarSystem,
            ChronoSchismRisk { risk_level: 0.0 },
        )).id();

        // Simulate an FTL jump into the system
        app.world_mut().spawn((
            Fleet,
            FtlJumpEvent { target_system: system_entity, is_experimental: true },
        ));

        app.update();

        let risk = app.world().get::<ChronoSchismRisk>(system_entity).unwrap();
        assert!(risk.risk_level > 0.0, "Experimental FTL jumps should increase Chrono-Schism risk in the system");
    }

    #[test]
    fn test_chrono_schism_displaces_fleet_temporally() {
        let mut app = setup_app();
        app.world_mut().resource_mut::<GlobalTime>().current_year = 2200;

        let system_entity = app.world_mut().spawn((
            StarSystem,
            ChronoSchismRisk { risk_level: 1.0 }, // Guaranteed displacement
        )).id();

        let fleet_entity = app.world_mut().spawn((
            Fleet,
            FtlArrivalEvent { target_system: system_entity },
            TemporalDisplacement { years_shifted: 0 },
        )).id();

        app.update();

        let displacement = app.world().get::<TemporalDisplacement>(fleet_entity).unwrap();
        assert!(displacement.years_shifted != 0, "Fleet arriving in a Chrono-Schism should be temporally displaced");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer2::fleet::Fleet;
use crate::layer3::map::StarSystem;

#[derive(Resource, Default)]
pub struct GlobalTime {
    pub current_year: i32,
}

#[derive(Component)]
pub struct ChronoSchismRisk {
    pub risk_level: f32, // 0.0 to 1.0
}

#[derive(Component)]
pub struct FtlJumpEvent {
    pub target_system: Entity,
    pub is_experimental: bool,
}

#[derive(Component)]
pub struct FtlArrivalEvent {
    pub target_system: Entity,
}

#[derive(Component)]
pub struct TemporalDisplacement {
    pub years_shifted: i32,
}

pub fn track_experimental_ftl_usage_system(
    jump_query: Query<&FtlJumpEvent>,
    mut system_query: Query<&mut ChronoSchismRisk>,
) {
    for jump in jump_query.iter() {
        if jump.is_experimental {
            if let Ok(mut risk) = system_query.get_mut(jump.target_system) {
                risk.risk_level += 0.1;
                if risk.risk_level > 1.0 {
                    risk.risk_level = 1.0;
                }
            }
        }
    }
}

pub fn apply_chrono_schism_displacement_system(
    arrival_query: Query<(Entity, &FtlArrivalEvent)>,
    system_query: Query<&ChronoSchismRisk>,
    mut commands: Commands,
) {
    for (entity, arrival) in arrival_query.iter() {
        if let Ok(risk) = system_query.get(arrival.target_system) {
            // Simplified displacement: if risk is maxed, shift them by 10 years (past or future)
            if risk.risk_level >= 1.0 {
                commands.entity(entity).insert(TemporalDisplacement { years_shifted: 10 }); // Always 10 for basic implementation
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Displacement Logic:** The years shifted should be random (both past and future) and proportional to the risk level. We need a `rand` dependency to make it unpredictable.
- **Chronicle Integration:** A massive displacement should fire an `AddChronicleEvent` to record the fleet's disappearance and sudden return in the timeline.
- **Visuals:** Add a `ChronoSchismMarker` component to the `StarSystem` when the risk is high to render a distorted visual effect on the galaxy map.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `chrono_schism`.
- [ ] Experimental FTL usage reliably increases `ChronoSchismRisk` on the target system.
- [ ] High `ChronoSchismRisk` triggers a temporal displacement event upon fleet arrival.

## 7. Technical Guidance
- Temporal displacement means the fleet should effectively disappear from the active simulation until `GlobalTime::current_year` matches their `target_year`. Alternatively, if displaced into the past, spawn a temporary duplicate "ghost" fleet.
- Clean up processed events (`FtlJumpEvent`, `FtlArrivalEvent`) at the end of the tick.
- Balance the `risk_level` accumulation so it takes a sustained campaign to fracture a system's timeline.

## 8. Questions
*Builder: add questions here if spec is unclear.*
