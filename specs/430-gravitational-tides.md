# Spec 430: Gravitational Tides

## 1. Overview
The planet is being pulled and squeezed by a massive neighbor, creating cyclical hazards. High Gravity periods slow movement and increase energy consumption. Low Gravity periods allow fast movement but increase the risk of pops taking fall damage or items floating away.

## 2. Dependencies
- `012-input-architecture`
- `042-energy-system`
- `065-day-night-cycle`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_high_gravity_slows_movement() {
        let mut app = App::new();
        app.insert_resource(GravitationalTide { state: TideState::High });
        app.add_systems(Update, apply_gravity_modifiers_system);

        let entity = app.world_mut().spawn((
            MovementSpeed { base: 10.0, current: 10.0 },
        )).id();

        app.update();

        let speed = app.world().get::<MovementSpeed>(entity).unwrap();
        assert_eq!(speed.current, 5.0); // High gravity halves speed
    }

    #[test]
    fn test_low_gravity_increases_movement_and_risk() {
        let mut app = App::new();
        app.insert_resource(GravitationalTide { state: TideState::Low });
        app.add_event::<FallDamageRiskEvent>();
        app.add_systems(Update, apply_gravity_modifiers_system);

        let entity = app.world_mut().spawn((
            MovementSpeed { base: 10.0, current: 10.0 },
            Position { value: Vec3::new(0.0, 10.0, 0.0) }, // Elevated position
        )).id();

        app.update();

        let speed = app.world().get::<MovementSpeed>(entity).unwrap();
        assert_eq!(speed.current, 15.0); // Low gravity increases speed

        let risk_events = app.world().resource::<Events<FallDamageRiskEvent>>();
        let mut reader = risk_events.get_reader();
        let events: Vec<_> = reader.read(risk_events).collect();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].entity, entity);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct GravitationalTide {
    pub state: TideState,
}

#[derive(PartialEq, Debug)]
pub enum TideState {
    High,
    Normal,
    Low,
}

#[derive(Component)]
pub struct MovementSpeed {
    pub base: f32,
    pub current: f32,
}

#[derive(Component)]
pub struct Position {
    pub value: Vec3,
}

#[derive(Event)]
pub struct FallDamageRiskEvent {
    pub entity: Entity,
}

pub fn apply_gravity_modifiers_system(
    tide: Res<GravitationalTide>,
    mut q_movement: Query<(Entity, &mut MovementSpeed, Option<&Position>)>,
    mut fall_risk_events: EventWriter<FallDamageRiskEvent>,
) {
    for (entity, mut speed, pos) in q_movement.iter_mut() {
        match tide.state {
            TideState::High => {
                speed.current = speed.base * 0.5;
            }
            TideState::Normal => {
                speed.current = speed.base;
            }
            TideState::Low => {
                speed.current = speed.base * 1.5;
                if let Some(pos) = pos {
                    if pos.value.y > 5.0 {
                        fall_risk_events.send(FallDamageRiskEvent { entity });
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate tide changes based on actual orbital positions from Layer 2.
- Have High Gravity also increase energy consumption for automated machines.
- Add `Unsealed` component to items, and in Low Gravity, have a chance for unsealed items to float away.

## 6. Acceptance Criteria
- [ ] `GravitationalTide` resource tracks the current tide state.
- [ ] High Tide reduces `MovementSpeed` by 50%.
- [ ] Low Tide increases `MovementSpeed` by 50% and emits `FallDamageRiskEvent` for elevated pops.
- [ ] Tests pass with >= 85% coverage.

## 7. Technical Guidance
- Integrate with Layer 2 orbital mechanics so that tides happen predictably based on planetary rotation and moon/gas giant positions.
- Make sure energy modifiers for High Gravity apply to `EnergyConsumer` entities.

## 8. Questions
- Should Low Gravity provide a Morale buff for pops who enjoy floating?
- *Architect:* Yes, Pops with the "Zero-G Native" or similar trait should get a small Morale buff, otherwise it is strictly neutral for Morale.
