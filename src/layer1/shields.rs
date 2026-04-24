use crate::layer1::biology::health::Health;
use bevy_ecs::prelude::*;

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

            health.take_damage(damage);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_shield_blocks_fast_projectile() {
        let mut app = App::new();
        app.add_event::<DamageEvent>();
        app.add_systems(Update, apply_damage_with_shields);

        let target = app
            .world_mut()
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                KineticBarrier {
                    power: 50.0,
                    max_power: 50.0,
                    velocity_threshold: 10.0,
                },
            ))
            .id();

        app.world_mut().send_event(DamageEvent {
            target,
            amount: 20.0,
            velocity: 50.0, // Fast
        });

        app.update();

        let health = app.world().get::<Health>(target).unwrap();
        let barrier = app.world().get::<KineticBarrier>(target).unwrap();

        assert_eq!(
            health.current, 100.0,
            "Fast projectile damage should be fully absorbed"
        );
        assert_eq!(
            barrier.power, 30.0,
            "Barrier power should be reduced by absorbed damage"
        );
    }

    #[test]
    fn test_shield_ignores_slow_melee() {
        let mut app = App::new();
        app.add_event::<DamageEvent>();
        app.add_systems(Update, apply_damage_with_shields);

        let target = app
            .world_mut()
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                KineticBarrier {
                    power: 50.0,
                    max_power: 50.0,
                    velocity_threshold: 10.0,
                },
            ))
            .id();

        app.world_mut().send_event(DamageEvent {
            target,
            amount: 20.0,
            velocity: 5.0, // Slow
        });

        app.update();

        let health = app.world().get::<Health>(target).unwrap();
        let barrier = app.world().get::<KineticBarrier>(target).unwrap();

        assert_eq!(
            health.current, 80.0,
            "Slow melee damage should bypass the shield"
        );
        assert_eq!(
            barrier.power, 50.0,
            "Barrier power should not be consumed by slow attacks"
        );
    }

    #[test]
    fn test_shield_break_overflow_damage() {
        let mut app = App::new();
        app.add_event::<DamageEvent>();
        app.add_systems(Update, apply_damage_with_shields);

        let target = app
            .world_mut()
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                KineticBarrier {
                    power: 10.0,
                    max_power: 50.0,
                    velocity_threshold: 10.0,
                },
            ))
            .id();

        app.world_mut().send_event(DamageEvent {
            target,
            amount: 25.0,
            velocity: 50.0, // Fast
        });

        app.update();

        let health = app.world().get::<Health>(target).unwrap();
        let barrier = app.world().get::<KineticBarrier>(target).unwrap();

        assert_eq!(barrier.power, 0.0, "Barrier should be completely depleted");
        assert_eq!(
            health.current, 85.0,
            "Overflow damage (15.0) should be applied to health"
        );
    }
}
