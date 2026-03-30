# Airlocks & Pressure

## 1. Overview
The air outside is poison. The air inside is life. This feature introduces a pressure and atmosphere system to Layer 1. Buildings must be sealed to retain breathable atmosphere and heat. Standard doors release a puff of atmosphere when opened, while "Airlocks" minimize this loss but slow down Pop movement. Breaches or unsealed buildings vent atmosphere into the void, causing Pops to suffocate.

## 2. Dependencies
- `001-architecture-setup` (ECS Core)
- `002-basic-map` (Grid System)
- `004-basic-building` (Building and Walls)
- `034-pop-health` (Health and Suffocation Damage)
- `063-atmospheric-simulation` (AtmosphereGrid and Gases)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_door_vents_atmosphere_on_open() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let door_entity = app.world.spawn((
            Door { is_open: false, is_airlock: false },
            Position { x: 10, y: 10 },
        )).id();

        app.world.insert_resource(AtmosphereGrid::new(20, 20));
        // Set interior pressure high
        app.world.resource_mut::<AtmosphereGrid>().set_pressure(9, 10, 100.0);
        // Set exterior pressure low
        app.world.resource_mut::<AtmosphereGrid>().set_pressure(11, 10, 0.0);

        // Act: Open door
        app.world.entity_mut(door_entity).get_mut::<Door>().unwrap().is_open = true;

        app.add_systems(Update, process_door_venting_system);
        app.update();

        // Assert: Atmosphere vented
        let grid = app.world.resource::<AtmosphereGrid>();
        assert!(grid.get_pressure(9, 10) < 100.0, "Interior pressure should drop");
        assert!(grid.get_pressure(11, 10) > 0.0, "Exterior pressure should rise");
    }

    #[test]
    fn test_airlock_minimizes_venting() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let airlock_entity = app.world.spawn((
            Door { is_open: false, is_airlock: true },
            Position { x: 10, y: 10 },
        )).id();

        app.world.insert_resource(AtmosphereGrid::new(20, 20));
        app.world.resource_mut::<AtmosphereGrid>().set_pressure(9, 10, 100.0);
        app.world.resource_mut::<AtmosphereGrid>().set_pressure(11, 10, 0.0);

        // Act: Open airlock
        app.world.entity_mut(airlock_entity).get_mut::<Door>().unwrap().is_open = true;

        app.add_systems(Update, process_door_venting_system);
        app.update();

        // Assert: Atmosphere vented minimally compared to standard door
        let grid = app.world.resource::<AtmosphereGrid>();
        assert!(grid.get_pressure(9, 10) > 95.0, "Airlock should preserve most interior pressure");
        assert!(grid.get_pressure(11, 10) < 5.0, "Airlock should leak minimal pressure");
    }

    #[test]
    fn test_airlock_slows_movement() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let airlock_entity = app.world.spawn((
            Door { is_open: false, is_airlock: true },
            Position { x: 10, y: 10 },
        )).id();

        let pop_entity = app.world.spawn((
            Pop,
            Position { x: 10, y: 10 },
            MovementStats { base_speed: 1.0, current_speed: 1.0 },
        )).id();

        app.add_systems(Update, apply_door_movement_penalties_system);
        app.update();

        // Assert: Movement speed reduced while on airlock tile
        let stats = app.world.get::<MovementStats>(pop_entity).unwrap();
        assert!(stats.current_speed < stats.base_speed, "Airlock should slow movement");
    }

    #[test]
    fn test_unsealed_room_suffocates_pops() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.world.insert_resource(AtmosphereGrid::new(20, 20));
        // Set pressure to 0
        app.world.resource_mut::<AtmosphereGrid>().set_pressure(5, 5, 0.0);

        let pop_entity = app.world.spawn((
            Pop,
            Position { x: 5, y: 5 },
            Health { current: 100.0, max: 100.0 },
            Needs { oxygen: 100.0, ..Default::default() },
        )).id();

        app.add_systems(Update, (process_suffocation_system, apply_health_damage_system).chain());
        app.update();

        // Assert: Pop takes damage from lack of oxygen
        let health = app.world.get::<Health>(pop_entity).unwrap();
        assert!(health.current < 100.0, "Pop should take suffocation damage in vacuum");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Door {
    pub is_open: bool,
    pub is_airlock: bool,
}

// System to handle atmospheric venting when doors are open
pub fn process_door_venting_system(
    mut grid: ResMut<AtmosphereGrid>,
    query: Query<(&Door, &Position)>,
) {
    for (door, pos) in query.iter() {
        if door.is_open {
            let vent_rate = if door.is_airlock { 0.05 } else { 0.5 };

            // Simplified diffusion between adjacent tiles
            let p1 = grid.get_pressure(pos.x - 1, pos.y);
            let p2 = grid.get_pressure(pos.x + 1, pos.y);

            let diff = (p1 - p2) * vent_rate;

            grid.set_pressure(pos.x - 1, pos.y, p1 - diff);
            grid.set_pressure(pos.x + 1, pos.y, p2 + diff);
        }
    }
}

// System to slow down pops moving through airlocks
pub fn apply_door_movement_penalties_system(
    mut pop_query: Query<(&Position, &mut MovementStats), With<Pop>>,
    door_query: Query<(&Position, &Door)>,
) {
    for (pop_pos, mut stats) in pop_query.iter_mut() {
        stats.current_speed = stats.base_speed; // Reset
        for (door_pos, door) in door_query.iter() {
            if pop_pos.x == door_pos.x && pop_pos.y == door_pos.y && door.is_airlock {
                stats.current_speed *= 0.5; // Slow down
            }
        }
    }
}

// System to apply suffocation damage
pub fn process_suffocation_system(
    grid: Res<AtmosphereGrid>,
    mut query: Query<(&Position, &mut Health, &mut Needs), With<Pop>>,
) {
    for (pos, mut health, mut needs) in query.iter_mut() {
        let pressure = grid.get_pressure(pos.x, pos.y);
        if pressure < 20.0 { // Arbitrary breathable threshold
            needs.oxygen -= 10.0;
            if needs.oxygen <= 0.0 {
                health.current -= 5.0; // Damage
            }
        } else {
            needs.oxygen = 100.0; // Recover oxygen
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor 1:** `process_door_venting_system` currently only diffuses along the X-axis for simplicity. It needs to check all 4 cardinal directions and diffuse based on the pressure gradients.
- **Refactor 2:** `apply_door_movement_penalties_system` uses an O(N*M) nested loop. We should use a `HashMap` of Door positions or add a `CurrentTile` component to the Pop to make lookup O(1).
- **Refactor 3:** Suffocation damage should likely trigger a `NeedsUnmetEvent` and potentially a `ChronicleEvent` if a Pop dies from it.

## 6. Acceptance Criteria
- [ ] `cargo test` passes all tests defined in the RED phase.
- [ ] `cargo clippy -- -D warnings` returns 0 warnings.
- [ ] Test coverage hits at least 85% for `src/layer1/pressure.rs`.
- [ ] Standard doors vent a significant amount of atmosphere when open.
- [ ] Airlocks vent less than 10% of the atmosphere compared to standard doors.
- [ ] Pops moving through an airlock suffer a minimum 50% movement speed penalty.
- [ ] Pops in tiles with low atmosphere pressure take suffocation damage over time.

## 7. Technical Guidance
- Ensure `AtmosphereGrid` operations are efficient, as pressure diffusion will run every tick.
- Coordinate with the `movement_system` to ensure the airlock speed penalty properly delays the Pop's tile transition.
- Use `is_open` state transitions driven by the Utility AI when a Pop attempts to pass through the door.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
- **Builder:** The spec specifies `AtmosphereGrid` for pressure and a `Door` component, however, the codebase already has a `PressureGrid` in `src/layer1/pressure.rs` that manages pressure and vacuum, and a `DoorControl` component in `src/layer1/control.rs` managing door states. There's also an `AtmosphereGrid` in `src/layer1/nature/atmosphere.rs` which is actually for pollution. Following the spec literally would duplicate logic and mix pollution with pressure. Should I implement this using the existing `PressureGrid` and `DoorControl` instead? Because the spec is contradictory with the existing state of the project.
