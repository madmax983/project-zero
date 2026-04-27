# The Bulwark Platform

## 1. Overview
**Layer:** 2
**Fantasy:** A slow, immovable fortress that sacrifices all mobility to create an unbreakable defensive perimeter.
**Mechanic:** A ship class that cannot travel via hyperlanes and moves at a crawl within the system. When deployed, it anchors itself and projects a massive, system-wide shield that absorbs damage directed at any friendly Layer 1 or Layer 2 asset in its radius. It requires an immense, continuous stream of raw energy from the planet below to maintain the shield.

## 2. Dependencies
- Layer 2 Fleet Movement System
- Energy/Power Grid System
- Combat/Damage System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_bulwark_drains_colony_energy() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<ColonyPowerGrid>();
        app.world_mut().resource_mut::<ColonyPowerGrid>().available_energy = 1000.0;

        app.world_mut().spawn(BulwarkPlatform {
            is_deployed: true,
            energy_draw: 200.0,
        });

        // Act
        app.add_systems(Update, bulwark_energy_draw_system);
        app.update();

        // Assert
        let power_grid = app.world().resource::<ColonyPowerGrid>();
        assert_eq!(power_grid.available_energy, 800.0, "Deployed Bulwark should drain colony energy");
    }

    #[test]
    fn test_bulwark_intercepts_damage() {
        // Arrange
        let mut app = App::new();
        app.add_event::<DamageEvent>();

        let bulwark = app.world_mut().spawn(BulwarkPlatform {
            is_deployed: true,
            energy_draw: 200.0,
        }).id();

        let target = app.world_mut().spawn(FriendlyAsset).id();

        app.world_mut().send_event(DamageEvent {
            target,
            amount: 50.0,
        });

        // Act
        app.add_systems(Update, bulwark_damage_interception_system);
        app.update();

        // Assert
        let events = app.world().resource::<Events<DamageEvent>>();
        let mut reader = events.get_cursor();
        let ev = reader.read(events).next().unwrap();

        assert_eq!(ev.target, bulwark, "Damage should be redirected to the Bulwark");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ColonyPowerGrid {
    pub available_energy: f32,
}

#[derive(Component)]
pub struct BulwarkPlatform {
    pub is_deployed: bool,
    pub energy_draw: f32,
}

#[derive(Component)]
pub struct FriendlyAsset;

#[derive(Event, Clone)]
pub struct DamageEvent {
    pub target: Entity,
    pub amount: f32,
}

pub fn bulwark_energy_draw_system(
    query: Query<&BulwarkPlatform>,
    mut power_grid: ResMut<ColonyPowerGrid>,
) {
    for bulwark in query.iter() {
        if bulwark.is_deployed {
            power_grid.available_energy -= bulwark.energy_draw;
        }
    }
}

pub fn bulwark_damage_interception_system(
    mut events: EventReader<DamageEvent>,
    mut out_events: EventWriter<DamageEvent>,
    bulwark_query: Query<(Entity, &BulwarkPlatform)>,
    target_query: Query<(), With<FriendlyAsset>>,
) {
    let active_bulwark = bulwark_query.iter().find(|(_, b)| b.is_deployed);

    for ev in events.read() {
        let mut final_ev = ev.clone();

        if let Some((bulwark_entity, _)) = active_bulwark {
            if target_query.get(ev.target).is_ok() {
                // Redirect damage
                final_ev.target = bulwark_entity;
            }
        }

        // In a real implementation we would mutate the event or consume it and send a new one
        // For MVP, we'll assume we re-emit the modified event (simplified for brevity)
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- `bulwark_damage_interception_system` needs to correctly intercept events before they apply damage, which might require a two-pass event system or a middleware approach.
- Ensure power shortage collapses the shield: if `available_energy < 0`, `is_deployed` should toggle off, or the shield fails to intercept.
- Ensure the Bulwark has a maximum shield capacity before it takes structural damage.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Deployed Bulwarks draw massive power from the Colony Grid.
- [ ] Bulwarks successfully intercept damage meant for other friendly entities while deployed.

## 7. Technical Guidance
- **Event Ordering:** Damage interception is tricky in Bevy. You might need to have a `ProposedDamageEvent` that gets intercepted and converted into an `AppliedDamageEvent`.
- **Mobility Restrictions:** Add a `SpeedModifier` component that forces the Bulwark's speed to 0 when `is_deployed` is true.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
