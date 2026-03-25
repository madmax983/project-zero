# Orbital Debt Collections

**1. Overview**
If a colony falls deep into debt with a Layer 3 megacorp, they don't just send angry messages. They park a Layer 2 "Collection Cruiser" in orbit. This cruiser periodically drops specialized, heavily armored "Repo Drones" directly onto Layer 1. These drones don't attack; they simply dismantle high-value buildings, pack the materials, and rocket back to orbit until the debt is paid.

**2. Dependencies**
- `layer3::economy::Debt`
- `layer1::buildings::Building`
- `layer1::units::Drone`

**3. RED Phase: Tests First**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_drone_spawns_on_high_debt() {
        let mut app = App::new();
        app.add_systems(Update, debt_collection_system);

        app.insert_resource(ColonyDebt { amount: 10000.0, threshold: 5000.0 });

        app.update();

        let drone_count = app.world_mut().query::<&RepoDrone>().iter(app.world()).count();
        assert!(drone_count > 0, "High debt should trigger the spawning of Repo Drones");
    }

    #[test]
    fn test_drone_dismantles_building() {
        let mut app = App::new();
        app.add_systems(Update, drone_dismantle_system);

        let target_building = app.world_mut().spawn((
            Building { value: 1000.0 },
            Position { x: 10, y: 10 },
        )).id();

        app.world_mut().spawn((
            RepoDrone { target: Some(target_building), dismantling_progress: 100.0 },
            Position { x: 10, y: 10 },
        ));

        app.update();

        assert!(app.world().get_entity(target_building).is_none(), "Drone should destroy the building upon completing dismantling");
    }
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct ColonyDebt {
    pub amount: f32,
    pub threshold: f32,
}

#[derive(Component)]
pub struct RepoDrone {
    pub target: Option<Entity>,
    pub dismantling_progress: f32,
}

#[derive(Component)]
pub struct Building {
    pub value: f32,
}

#[derive(Component, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub fn debt_collection_system(
    mut commands: Commands,
    debt: Option<Res<ColonyDebt>>,
) {
    if let Some(debt) = debt {
        if debt.amount > debt.threshold {
            commands.spawn((
                RepoDrone { target: None, dismantling_progress: 0.0 },
                Position { x: 0, y: 0 },
            ));
        }
    }
}

pub fn drone_dismantle_system(
    mut commands: Commands,
    mut drones: Query<&mut RepoDrone>,
) {
    for drone in drones.iter_mut() {
        if let Some(target) = drone.target {
            if drone.dismantling_progress >= 100.0 {
                if let Some(entity_commands) = commands.get_entity(target) {
                    entity_commands.despawn();
                }
            }
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Make drones actively pathfind towards the highest value buildings instead of instantly dismantling targeted ones.
- Debt should decrease proportionately when a building is successfully dismantled and "extracted".

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] `RepoDrone` entities spawn when debt threshold is exceeded.
- [ ] Drones successfully target and despawn `Building` entities.

**7. Technical Guidance**
- Integrate this with existing pathfinding or navigation meshes if drones need to physically move to the buildings.
- Broadcast an event when a building is dismantled to update the UI and subtract from the debt pool.

**8. Questions**
*Builder: add questions here if spec is unclear.*
