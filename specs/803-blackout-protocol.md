# The Blackout Protocol

## 1. Overview
Sometimes hiding is better than fighting. The Blackout Protocol is a global emergency toggle that instantly cuts all power and light emissions across the colony. This is primarily used to evade detection by sensitive orbital enemies or hunter-killer drone swarms. However, sudden total darkness severely stresses the population. Untrained Pops will panic, dropping tools and potentially triggering localized riots.

## 2. Dependencies
- Layer 1 `EnergyGrid` or power system.
- `Pop` needs and psychological stress trackers.
- Layer 2/3 threat detection mechanics (or proxy events representing them).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::energy::{EnergyGrid, PowerConsumer};
    use crate::layer1::pop::{Pop, Stress};

    fn setup_app() -> App {
        let mut app = App::new();
        app.init_resource::<BlackoutState>();
        app.add_systems(Update, (
            toggle_blackout_system,
            apply_blackout_power_cut_system,
            blackout_panic_system,
        ));
        app
    }

    #[test]
    fn test_blackout_cuts_power_to_consumers() {
        let mut app = setup_app();

        let machine = app.world.spawn(PowerConsumer {
            requires: 50,
            is_powered: true,
        }).id();

        // Trigger blackout
        app.world.resource_mut::<BlackoutState>().active = true;

        app.update();

        let consumer = app.world.get::<PowerConsumer>(machine).unwrap();
        assert_eq!(consumer.is_powered, false, "Consumer should lose power during blackout");
    }

    #[test]
    fn test_blackout_causes_panic_in_untrained_pops() {
        let mut app = setup_app();

        let untrained_pop = app.world.spawn((
            Pop,
            Stress { level: 10.0 },
            BlackoutTraining { is_trained: false },
        )).id();

        let trained_pop = app.world.spawn((
            Pop,
            Stress { level: 10.0 },
            BlackoutTraining { is_trained: true },
        )).id();

        app.world.resource_mut::<BlackoutState>().active = true;

        app.update();

        let untrained_stress = app.world.get::<Stress>(untrained_pop).unwrap();
        let trained_stress = app.world.get::<Stress>(trained_pop).unwrap();

        assert!(untrained_stress.level > 10.0, "Untrained pop should gain stress");
        assert_eq!(trained_stress.level, 10.0, "Trained pop should not gain stress");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::energy::PowerConsumer;
use crate::layer1::pop::Stress;

#[derive(Resource, Default)]
pub struct BlackoutState {
    pub active: bool,
}

#[derive(Component)]
pub struct BlackoutTraining {
    pub is_trained: bool,
}

pub fn toggle_blackout_system(
    // Placeholder for input/command handling to flip the state
) {}

pub fn apply_blackout_power_cut_system(
    blackout: Res<BlackoutState>,
    mut consumers: Query<&mut PowerConsumer>,
) {
    if blackout.is_changed() && blackout.active {
        for mut consumer in consumers.iter_mut() {
            consumer.is_powered = false;
        }
    }
}

pub fn blackout_panic_system(
    blackout: Res<BlackoutState>,
    mut pops: Query<(&mut Stress, Option<&BlackoutTraining>)>,
) {
    if blackout.active {
        for (mut stress, training) in pops.iter_mut() {
            let is_trained = training.map_or(false, |t| t.is_trained);
            if !is_trained {
                // Apply stress penalty per tick while blackout is active
                stress.level += 1.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** The `BlackoutState` needs to feed into whatever system manages enemy threat detection. If active, detection radius should shrink massively.
- **Exceptions:** Some critical systems (life support) might have shielded, isolated batteries that *don't* cut during a blackout, creating a dim red glow.
- **Lighting:** Integrate with the UI/Render pipeline to actually darken the terminal or web canvas when active.

## 6. Acceptance Criteria (Testable!)
- [ ] Toggling `BlackoutState` instantly unpowers all standard `PowerConsumer` components.
- [ ] Untrained pops gain stress while the blackout is active.
- [ ] Trained pops do not gain stress from the darkness.
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.

## 7. Technical Guidance
- The power cut needs to override normal `EnergyGrid` distribution logic. The simplest way is to intercept the distribution phase and zero out available power, rather than just flipping a boolean on the consumer, to ensure the grid doesn't falsely report a surplus.
- Consider an "All Clear" event when the blackout ends, giving a small morale boost.

## 8. Questions
*Builder: add questions here if spec is unclear.*
