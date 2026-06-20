# 693: Corporate Sponsorship

## Overview

A feature allowing colonies to accept funding and resources from a Layer 3 Corporation. In exchange, the colony is forced to construct "Billboards" (which consume power and space but produce nothing) and must use specific, DRM-locked technology. If the DRM-locked tech breaks, it cannot be repaired locally, creating a tension between quick cash injections and long-term autonomy.

## Dependencies

- `018` — Mining and Resources
- `039` — Trade System

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::architecture::BuildingType;
    use crate::layer1::health::Health;

    #[test]
    fn test_accepting_sponsorship_grants_resources() {
        let mut app = App::new();
        app.init_resource::<ColonyResources>();
        app.world_mut().resource_mut::<ColonyResources>().credits = 0.0;

        let deal = SponsorshipDeal {
            corporation_id: "lightspeed_cola".to_string(),
            upfront_credits: 5000.0,
            required_billboards: 3,
            drm_tech_unlocked: vec!["nano_med_bay".to_string()],
        };

        // Accept the deal
        app.world_mut().send_event(AcceptSponsorshipEvent { deal });
        app.add_systems(Update, process_sponsorship_acceptance);
        app.update();

        // Colony should have received the credits
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.credits, 5000.0);
    }

    #[test]
    fn test_billboard_requirement_enforced() {
        let mut app = App::new();
        app.init_resource::<ColonyResources>();
        app.world_mut().resource_mut::<ColonyResources>().credits = 1000.0;

        let deal = SponsorshipDeal {
            corporation_id: "megacorp".to_string(),
            upfront_credits: 1000.0,
            required_billboards: 2,
            drm_tech_unlocked: vec![],
        };
        app.world_mut().insert_resource(ActiveSponsorship {
            deal,
            billboards_built: 0,
            breach_timer: 1, // Ticks until breach if not built
        });

        // Advance time without building billboards
        app.add_systems(Update, enforce_sponsorship_requirements);
        app.update();

        // A breach event or penalty should be triggered
        let penalty = app.world().resource::<ColonyResources>();
        assert!(penalty.credits < 1000.0); // Penalty applied
    }

    #[test]
    fn test_drm_tech_cannot_be_repaired() {
        let mut app = App::new();
        let building_id = app.world_mut().spawn((
            Building {
                building_type: BuildingType::Hospital,
            },
            Health {
                current: 10.0,
                max: 100.0,
                has_rust_lung: false,
            },
            DrmLocked, // Sponsored building
        )).id();

        // Attempt a repair action
        app.world_mut().send_event(RepairBuildingEvent {
            target: building_id,
            repair_amount: 50.0,
        });

        app.add_systems(Update, handle_repair_requests);
        app.update();

        // Health should not increase because it is DRM locked
        let health = app.world().get::<Health>(building_id).unwrap();
        assert_eq!(health.current, 10.0);
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::architecture::{Building, BuildingType};
use crate::layer1::health::Health;

#[derive(Clone, Debug)]
pub struct SponsorshipDeal {
    pub corporation_id: String,
    pub upfront_credits: f32,
    pub required_billboards: u32,
    pub drm_tech_unlocked: Vec<String>,
}

#[derive(Event)]
pub struct AcceptSponsorshipEvent {
    pub deal: SponsorshipDeal,
}

#[derive(Resource)]
pub struct ActiveSponsorship {
    pub deal: SponsorshipDeal,
    pub billboards_built: u32,
    pub breach_timer: i32,
}

#[derive(Component)]
pub struct DrmLocked;

#[derive(Event)]
pub struct RepairBuildingEvent {
    pub target: Entity,
    pub repair_amount: f32,
}

// Ensure ColonyResources exists for the green phase to compile
#[derive(Resource, Default)]
pub struct ColonyResources {
    pub credits: f32,
}

pub fn process_sponsorship_acceptance(
    mut events: EventReader<AcceptSponsorshipEvent>,
    mut commands: Commands,
    mut resources: ResMut<ColonyResources>,
) {
    for event in events.read() {
        resources.credits += event.deal.upfront_credits;
        commands.insert_resource(ActiveSponsorship {
            deal: event.deal.clone(),
            billboards_built: 0,
            breach_timer: 1000,
        });
    }
}

pub fn enforce_sponsorship_requirements(
    sponsorship: Option<ResMut<ActiveSponsorship>>,
    mut resources: ResMut<ColonyResources>,
    billboard_query: Query<&Building>,
) {
    let mut billboard_count = 0;
    for building in billboard_query.iter() {
        if building.building_type == BuildingType::Billboard {
            billboard_count += 1;
        }
    }

    if let Some(mut s) = sponsorship {
        s.billboards_built = billboard_count;
        if s.billboards_built < s.deal.required_billboards {
            s.breach_timer -= 1;
            if s.breach_timer <= 0 {
                // Apply a generic penalty
                resources.credits -= 500.0;
                s.breach_timer = 1000; // Reset timer for next penalty
            }
        }
    }
}

pub fn handle_repair_requests(
    mut events: EventReader<RepairBuildingEvent>,
    mut buildings: Query<(Option<&DrmLocked>, &mut Health)>,
) {
    for event in events.read() {
        if let Ok((drm_locked, mut health)) = buildings.get_mut(event.target) {
            if drm_locked.is_none() {
                health.current += event.repair_amount;
                if health.current > health.max {
                    health.current = health.max;
                }
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Integration**: Integrate the `AcceptSponsorshipEvent` into the UI (e.g., a "Trade Offers" screen) and hook the DRM repair denial into the Pop `UtilityAI` so Pops don't waste time trying to pathfind to a broken DRM building.
- **Generics**: Generalize `ActiveSponsorship` into a `Contract` or `Edict` system that handles various types of agreements (e.g., Penal Contracts, Imperial Quotas) using a trait `evaluate_conditions`.
- **Penalties**: Change the penalty for breaching a contract from a simple credit deduction to a diplomatic hit on Layer 3, potentially spawning a "Repo Fleet".

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Accepting a sponsorship grants the expected resources and sets an active contract.
- [ ] Failing to build required billboards triggers a penalty.
- [ ] Buildings flagged as `DrmLocked` reject repair attempts.

## Technical Guidance

- Use the existing `ColonyResources` struct when modifying credits or resources.
- Ensure the `breach_timer` uses `SimulationTime` rather than an arbitrary tick decrement in final implementation.
- Tie the `DrmLocked` flag to the `TechTree` so specific nodes (like "Sponsored Hospital") automatically spawn with the flag.

## Questions

*Builder: add questions here if spec is unclear.*
- **Architectural Contradictions:** Building uses string for `building_type` which conflicts with enum `BuildingType` and missing `RepairBuildingEvent`.

*Architect:* Addressed. The `Building` component correctly uses the `BuildingType` enum, and the `RepairBuildingEvent` and `Health` structures have been updated to match the existing architecture.
