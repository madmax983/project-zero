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

    #[test]
    fn test_accepting_sponsorship_grants_resources() {
        let mut app = App::new();
        app.init_resource::<ColonyResources>();
        app.world_mut().resource_mut::<ColonyResources>().credits = 0;

        let deal = SponsorshipDeal {
            corporation_id: "lightspeed_cola".to_string(),
            upfront_credits: 5000,
            required_billboards: 3,
            drm_tech_unlocked: vec!["nano_med_bay".to_string()],
        };

        // Accept the deal
        app.world_mut().send_event(AcceptSponsorshipEvent { deal });
        app.update();

        // Colony should have received the credits
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.credits, 5000);
    }

    #[test]
    fn test_billboard_requirement_enforced() {
        let mut app = App::new();
        let deal = SponsorshipDeal {
            corporation_id: "megacorp".to_string(),
            upfront_credits: 1000,
            required_billboards: 2,
            drm_tech_unlocked: vec![],
        };
        app.world_mut().insert_resource(ActiveSponsorship {
            deal,
            billboards_built: 0,
            breach_timer: 100, // Ticks until breach if not built
        });

        // Advance time without building billboards
        app.world_mut().resource_mut::<ActiveSponsorship>().breach_timer = 0;
        app.update();

        // A breach event or penalty should be triggered
        let penalty = app.world().resource::<ColonyResources>();
        assert!(penalty.credits < 1000); // Penalty applied
    }

    #[test]
    fn test_drm_tech_cannot_be_repaired() {
        let mut app = App::new();
        let building_id = app.world_mut().spawn(Building {
            building_type: BuildingType::Hospital,
            health: 10.0,
            max_health: 100.0,
            is_drm_locked: true, // Sponsored building
        }).id();

        // Attempt a repair action
        app.world_mut().send_event(RepairBuildingEvent {
            target: building_id,
            repair_amount: 50.0,
        });

        app.update();

        // Health should not increase because it is DRM locked
        let building = app.world().get::<Building>(building_id).unwrap();
        assert_eq!(building.health, 10.0);
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct SponsorshipDeal {
    pub corporation_id: String,
    pub upfront_credits: i32,
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
pub struct Building {
    pub building_type: String,
    pub health: f32,
    pub max_health: f32,
    pub is_drm_locked: bool,
}

#[derive(Event)]
pub struct RepairBuildingEvent {
    pub target: Entity,
    pub repair_amount: f32,
}

// Ensure ColonyResources exists for the green phase to compile
#[derive(Resource, Default)]
pub struct ColonyResources {
    pub credits: i32,
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
    mut sponsorship: Option<ResMut<ActiveSponsorship>>,
    mut resources: ResMut<ColonyResources>,
) {
    if let Some(mut s) = sponsorship {
        if s.billboards_built < s.deal.required_billboards {
            s.breach_timer -= 1;
            if s.breach_timer <= 0 {
                // Apply a generic penalty
                resources.credits -= 500;
                s.breach_timer = 1000; // Reset timer for next penalty
            }
        }
    }
}

pub fn handle_repair_requests(
    mut events: EventReader<RepairBuildingEvent>,
    mut buildings: Query<&mut Building>,
) {
    for event in events.read() {
        if let Ok(mut building) = buildings.get_mut(event.target) {
            if !building.is_drm_locked {
                building.health += event.repair_amount;
                if building.health > building.max_health {
                    building.health = building.max_health;
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
- [ ] Buildings flagged as `is_drm_locked` reject repair attempts.

## Technical Guidance

- Use the existing `ColonyResources` struct when modifying credits or resources.
- Ensure the `breach_timer` uses `SimulationTime` rather than an arbitrary tick decrement in final implementation.
- Tie the `is_drm_locked` flag to the `TechTree` so specific nodes (like "Sponsored Hospital") automatically spawn with the flag.

## Questions

*Builder: add questions here if spec is unclear.*
