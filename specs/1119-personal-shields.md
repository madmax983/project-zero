# 1119: Personal Shields

## 1. Overview
**Layer:** 1

High-tech "Kinetic Barriers" block high-velocity projectiles (bullets, shrapnel) but allow slow-moving objects (melee weapons) to pass through. This creates a tactical tension: advanced ranged guards can be overrun by primitive melee attacks if they rely entirely on shields. The shield consumes power to operate and mitigate damage.

## 2. Dependencies
- `004-pop-entity.md`
- `034-pop-health.md`

## 3. RED Phase: Tests First

```rust
// src/layer1/shields/tests.rs

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[derive(Component)]
    struct Pop;

    #[derive(Component)]
    struct Health {
        current: f32,
        max: f32,
    }

    #[derive(Component)]
    struct KineticBarrier {
        power: f32,
        max_power: f32,
        velocity_threshold: f32,
    }

    #[derive(Event)]
    struct DamageEvent {
        target: Entity,
        amount: f32,
        velocity: f32,
    }

    fn apply_damage_with_shields(
        mut events: EventReader<DamageEvent>,
        mut q_targets: Query<(&mut Health, Option<&mut KineticBarrier>)>,
    ) {
        for event in events.read() {
            if let Ok((mut health, barrier_opt)) = q_targets.get_mut(event.target) {
                let mut damage = event.amount;

                if let Some(mut barrier) = barrier_opt {
                    if event.velocity > barrier.velocity_threshold && barrier.power > 0.0 {
                        // High velocity: shield absorbs damage, consumes power
                        let absorbed = damage.min(barrier.power);
                        barrier.power -= absorbed;
                        damage -= absorbed;
                    }
                }

                health.current = (health.current - damage).max(0.0);
            }
        }
    }

    #[test]
    fn test_shield_blocks_fast_projectile() {
        let mut app = App::new();
        app.add_event::<DamageEvent>();
        app.add_systems(Update, apply_damage_with_shields);

        let target = app.world_mut().spawn((
            Pop,
            Health { current: 100.0, max: 100.0 },
            KineticBarrier { power: 50.0, max_power: 50.0, velocity_threshold: 10.0 },
        )).id();

        app.world_mut().send_event(DamageEvent {
            target,
            amount: 20.0,
            velocity: 50.0, // Fast
        });

        app.update();

        let health = app.world().get::<Health>(target).unwrap();
        let barrier = app.world().get::<KineticBarrier>(target).unwrap();

        assert_eq!(health.current, 100.0, "Fast projectile damage should be fully absorbed");
        assert_eq!(barrier.power, 30.0, "Barrier power should be reduced by absorbed damage");
    }

    #[test]
    fn test_shield_ignores_slow_melee() {
        let mut app = App::new();
        app.add_event::<DamageEvent>();
        app.add_systems(Update, apply_damage_with_shields);

        let target = app.world_mut().spawn((
            Pop,
            Health { current: 100.0, max: 100.0 },
            KineticBarrier { power: 50.0, max_power: 50.0, velocity_threshold: 10.0 },
        )).id();

        app.world_mut().send_event(DamageEvent {
            target,
            amount: 20.0,
            velocity: 5.0, // Slow
        });

        app.update();

        let health = app.world().get::<Health>(target).unwrap();
        let barrier = app.world().get::<KineticBarrier>(target).unwrap();

        assert_eq!(health.current, 80.0, "Slow melee damage should bypass the shield");
        assert_eq!(barrier.power, 50.0, "Barrier power should not be consumed by slow attacks");
    }

    #[test]
    fn test_shield_break_overflow_damage() {
        let mut app = App::new();
        app.add_event::<DamageEvent>();
        app.add_systems(Update, apply_damage_with_shields);

        let target = app.world_mut().spawn((
            Pop,
            Health { current: 100.0, max: 100.0 },
            KineticBarrier { power: 10.0, max_power: 50.0, velocity_threshold: 10.0 },
        )).id();

        app.world_mut().send_event(DamageEvent {
            target,
            amount: 25.0,
            velocity: 50.0, // Fast
        });

        app.update();

        let health = app.world().get::<Health>(target).unwrap();
        let barrier = app.world().get::<KineticBarrier>(target).unwrap();

        assert_eq!(barrier.power, 0.0, "Barrier should be completely depleted");
        assert_eq!(health.current, 85.0, "Overflow damage (15.0) should be applied to health");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct KineticBarrier {
    pub power: f32,
    pub max_power: f32,
    pub velocity_threshold: f32,
}

#[derive(Event)]
pub struct DamageEvent {
    pub target: Entity,
    pub amount: f32,
    pub velocity: f32,
}

pub fn apply_damage_with_shields(
    mut events: EventReader<DamageEvent>,
    mut q_targets: Query<(&mut Health, Option<&mut KineticBarrier>)>,
) {
    for event in events.read() {
        if let Ok((mut health, barrier_opt)) = q_targets.get_mut(event.target) {
            let mut damage = event.amount;

            if let Some(mut barrier) = barrier_opt {
                if event.velocity > barrier.velocity_threshold && barrier.power > 0.0 {
                    let absorbed = damage.min(barrier.power);
                    barrier.power -= absorbed;
                    damage -= absorbed;
                }
            }

            health.current = (health.current - damage).max(0.0);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** Integrate with `Equipment` system so pops can equip and unequip shield generators.
- **Power System:** Ensure shield recharge consumes global or personal battery power. Consider adding an `EnergyConsumer` component or logic to handle shield regeneration.
- **Visual Feedback:** Implement a `ShieldHitEvent` to render a brief visual barrier effect when a fast projectile is blocked.
- **Damage Types:** Consider moving velocity into an enum of `DamageType` (e.g., `DamageType::Kinetic(f32)`, `DamageType::Thermal`, `DamageType::Melee`) to make damage mitigation more expressive.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] `KineticBarrier` blocks high-velocity damage by consuming its power
- [ ] `KineticBarrier` lets low-velocity damage bypass completely
- [ ] Overflow high-velocity damage correctly applies to target `Health`

## 7. Technical Guidance
- Ensure that the damage pipeline runs in a predictable order. Shield mitigation should happen *before* actual health reduction.
- For the MVP, `DamageEvent` should include the `velocity` field to differentiate between fast projectiles and slow melee hits.

## 8. Questions
*Builder: add questions here if spec is unclear.*
