# Spec 694: Corporate Sponsorship

## 1. Overview
This colony is brought to you by *Lightspeed Cola*. Accept funding/resources from a Layer 3 Corporation. In exchange, you must build "Billboards" (consume power, produce no resources) and use their specific, DRM-locked tech (cheaper but unrepairable).

## 2. Dependencies
- Layer 3 Diplomatic/Trade interface (or a stand-in event system)
- Building construction system
- Resource/Economy system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_accepting_sponsorship_grants_resources_and_contract() {
        let mut app = App::new();
        // Setup resources and sponsor event
        // Trigger accept sponsorship
        // Assert resources increased
        // Assert a CorporateContract resource/component is active
    }

    #[test]
    fn test_billboard_requirement_enforcement() {
        let mut app = App::new();
        // Setup active CorporateContract requiring 1 Billboard
        // Advance time without building Billboard
        // Assert penalty applied (e.g., fine or relationship drop)
    }

    #[test]
    fn test_drm_locked_tech_cannot_be_repaired() {
        let mut app = App::new();
        // Spawn a DRM-locked building (from sponsorship)
        // Damage it
        // Attempt to dispatch a repair job
        // Assert the repair job is invalid or fails
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Resource)]
pub struct CorporateContract {
    pub sponsor_name: String,
    pub required_billboards: u32,
    pub penalty_timer: Timer,
}

#[derive(Component)]
pub struct DrmLocked; // Marker component for unrepairable tech
```

## 5. REFACTOR Phase: Quality & Design
- Tie the sponsorship offers into the existing event or trade request queue.
- Clearly communicate the unrepairable nature of DRM tech in the UI.
- Handle edge cases where the sponsor faction ceases to exist on Layer 3.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Sponsorships provide an initial burst of resources but enforce long-term restrictive building requirements.

## 7. Technical Guidance
- The `DrmLocked` marker can be checked in the utility AI or job assignment system to filter out repair tasks.
- Billboards should just be a new building type that consumes power but provides no other output.

## 8. Questions
*Builder: add questions here if spec is unclear.*
