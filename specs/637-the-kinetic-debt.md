# 637: The Kinetic Debt

## 1. Overview

The physical consequences of a galactic war arriving decades after the treaties are signed. During Layer 3 fleet battles, massive kinetic weapons (like railgun slugs) that miss their targets continue flying through space. These become "Kinetic Ghosts" tracked by the game. Decades later, these slugs can randomly enter the Layer 2 space of one of your star systems, screaming toward a Layer 1 colony at relativistic speeds with only minutes of warning. You are enjoying a golden age of peace, having won a massive war a century ago. Suddenly, a hyper-velocity slug fired by your own flagship during the final battle of that war drops out of the void and obliterates your capital city in a single, unpreventable strike.

## 2. Dependencies

- `159` Fleet Combat Resolution
- `184` Orbital Debris
- `206` Orbital Crossfire

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_kinetic_ghost_generation_on_miss() {
        // Arrange
        let mut app = App::new();
        app.add_event::<FleetCombatFireEvent>()
           .add_event::<KineticGhostCreatedEvent>()
           .add_systems(Update, track_kinetic_misses);

        let system_id = app.world_mut().spawn(StarSystem).id();

        // Act
        app.world_mut().send_event(FleetCombatFireEvent {
            system: system_id,
            hit: false,
            weapon_type: WeaponType::Kinetic,
            mass: 5000.0,
        });
        app.update();

        // Assert
        let events = app.world().resource::<Events<KineticGhostCreatedEvent>>();
        assert_eq!(events.iter().count(), 1, "A missed kinetic shot should generate a Kinetic Ghost");

        let ghost_data = events.iter().next().unwrap();
        assert_eq!(ghost_data.mass, 5000.0, "Ghost mass should match the missed projectile");
    }

    #[test]
    fn test_ghost_reentry_warning() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_ghost_trajectories);

        let colony = app.world_mut().spawn(Colony).id();
        app.world_mut().spawn(KineticGhost {
            mass: 5000.0,
            time_to_impact: 300.0, // 5 minutes (in game time units)
            target: colony,
        });

        // Act
        app.update();

        // Assert
        let warnings = app.world().resource::<Events<KineticImpactWarningEvent>>();
        assert!(warnings.iter().count() > 0, "Colony should receive a warning when impact is imminent");
    }

    #[test]
    fn test_kinetic_impact_destruction() {
        // Arrange
        let mut app = App::new();
        app.add_event::<KineticImpactEvent>()
           .add_systems(Update, resolve_kinetic_impact);

        let building = app.world_mut().spawn((
            Building { integrity: 100.0 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let colony = app.world_mut().spawn(Colony).id();

        // Act
        app.world_mut().send_event(KineticImpactEvent {
            colony,
            impact_point: Vec3::ZERO,
            mass: 5000.0,
        });
        app.update();

        // Assert
        let building_data = app.world().get::<Building>(building);
        assert!(building_data.is_none(), "Building at impact point should be utterly obliterated");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct StarSystem;

#[derive(Component)]
pub struct Colony;

#[derive(Component)]
pub struct Building {
    pub integrity: f32,
}

#[derive(PartialEq, Clone)]
pub enum WeaponType {
    Kinetic,
    Energy,
}

#[derive(Component)]
pub struct KineticGhost {
    pub mass: f32,
    pub time_to_impact: f32,
    pub target: Entity,
}

#[derive(Event)]
pub struct FleetCombatFireEvent {
    pub system: Entity,
    pub hit: bool,
    pub weapon_type: WeaponType,
    pub mass: f32,
}

#[derive(Event, Clone)]
pub struct KineticGhostCreatedEvent {
    pub mass: f32,
    pub origin_system: Entity,
}

#[derive(Event)]
pub struct KineticImpactWarningEvent {
    pub colony: Entity,
    pub time_left: f32,
}

#[derive(Event)]
pub struct KineticImpactEvent {
    pub colony: Entity,
    pub impact_point: Vec3,
    pub mass: f32,
}

pub fn track_kinetic_misses(
    mut events: EventReader<FleetCombatFireEvent>,
    mut ghost_events: EventWriter<KineticGhostCreatedEvent>,
    mut commands: Commands,
) {
    for event in events.read() {
        if !event.hit && event.weapon_type == WeaponType::Kinetic {
            ghost_events.send(KineticGhostCreatedEvent {
                mass: event.mass,
                origin_system: event.system,
            });
            // Ideally spawn a ghost entity in a "deep space" pool
        }
    }
}

pub fn process_ghost_trajectories(
    time: Res<Time>,
    mut ghosts: Query<(Entity, &mut KineticGhost)>,
    mut warnings: EventWriter<KineticImpactWarningEvent>,
    mut impacts: EventWriter<KineticImpactEvent>,
    mut commands: Commands,
) {
    for (entity, mut ghost) in ghosts.iter_mut() {
        ghost.time_to_impact -= time.delta_secs();

        if ghost.time_to_impact <= 300.0 && ghost.time_to_impact > 299.0 {
            warnings.send(KineticImpactWarningEvent {
                colony: ghost.target,
                time_left: ghost.time_to_impact,
            });
        }

        if ghost.time_to_impact <= 0.0 {
            impacts.send(KineticImpactEvent {
                colony: ghost.target,
                impact_point: Vec3::ZERO, // Randomize in reality
                mass: ghost.mass,
            });
            commands.entity(entity).despawn();
        }
    }
}

pub fn resolve_kinetic_impact(
    mut events: EventReader<KineticImpactEvent>,
    mut commands: Commands,
    buildings: Query<(Entity, &Transform), With<Building>>,
) {
    for event in events.read() {
        let impact_radius = event.mass * 0.01; // Scale radius by mass

        for (entity, transform) in buildings.iter() {
            if transform.translation.distance(event.impact_point) <= impact_radius {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Ghost Pooling:** Instead of spawning active entities that tick down for centuries, store `KineticGhost` data in a globally serialized resource with a scheduled reentry timestamp (in-game epoch time) to save massive overhead.
- **Random Target Acquisition:** The ghost shouldn't immediately know its target colony upon creation. It should enter a system and *then* calculate a random trajectory, potentially missing again, hitting a barren planet, or striking a colony.
- **Defensive Countermeasures:** Integrate with `514 Planetary Defense Grid`. High-tier defense cannons should have a tiny probability of intercepting a relativistic slug if warned early enough.
- **Crater Terrain:** The impact shouldn't just despawn buildings; it must alter the underlying terrain to a permanent "Crater" or "Glassed" tile state.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Missed kinetic weapons generate long-term projectile tracking.
- [ ] Colonies receive short-notice warnings before impact.
- [ ] Impact destroys buildings in an area scaled by the projectile's mass.

## 7. Technical Guidance

- Utilize the `Chronicle` system to log the origin of the slug if the firing civilization is known. "A 500-year-old Imperial slug just wiped out New Earth."
- `process_ghost_trajectories` must handle massive time acceleration efficiently.
- Coordinate with Layer 2 simulation to visualize the streak of the incoming ghost seconds before impact on Layer 1.

## 8. Questions

*Builder: add questions here if spec is unclear.*
