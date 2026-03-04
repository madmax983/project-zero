# 268: Subliminal Advertising

## 1. Overview

Monetizing the colony's wall space at the expense of citizen contentment. "Ad-Screens" are buildings placed in high-traffic areas. They generate passive `Credits` based on the number of Pops who walk near them. However, they artificially inflate the decay rate of `Luxury` and `Leisure` needs for those exposed Pops, creating a populace that is richer in corporate funds but deeply unsatisfied due to manufactured desires.

## 2. Dependencies

- `006` Building Placement
- `004` Pop Entity (Needs)
- `039` Trade System (Credits/Allowance)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::{Need, NeedTracker};
    use crate::layer1::trade::Allowance;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, update_ad_screens_system);
        app
    }

    #[test]
    fn test_ad_screen_generates_credits_and_inflates_needs() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            NeedTracker {
                leisure: Need { value: 100.0, decay_rate: 1.0 },
                luxury: Need { value: 100.0, decay_rate: 1.0 },
                ..Default::default()
            },
        )).id();

        let screen_id = app.world_mut().spawn((
            Transform::from_xyz(1.0, 0.0, 0.0),
            AdScreen {
                radius: 2.0,
                credits_per_pop: 0.5,
                need_decay_multiplier: 2.0,
                accumulated_credits: 0.0,
            },
        )).id();

        app.update(); // Tick 1

        let screen = app.world().get::<AdScreen>(screen_id).unwrap();
        assert_eq!(screen.accumulated_credits, 0.5);

        let needs = app.world().get::<NeedTracker>(pop_id).unwrap();
        // Base decay (1.0) * Multiplier (2.0) = 2.0 decay
        assert_eq!(needs.leisure.value, 98.0);
        assert_eq!(needs.luxury.value, 98.0);
    }

    #[test]
    fn test_ad_screen_ignores_pops_out_of_range() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Transform::from_xyz(10.0, 0.0, 0.0), // Out of range
            NeedTracker {
                leisure: Need { value: 100.0, decay_rate: 1.0 },
                luxury: Need { value: 100.0, decay_rate: 1.0 },
                ..Default::default()
            },
        )).id();

        let screen_id = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            AdScreen {
                radius: 2.0,
                credits_per_pop: 0.5,
                need_decay_multiplier: 2.0,
                accumulated_credits: 0.0,
            },
        )).id();

        app.update();

        let screen = app.world().get::<AdScreen>(screen_id).unwrap();
        assert_eq!(screen.accumulated_credits, 0.0); // No credits generated

        let needs = app.world().get::<NeedTracker>(pop_id).unwrap();
        // Only base decay applies
        assert_eq!(needs.leisure.value, 99.0);
        assert_eq!(needs.luxury.value, 99.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::needs::NeedTracker;

#[derive(Component)]
pub struct AdScreen {
    pub radius: f32,
    pub credits_per_pop: f32,
    pub need_decay_multiplier: f32,
    pub accumulated_credits: f32,
}

pub fn update_ad_screens_system(
    mut screens: Query<(&Transform, &mut AdScreen)>,
    mut pops: Query<(&Transform, &mut NeedTracker)>,
) {
    for (screen_transform, mut screen) in screens.iter_mut() {
        let screen_pos = screen_transform.translation.truncate();
        let radius_sq = screen.radius * screen.radius;

        for (pop_transform, mut needs) in pops.iter_mut() {
            let pop_pos = pop_transform.translation.truncate();
            if screen_pos.distance_squared(pop_pos) <= radius_sq {
                // Generate credits
                screen.accumulated_credits += screen.credits_per_pop;

                // Inflate need decay (apply the extra decay here)
                // Assuming base decay is applied elsewhere, we add the difference
                let extra_leisure_decay = needs.leisure.decay_rate * (screen.need_decay_multiplier - 1.0);
                let extra_luxury_decay = needs.luxury.decay_rate * (screen.need_decay_multiplier - 1.0);

                needs.leisure.value = (needs.leisure.value - extra_leisure_decay).max(0.0);
                needs.luxury.value = (needs.luxury.value - extra_luxury_decay).max(0.0);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Performance**: The $O(N \times M)$ distance check (Screens vs Pops) can become expensive. Consider using a spatial hash grid or running this check less frequently (e.g., using a fixed timestep or checking only once per second instead of every tick).
- **Integration**: The accumulated credits need to be transferred to the global `ColonyResources` or a faction vault periodically.
- **Buffs/Debuffs Structure**: Directly modifying `needs.value` in the system circumvents the standard `metabolism_system`. It would be cleaner to apply an `AuraEffect` or a temporary modifier component (`AdTargeted`) to the Pop, which the `metabolism_system` then reads to scale decay rates.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for the new module.
- [ ] Pops near an `AdScreen` have their Leisure and Luxury decay faster.
- [ ] `AdScreen` accumulates credits proportionally to the number of nearby Pops.

## 7. Technical Guidance

- Implement `AdScreen` in a new file `src/layer1/artifacts/ad_screen.rs`.
- For the MVP, it's acceptable to do the extra decay deduction directly in the `update_ad_screens_system`, but strongly consider adding a `NeedDecayModifier` component if the architecture supports it easily.
- Register the system in `src/layer1/systems/artifacts.rs` or similar.

## 8. Questions

*Builder: add questions here if spec is unclear. Architect will address.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
