# 1356 - Echo Casters

**1. Overview**
Instantaneous travel that leaves pieces of your soul behind. Instead of slow colony ships, you can build "Echo Casters" to instantly transmit the minds of your Pops to distant worlds where "Sleeper Shells" await. However, transmission errors occasionally cause fragments of the original mind to remain behind, creating "Echo Pops" that have partial memories and unstable moods.

**2. Dependencies**
- `Pop` entities.
- A system for traveling or moving pops between colonies (Layer 2).

**3. RED Phase: Tests First**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_echo_caster_transmission() {
        // Arrange
        let mut world = World::new();
        let source_colony = world.spawn(Colony).id();
        let target_colony = world.spawn(Colony).id();

        // Add an EchoCaster to the source colony
        world.entity_mut(source_colony).insert(EchoCaster { error_rate: 0.1 });

        // Spawn a pop in the source colony
        let pop = world.spawn((PopBundle::default(), Location(source_colony))).id();

        // Act
        // Transmit the pop
        world.send_event(TransmitPopEvent { pop, target: target_colony });

        let mut schedule = Schedule::default();
        schedule.add_systems(echo_transmission_system);
        schedule.run(&mut world);

        // Assert
        // The pop's location should be updated.
        let new_loc = world.get::<Location>(pop).unwrap();
        assert_eq!(new_loc.0, target_colony);
    }

    #[test]
    fn test_echo_caster_creates_echo_pop_on_error() {
        // Arrange
        let mut world = World::new();
        let source_colony = world.spawn(Colony).id();
        let target_colony = world.spawn(Colony).id();

        // 100% error rate to guarantee an echo
        world.entity_mut(source_colony).insert(EchoCaster { error_rate: 1.0 });

        let pop = world.spawn((PopBundle::default(), Location(source_colony))).id();
        world.insert_resource(Events::<TransmitPopEvent>::default());
        world.send_event(TransmitPopEvent { pop, target: target_colony });

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(echo_transmission_system);
        schedule.run(&mut world);

        // Assert
        // The original pop moved
        assert_eq!(world.get::<Location>(pop).unwrap().0, target_colony);

        // An Echo pop was created at the source
        let mut echo_query = world.query_filtered::<&Location, With<EchoPop>>();
        let echos: Vec<_> = echo_query.iter(&world).collect();
        assert_eq!(echos.len(), 1);
        assert_eq!(echos[0].0, source_colony);
    }
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
use bevy_ecs::prelude::*;
use bevy_ecs::event::Events;

#[derive(Component)]
pub struct Colony;

#[derive(Component)]
pub struct Location(pub Entity);

#[derive(Component, Default)]
pub struct PopBundle;

#[derive(Component)]
pub struct EchoCaster {
    pub error_rate: f32, // 0.0 to 1.0
}

#[derive(Component)]
pub struct EchoPop;

#[derive(Event)]
pub struct TransmitPopEvent {
    pub pop: Entity,
    pub target: Entity,
}

pub fn echo_transmission_system(
    mut commands: Commands,
    mut events: EventReader<TransmitPopEvent>,
    mut pop_query: Query<&mut Location>, // Mutably get the location
    caster_query: Query<&EchoCaster>,
) {
    for event in events.read() {
        if let Ok(mut loc) = pop_query.get_mut(event.pop) {
            let source = loc.0;

            // Move the pop
            loc.0 = event.target;

            // Check for transmission error
            if let Ok(caster) = caster_query.get(source) {
                // Simplified random check for testability (assume it hits if error_rate > 0.5)
                // In a real implementation, use a seeded RNG or passing an RNG resource.
                if caster.error_rate > 0.5 {
                    commands.spawn((
                        PopBundle::default(),
                        Location(source),
                        EchoPop,
                    ));
                }
            }
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- The random check in `echo_transmission_system` needs a proper RNG injected as a resource to allow deterministic testing.
- `EchoPop` could contain a reference to the original `Entity` or copy over specific memories to make the narrative impact stronger.
- Add an event `EchoFragmentCreatedEvent` for Chronicle integration.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified (Pops move, echoes sometimes spawn)

**7. Technical Guidance**
- When copying memories to the `EchoPop`, ensure you don't shallow copy entities that might be despawned.
- Ensure the `EchoPop` has modifiers to its needs or behavior (e.g., higher stress, lower stability) to reflect its fragmented nature.

**8. Questions**
*Builder: add questions here if spec is unclear.*
