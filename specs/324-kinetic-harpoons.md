# 324: Kinetic Harpoons

## 1. Overview

You can launch massive "Harpoon Tethers" from Layer 1 to snag passing Comets or small Asteroids in Layer 2, forcefully pulling them down to the surface for immediate, massive resource extraction. Pulling them down causes a localized impact event (damage) on the landing tile. This offers high-risk, immediate massive resource injection versus the catastrophic danger of orbital mechanics failures.

## 2. Dependencies

- `018` Mining Resources (for resource nodes)
- `149` Cometary Injection (for the target celestial bodies)
- `184` Orbital Debris (for impact mechanics)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_harpoon_launch_snags_target() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_harpoon_launch);

        let comet = app.world_mut().spawn(CelestialBody {
            mass: 500,
            resource_yield: 1000,
            status: BodyStatus::Orbiting,
        }).id();

        let launcher = app.world_mut().spawn((
            GridPosition { x: 10, y: 10 },
            HarpoonLauncher { ready: true }
        )).id();

        app.world_mut().insert_resource(LaunchEvent { launcher, target: comet });

        // Act
        app.update();

        // Assert
        let body = app.world().get::<CelestialBody>(comet).unwrap();
        assert_eq!(body.status, BodyStatus::Snagged, "Comet should be marked as snagged");

        let launcher_state = app.world().get::<HarpoonLauncher>(launcher).unwrap();
        assert!(!launcher_state.ready, "Launcher should not be ready after firing");
    }

    #[test]
    fn test_snagged_body_impacts_surface() {
        // Arrange
        let mut app = App::new();
        app.add_event::<ImpactEvent>();
        app.add_systems(Update, process_harpoon_winch);

        let comet = app.world_mut().spawn(CelestialBody {
            mass: 500,
            resource_yield: 1000,
            status: BodyStatus::Snagged,
        }).id();

        let launcher = app.world_mut().spawn((
            GridPosition { x: 10, y: 10 },
            WinchSystem { target: Some(comet), strength: 1000, progress: 99.0 } // Almost landed
        )).id();

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<ImpactEvent>>();
        let mut reader = events.get_reader();
        let mut impact_occurred = false;

        for ev in reader.read(events) {
            if ev.x == 10 && ev.y == 10 {
                impact_occurred = true;
                assert_eq!(ev.yield_amount, 1000, "Impact should deliver the resource yield");
            }
        }

        assert!(impact_occurred, "Snagged body reaching 100 progress should cause an impact event");
    }

    #[test]
    fn test_winch_failure_causes_catastrophe() {
        // Arrange
        let mut app = App::new();
        app.add_event::<ImpactEvent>();
        app.add_systems(Update, process_harpoon_winch);

        let massive_comet = app.world_mut().spawn(CelestialBody {
            mass: 5000, // Too heavy for winch
            resource_yield: 1000,
            status: BodyStatus::Snagged,
        }).id();

        let launcher = app.world_mut().spawn((
            GridPosition { x: 10, y: 10 },
            WinchSystem { target: Some(massive_comet), strength: 1000, progress: 50.0 }
        )).id();

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<ImpactEvent>>();
        let mut reader = events.get_reader();
        let mut catastrophic_impact = false;

        for ev in reader.read(events) {
            if ev.x == 10 && ev.y == 10 {
                catastrophic_impact = true;
                assert!(ev.damage_radius > 0, "Winch failure should cause a damaging impact radius");
                assert_eq!(ev.yield_amount, 0, "Winch failure destroys the resource yield");
            }
        }

        assert!(catastrophic_impact, "Massive body exceeding winch strength should cause failure");

        // Ensure launcher is destroyed or broken
        assert!(app.world().get::<WinchSystem>(launcher).is_none() || app.world().get::<WinchSystem>(launcher).unwrap().target.is_none(), "Winch target should be cleared on failure");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(PartialEq, Debug)]
pub enum BodyStatus {
    Orbiting,
    Snagged,
}

#[derive(Component)]
pub struct CelestialBody {
    pub mass: u32,
    pub resource_yield: u32,
    pub status: BodyStatus,
}

#[derive(Component)]
pub struct HarpoonLauncher {
    pub ready: bool,
}

#[derive(Resource)]
pub struct LaunchEvent {
    pub launcher: Entity,
    pub target: Entity,
}

#[derive(Component)]
pub struct WinchSystem {
    pub target: Option<Entity>,
    pub strength: u32,
    pub progress: f32, // 0.0 (orbit) to 100.0 (ground)
}

#[derive(Event)]
pub struct ImpactEvent {
    pub x: i32,
    pub y: i32,
    pub yield_amount: u32,
    pub damage_radius: u32,
}

pub fn process_harpoon_launch(
    mut commands: Commands,
    mut launch_event: Option<ResMut<LaunchEvent>>,
    mut launchers: Query<&mut HarpoonLauncher>,
    mut bodies: Query<&mut CelestialBody>,
) {
    if let Some(event) = launch_event.take() {
        if let Ok(mut launcher) = launchers.get_mut(event.launcher) {
            if launcher.ready {
                if let Ok(mut body) = bodies.get_mut(event.target) {
                    body.status = BodyStatus::Snagged;
                    launcher.ready = false;

                    // Attach winch (in reality, requires a command to add component safely)
                    commands.entity(event.launcher).insert(WinchSystem {
                        target: Some(event.target),
                        strength: 2000,
                        progress: 0.0,
                    });
                }
            }
        }
    }
}

pub fn process_harpoon_winch(
    mut events: EventWriter<ImpactEvent>,
    mut launchers: Query<(Entity, &GridPosition, &mut WinchSystem)>,
    bodies: Query<&CelestialBody>,
    mut commands: Commands,
) {
    for (entity, pos, mut winch) in launchers.iter_mut() {
        if let Some(target_entity) = winch.target {
            if let Ok(body) = bodies.get(target_entity) {
                if body.mass > winch.strength {
                    // Catastrophic failure
                    events.send(ImpactEvent {
                        x: pos.x,
                        y: pos.y,
                        yield_amount: 0, // Vaporized
                        damage_radius: 5, // Huge crater
                    });
                    winch.target = None;
                    commands.entity(target_entity).despawn();
                } else {
                    // Successful reeling
                    winch.progress += 1.0;
                    if winch.progress >= 100.0 {
                        events.send(ImpactEvent {
                            x: pos.x,
                            y: pos.y,
                            yield_amount: body.resource_yield,
                            damage_radius: 1, // Small localized damage for successful catch
                        });
                        winch.target = None;
                        commands.entity(target_entity).despawn();
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Energy Consumption:** The `WinchSystem` should consume massive amounts of `Energy` from the grid per tick. If power fails during the reel-in phase, the progress stalls or instantly fails.
- **Impact System Integration:** Connect `ImpactEvent` to the actual `TerrainGrid` to destroy buildings and convert tiles into "Crater" or "Resource Node" tiles based on the `yield_amount`.
- **Layer 2 Interaction:** When a body is snagged, it should be removed from the abstract System view pathing or at least immobilized on the Layer 2 map.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] Test coverage >=85% for new code.
- [ ] Launcher snags Comet, changing its status.
- [ ] Winch reels in body over time (progress increments).
- [ ] Safe arrival spawns resources and minor damage.
- [ ] Overweight body snaps winch, causing major damage and zero resources.

## 7. Technical Guidance

- Implement inside `src/layer1/tech/harpoon.rs`.
- Ensure the `ImpactEvent` listener is the same one used by `063-orbital-debris.md` (Orbital Debris) to reuse explosion and terrain damage logic.
- Add a new "Targeting" UI state when the player clicks the Harpoon building, switching to a Layer 2 overlay to pick the comet.

## 8. Questions

*Builder: add questions here if spec is unclear.*
