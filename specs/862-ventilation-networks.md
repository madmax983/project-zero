# Specification: 862 Ventilation Networks

## 1. Overview
**Layer:** 1
**Fantasy:** The unseen arteries of the station. Useful for air, dangerous for security.
**Mechanic:** "Vents" connect rooms for atmosphere equalization. However, they also allow passage for "Small" entities (Vermin, Drones, Spies) even if doors are locked. "Grates" block movement but reduce airflow.

## 2. Dependencies
- Layer 1 Pathfinding & Grid (TerrainGrid)
- AtmosphereGrid/PressureGrid (from Airlocks & Pressure)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_small_entities_can_traverse_vents() {
        let mut app = App::new();
        app.add_plugins(VentilationPlugin);

        // Grid positions
        let pos_a = UVec2::new(0, 0);
        let pos_b = UVec2::new(0, 1);

        // Vent connecting A and B
        let vent = app.world_mut().spawn(VentConnection { pos_a, pos_b, grated: false }).id();

        // Small entity
        let rat = app.world_mut().spawn((SmallEntity, PathNavigator, Position(pos_a))).id();

        // Large entity
        let human = app.world_mut().spawn((PathNavigator, Position(pos_a))).id();

        // System should allow rat to path through, but not human
        app.world_mut().send_event(PathRequest { entity: rat, start: pos_a, end: pos_b });
        app.world_mut().send_event(PathRequest { entity: human, start: pos_a, end: pos_b });
        app.update();

        let rat_path = app.world().get::<PathResult>(rat).unwrap();
        assert!(rat_path.success, "Small entities should path through open vents");

        let human_path = app.world().get::<PathResult>(human).unwrap();
        assert!(!human_path.success, "Large entities should not path through vents");
    }

    #[test]
    fn test_grates_block_movement_but_allow_some_air() {
        let mut app = App::new();
        app.add_plugins(VentilationPlugin);

        let pos_a = UVec2::new(0, 0);
        let pos_b = UVec2::new(0, 1);

        let vent = app.world_mut().spawn(VentConnection { pos_a, pos_b, grated: true }).id();

        let rat = app.world_mut().spawn((SmallEntity, PathNavigator, Position(pos_a))).id();

        app.world_mut().send_event(PathRequest { entity: rat, start: pos_a, end: pos_b });
        app.update();

        let rat_path = app.world().get::<PathResult>(rat).unwrap();
        assert!(!rat_path.success, "Small entities should not path through grated vents");

        // Verify airflow is still present but reduced
        let airflow_amount = calculate_vent_airflow(&app.world().get::<VentConnection>(vent).unwrap());
        assert!(airflow_amount > 0.0 && airflow_amount < 1.0, "Grated vent should have reduced airflow");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Position(pub UVec2);

#[derive(Component)]
pub struct VentConnection {
    pub pos_a: UVec2,
    pub pos_b: UVec2,
    pub grated: bool,
}

#[derive(Component)]
pub struct SmallEntity;

#[derive(Component)]
pub struct PathNavigator;

#[derive(Event)]
pub struct PathRequest {
    pub entity: Entity,
    pub start: UVec2,
    pub end: UVec2,
}

#[derive(Component)]
pub struct PathResult {
    pub success: bool,
}

pub struct VentilationPlugin;

impl Plugin for VentilationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PathRequest>()
           .add_systems(Update, handle_vent_pathfinding);
    }
}

fn handle_vent_pathfinding(
    mut events: EventReader<PathRequest>,
    vents: Query<&VentConnection>,
    small_entities: Query<&SmallEntity>,
    mut commands: Commands,
) {
    for event in events.read() {
        let is_small = small_entities.get(event.entity).is_ok();

        let mut success = false;
        if is_small {
            // Check if there is an ungrated vent connecting start and end
            for vent in vents.iter() {
                if !vent.grated && ((vent.pos_a == event.start && vent.pos_b == event.end) ||
                                    (vent.pos_b == event.start && vent.pos_a == event.end)) {
                    success = true;
                    break;
                }
            }
        }

        commands.entity(event.entity).insert(PathResult { success });
    }
}

pub fn calculate_vent_airflow(vent: &VentConnection) -> f32 {
    if vent.grated {
        0.5 // 50% airflow
    } else {
        1.0 // 100% airflow
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- `handle_vent_pathfinding` currently iterates over all vents for each path request. Refactor to query a spatial grid or hashmap of vents for `O(1)` adjacency lookups.
- Integrate `calculate_vent_airflow` directly into the `AtmosphereGrid` diffusion system instead of being a standalone function.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the module.
- [ ] Entities with `SmallEntity` can successfully path through `VentConnection`s where `grated` is false.
- [ ] No entities can path through a grated vent.
- [ ] `calculate_vent_airflow` returns reduced flow for grated vents.

## 7. Technical Guidance
- In a full pathfinding implementation (A*), vents act as edges in the nav graph specifically available only to agents possessing `SmallEntity`.
- Consider how toxins or pathogens will flow through these vents; they should use the airflow modifier.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
