# 616: Cryo-Stasis Vaults

## 1. Overview
The ultimate survival mechanism. Constructible "Cryo-Pods" allow players to manually freeze pops, stopping their hunger and needs consumption entirely. This provides an emergency release valve during famines or extreme resource shortages. However, thawing takes time and causes "Cryo-Sickness", introducing long-term recovery costs. Players face hard choices regarding who gets the pod and who stays awake to keep the power running.

## 2. Dependencies
- Layer 1 Build System (for constructing pods)
- Layer 1 Needs & Consumption System
- Layer 1 Health System (for sickness effects)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use layer1::needs::{Hunger, Thirst};
    use layer1::health::CryoSickness;

    #[test]
    fn test_frozen_pops_do_not_consume_needs() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, process_needs_system);

        // Setup pop: Frozen pop should NOT accumulate hunger
        let pop_entity = app.world_mut().spawn((Frozen, Hunger(0), Thirst(0))).id();

        app.update();

        let hunger = app.world().get::<Hunger>(pop_entity).unwrap().0;
        assert_eq!(hunger, 0, "Frozen pops should not consume needs");
    }

    #[test]
    fn test_thawing_pops_causes_sickness() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, thaw_cryo_system);

        // Setup pop thawing from a pod
        let pop_entity = app.world_mut().spawn((Thawing { time_left: 0 }, Hunger(0))).id();

        app.update();

        // Thawing should be removed and CryoSickness should be added
        assert!(app.world().get::<Thawing>(pop_entity).is_none(), "Thawing state should be removed when complete");
        assert!(app.world().get::<CryoSickness>(pop_entity).is_some(), "Thawed pops should receive CryoSickness");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct Frozen;

#[derive(Component)]
pub struct Thawing {
    pub time_left: u32,
}

#[derive(Component)]
pub struct CryoSickness;

pub fn process_needs_system(
    mut needs_query: Query<(&mut Hunger, &mut Thirst), Without<Frozen>>,
) {
    // Only process needs for active (unfrozen) pops
    for (mut hunger, mut thirst) in needs_query.iter_mut() {
        hunger.0 += 1;
        thirst.0 += 1;
    }
}

pub fn thaw_cryo_system(
    mut commands: Commands,
    mut thawing_query: Query<(Entity, &mut Thawing)>,
) {
    for (entity, mut thawing) in thawing_query.iter_mut() {
        if thawing.time_left == 0 {
            commands.entity(entity).remove::<Thawing>().insert(CryoSickness);
        } else {
            thawing.time_left -= 1;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Abstract the `Without<Frozen>` filter into generic needs processing loops instead of manually updating it for each resource individually.
- Implement time-based duration instead of `time_left` ticks for the `Thawing` state.
- Define actual debuffs for `CryoSickness` (e.g., reduced move speed, lower work efficiency).
- Send an event upon successful thawing to handle UI notification.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops in Cryo-Stasis do not consume any needs (Hunger/Thirst).
- [ ] Thawing transitions a pop to active state and applies CryoSickness.

## 7. Technical Guidance
- **Interactions:** The act of freezing should require the pop to pathfind to the Cryo-Pod and successfully enter it.
- **Power Requirement:** Pods should ideally consume a small amount of power. If power fails, pops inside should begin to thaw automatically or die (dependent on difficulty settings).
- **Thawing Process:** Thawing should take several simulation ticks to complete before the pop is usable.

## 8. Questions
*Builder: add questions here if spec is unclear.*
