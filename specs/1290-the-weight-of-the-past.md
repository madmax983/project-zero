# 1290: The Weight of the Past

## 1. Overview
**Layer:** 1

**Fantasy:** A society literally crushed by its own history.

**Mechanic:** Every time a Pop dies, their digital engrams are automatically uploaded to the colony's central "Ancestral Server." As the population grows and dies over generations, the server requires exponentially more power and cooling to maintain the memories.

## 2. Dependencies
- Base Layer 1 Population System
- Death Event System
- Grid / Power System
- Grid / Cooling System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // RED Phase Test Setup
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<PopDeathEvent>();
        app.add_systems(Update, (
            handle_engram_upload_system,
            scale_server_power_demand_system,
        ));
        app
    }

    #[test]
    fn test_pop_death_uploads_engram() {
        let mut app = setup_app();

        let server = app.world_mut().spawn(AncestralServer { stored_engrams: 0 }).id();
        let pop = app.world_mut().spawn(Pop).id();

        app.world_mut().send_event(PopDeathEvent { pop_entity: pop });
        app.update();

        let server_data = app.world().get::<AncestralServer>(server).unwrap();
        assert_eq!(server_data.stored_engrams, 1, "Pop death should increase stored engrams on the server");
    }

    #[test]
    fn test_engram_count_increases_power_demand() {
        let mut app = setup_app();

        let server = app.world_mut().spawn((
            AncestralServer { stored_engrams: 100 },
            PowerConsumer { current_demand: 10.0 },
        )).id();

        app.update();

        let consumer = app.world().get::<PowerConsumer>(server).unwrap();
        assert!(consumer.current_demand > 10.0, "High engram count should scale the power demand exponentially");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Event)]
pub struct PopDeathEvent {
    pub pop_entity: Entity,
}

#[derive(Component)]
pub struct AncestralServer {
    pub stored_engrams: u32,
}

#[derive(Component)]
pub struct PowerConsumer {
    pub current_demand: f32,
}

pub fn handle_engram_upload_system(
    mut events: EventReader<PopDeathEvent>,
    mut servers: Query<&mut AncestralServer>,
) {
    for _event in events.read() {
        // Assume one server for minimal implementation
        if let Some(mut server) = servers.iter_mut().next() {
            server.stored_engrams += 1;
        }
    }
}

pub fn scale_server_power_demand_system(
    mut servers: Query<(&AncestralServer, &mut PowerConsumer)>,
) {
    for (server, mut consumer) in servers.iter_mut() {
        // Base demand is 10, each engram adds 0.1, scaling quadratically for "exponential" feel
        let engrams_f32 = server.stored_engrams as f32;
        consumer.current_demand = 10.0 + (engrams_f32 * 0.1) + (engrams_f32 * engrams_f32 * 0.001);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Multiple Servers**: The current logic blindly grabs the `.next()` server. It should probably target a specific global resource or ensure it routes to the correct colony's server.
- **Cooling Demand**: The spec mentions cooling. A secondary system `scale_server_cooling_demand_system` should be added hooking into the `ThermalGrid` or `HeatGenerator` component.
- **Manual Deletion**: The design mentions manually deleting engrams. A future system/event `PurgeEngramsEvent` will be needed to reduce `stored_engrams`, likely causing a massive unrest spike.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- **Grid Integration**: Ensure `PowerConsumer` is processed before the grid resolves its tick, otherwise the server won't correctly pull the new power requirement.
- **Save/Load**: `stored_engrams` will grow continuously throughout a playthrough. Ensure it's part of the standard save state.

## 8. Questions
*Builder: add questions here if spec is unclear.*
