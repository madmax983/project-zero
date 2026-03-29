# 743 - The Gravity Loom

## 1. Overview
Weaving starship armor out of collapsed matter, one micro-singularity at a time. High-end Layer 2 orbital stations can use "Gravity Looms" to compress Layer 1 resources into hyper-dense "Singularity Plating." The loom requires a continuous, massive power beam from the Layer 1 colony. If the power beam fluctuates by even 1%, the loom drops the micro-singularity onto the planet.

## 2. Dependencies
- Layer 1 Power grid and transmission systems.
- Layer 2 Station crafting mechanics.
- Cross-layer event bridging (Power fluctuation -> Orbital drop).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_power_fluctuation_triggers_singularity_drop() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<PowerFluctuationEvent>();
        app.add_event::<SingularityDropEvent>();
        app.add_systems(Update, handle_gravity_loom_power_system);

        app.world_mut().spawn(GravityLoom {
            is_active: true,
            required_power: 1000.0,
            current_power_received: 1000.0,
            target_colony_entity: Entity::from_raw(1),
        });

        // Act - Simulate a 2% drop in power
        app.world_mut().send_event(PowerFluctuationEvent {
            transmitter_entity: Entity::from_raw(1),
            new_power_level: 980.0, // < 99% of required 1000
        });
        app.update();

        // Assert
        let drop_events = app.world().resource::<Events<SingularityDropEvent>>();
        assert_eq!(drop_events.len(), 1, "SingularityDropEvent should trigger when power fluctuates > 1%");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct GravityLoom {
    pub is_active: bool,
    pub required_power: f32,
    pub current_power_received: f32,
    pub target_colony_entity: Entity,
}

#[derive(Event)]
pub struct PowerFluctuationEvent {
    pub transmitter_entity: Entity,
    pub new_power_level: f32,
}

#[derive(Event)]
pub struct SingularityDropEvent {
    pub target_colony_entity: Entity,
}

pub fn handle_gravity_loom_power_system(
    mut query: Query<&mut GravityLoom>,
    mut fluctuation_events: EventReader<PowerFluctuationEvent>,
    mut drop_events: EventWriter<SingularityDropEvent>,
) {
    for event in fluctuation_events.read() {
        for mut loom in query.iter_mut() {
            if loom.is_active && loom.target_colony_entity == event.transmitter_entity {
                let fluctuation_percentage = ((loom.required_power - event.new_power_level).abs() / loom.required_power) * 100.0;

                if fluctuation_percentage > 1.0 {
                    drop_events.send(SingularityDropEvent {
                        target_colony_entity: loom.target_colony_entity,
                    });
                    loom.is_active = false; // Disable loom after catastrophic failure
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create a handler for `SingularityDropEvent` in the Layer 1 systems that calculates impact damage (destroying buildings/terrain based on a blast radius).
- Smooth out power transmission logic so minor tick-level jitter doesn't instantly cause a failure; consider a short buffer or threshold over time.

## 6. Acceptance Criteria (Testable!)
- [ ] RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] A >1% power drop triggers the `SingularityDropEvent`.

## 7. Technical Guidance
- The connection between Layer 1 transmitter and Layer 2 Loom must be explicitly modeled (e.g., via a `PowerBeam` component bridging the entities).

## 8. Questions
*Builder: add questions here if spec is unclear.*
