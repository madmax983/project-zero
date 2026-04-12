# Spec 971: The Living Constitution

## 1. Overview
Colony Edicts (laws) are not static; they gain XP over time. If an edict like "Martial Law" or "Rationing" is active for a long duration, it becomes a "Tradition". Once an edict becomes a Tradition, attempting to remove or revoke it causes a massive, immediate spike in global Unrest, as the Pops have internalized it as part of their culture.

**Fantasy:** Laws evolve. You enacted rationing during a war, and now five years later, they refuse to stop because "it's our way."

## 2. Dependencies
- Layer 1 Policy/Edict system
- Layer 1 Social Unrest system (`src/layer1/social/unrest.rs`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::social::unrest::Unrest;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(ColonyPolicies::default());
        app.insert_resource(Unrest::default());
        app.init_resource::<Events<RevokePolicyEvent>>();
        app.add_systems(Update, (
            update_policy_tradition_system,
            handle_revoke_policy_system,
        ));
        app
    }

    #[test]
    fn test_policy_gains_tradition_xp_over_time() {
        let mut app = setup_app();

        // Enact a policy
        app.world_mut().resource_mut::<ColonyPolicies>().active_policies.insert(PolicyType::Rationing, PolicyState { active: true, duration: 0, is_tradition: false });

        app.update();

        let policies = app.world().resource::<ColonyPolicies>();
        let state = policies.active_policies.get(&PolicyType::Rationing).unwrap();
        assert_eq!(state.duration, 1, "Policy duration should increase per tick");
    }

    #[test]
    fn test_policy_becomes_tradition_after_duration() {
        let mut app = setup_app();

        // Enact a policy near the threshold
        app.world_mut().resource_mut::<ColonyPolicies>().active_policies.insert(PolicyType::Rationing, PolicyState { active: true, duration: 99, is_tradition: false });

        app.update(); // Tick 100

        let policies = app.world().resource::<ColonyPolicies>();
        let state = policies.active_policies.get(&PolicyType::Rationing).unwrap();
        assert!(state.is_tradition, "Policy should become a tradition after hitting duration threshold");
    }

    #[test]
    fn test_revoking_tradition_causes_unrest() {
        let mut app = setup_app();

        // Enact a tradition
        app.world_mut().resource_mut::<ColonyPolicies>().active_policies.insert(PolicyType::MartialLaw, PolicyState { active: true, duration: 150, is_tradition: true });

        let initial_unrest = app.world().resource::<Unrest>().level;

        // Revoke the tradition
        app.world_mut().resource_mut::<Events<RevokePolicyEvent>>().send(RevokePolicyEvent { policy: PolicyType::MartialLaw });

        app.update();

        let unrest = app.world().resource::<Unrest>();
        assert!(unrest.level > initial_unrest, "Revoking a tradition should heavily increase unrest");

        let policies = app.world().resource::<ColonyPolicies>();
        assert!(!policies.active_policies.contains_key(&PolicyType::MartialLaw), "Policy should be revoked");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

- Create `src/layer1/social/tradition.rs` (or add to `src/layer1/social/politics.rs` or where `ColonyPolicies` lives).
- Ensure `ColonyPolicies` has an `active_policies: HashMap<PolicyType, PolicyState>` where `PolicyState` has `duration: u32` and `is_tradition: bool`.
- Add `update_policy_tradition_system`:
  - Iterate through `active_policies`.
  - Increment `duration`.
  - If `duration >= 100` (or some configured TRADITION_THRESHOLD), set `is_tradition = true`.
- Update the system that handles policy revocation (or implement `handle_revoke_policy_system` listening to `RevokePolicyEvent`):
  - Check if the policy to be revoked `is_tradition`.
  - If yes, add a massive `UnrestModifier` (e.g., +0.5 value) to the `Unrest` resource.
  - Remove the policy from `active_policies`.

## 5. REFACTOR Phase: Quality & Design
- **Modifier Falloff:** The unrest modifier from revoking a tradition should probably decay over time so the colony doesn't permanently collapse.
- **UI Signals:** When a policy becomes a tradition, emit a `TraditionEstablishedEvent` for the chronicle/UI so the player is warned that removing it will hurt.

## 6. Acceptance Criteria
- [ ] Tests compile and pass in RED phase.
- [ ] Policies accumulate duration ticks when active.
- [ ] Policies flip to `is_tradition = true` when duration exceeds threshold.
- [ ] Revoking a tradition heavily penalizes global unrest.
- [ ] Test coverage ≥85%.
- [ ] Code passes `clippy -- -D warnings` and `cargo fmt`.

## 7. Technical Guidance
- Locate `ColonyPolicies` in the codebase (likely in `src/layer1/policies.rs` or similar). If it doesn't exist, you'll need to scaffold a simple version of it for this spec.
- Unrest modifiers are typically added to `Unrest.modifiers`. Be sure to use the correct `UnrestModifier` structure (it often has a `value` and `duration` field).

## 8. Questions
*Builder: add questions here if spec is unclear.*
