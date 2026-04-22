# Magnetic Reversal

## 1. Overview
This feature implements periodic "Pole Flip" events. During these events, the planetary magnetic field collapses, surface radiation spikes, compass/map navigation is scrambled, and migratory fauna lose their way.

## 2. Dependencies
- Pathfinding / Navigation systems.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_magnetic_reversal_starts() {
        let mut world = World::new();
        // Setup world...
        world.insert_resource(MagneticField { active: true, radiation_level: 0.0 });

        // Trigger event
        world.send_event(PoleFlipEvent);
        world.run_system_once(handle_pole_flip_system);

        let field = world.resource::<MagneticField>();
        assert!(!field.active);
        assert!(field.radiation_level > 0.0);
    }

    #[test]
    fn test_magnetic_reversal_scrambles_navigation() {
        let mut world = World::new();
        // Setup entity with navigation component
        let entity = world.spawn(NavigationComponent { heading: 90.0, target: Vec2::new(10.0, 10.0) }).id();
        world.insert_resource(MagneticField { active: false, radiation_level: 5.0 });

        world.run_system_once(navigation_scramble_system);

        let nav = world.get::<NavigationComponent>(entity).unwrap();
        assert_ne!(nav.heading, 90.0); // Heading should be scrambled
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
pub struct MagneticField {
    pub active: bool,
    pub radiation_level: f32,
}

pub struct PoleFlipEvent;

pub fn handle_pole_flip_system(
    mut events: EventReader<PoleFlipEvent>,
    mut field: ResMut<MagneticField>,
) {
    for _ in events.read() {
        field.active = false;
        field.radiation_level = 10.0; // Arbitrary spike
    }
}

#[derive(Component)]
pub struct NavigationComponent {
    pub heading: f32,
    pub target: Vec2,
}

pub fn navigation_scramble_system(
    field: Res<MagneticField>,
    mut query: Query<&mut NavigationComponent>,
) {
    if !field.active {
        for mut nav in query.iter_mut() {
            // Minimal scramble logic, e.g., invert or randomize. For GREEN, just invert.
            nav.heading = -nav.heading;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoded radiation spike value (10.0).
- **Improvements**: Make the radiation spike configurable. Better integration with existing pathfinding algorithms. Add a system to restore the magnetic field after a duration.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pole flip successfully toggles magnetic field and spikes radiation.
- [ ] Navigation is scrambled when magnetic field is down.

## 7. Technical Guidance
- Integrate with the simulation loop to manage the duration of the pole flip.
- Ensure radiation affects related gameplay systems based on design.

## 8. Questions
*Builder: add questions here if spec is unclear.*
