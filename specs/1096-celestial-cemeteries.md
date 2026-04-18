# 1096: Celestial Cemeteries

## Overview

A Layer 1 -> 2 feature where colonies can adopt an "Orbital Burial" policy. This sends corpses into orbit (Layer 2), freeing up land and providing a Morale boost. However, as "Coffin Density" increases in orbit, it raises the risk of ship collisions or launch failures. Eventually, the debris field may need to be "cleared."

## Dependencies

- Layer 1 Pop/Corpse mechanics.
- Layer 2 Orbit/Launch mechanics.

## RED Phase: Tests First

```rust
#[test]
fn test_orbital_burial_policy_increases_coffin_density() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .init_resource::<OrbitalCemetery>()
       .add_systems(Update, process_corpses_system);

    // Arrange: Enable policy and spawn a corpse
    app.world_mut().insert_resource(BurialPolicy::Orbital);
    app.world_mut().spawn(Corpse);

    app.update();

    // Assert: Corpse despawned from land, added to orbit
    assert_eq!(app.world().query::<&Corpse>().iter(app.world()).count(), 0);
    assert_eq!(app.world().resource::<OrbitalCemetery>().coffin_count, 1);
}

#[test]
fn test_coffin_density_causes_launch_risk() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .insert_resource(OrbitalCemetery { coffin_count: 500 }) // High density
       .add_systems(Update, calculate_launch_risk_system);

    // Arrange: A ship preparing to launch
    let ship_entity = app.world_mut().spawn(LaunchSequence { base_risk: 0.05 }).id();

    app.update();

    // Assert: Risk is increased due to coffin density
    let risk = app.world().get::<LaunchSequence>(ship_entity).unwrap().base_risk;
    assert!(risk > 0.05, "Launch risk should increase with coffin density");
}

#[test]
fn test_clearing_the_cemetery() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .insert_resource(OrbitalCemetery { coffin_count: 100 })
       .add_systems(Update, clear_cemetery_system);

    // Arrange: Fire the clearing laser
    app.world_mut().resource_mut::<Events<ClearCemeteryEvent>>().send(ClearCemeteryEvent {
        coffins_destroyed: 50,
    });

    app.update();

    // Assert: Density reduced
    assert_eq!(app.world().resource::<OrbitalCemetery>().coffin_count, 50);
}

#[test]
fn test_clearing_causes_morale_penalty() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .insert_resource(OrbitalCemetery { coffin_count: 100 })
       .add_systems(Update, clear_cemetery_system);

    // Arrange: A pop observing the desecration
    let pop = app.world_mut().spawn((Pop, Morale { value: 50 })).id();

    app.world_mut().resource_mut::<Events<ClearCemeteryEvent>>().send(ClearCemeteryEvent {
        coffins_destroyed: 10,
    });

    app.update();

    // Assert: Morale is penalized
    let morale = app.world().get::<Morale>(pop).unwrap().value;
    assert!(morale < 50, "Destroying ancestors should lower morale");
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Corpse;

#[derive(Resource)]
pub enum BurialPolicy {
    Land,
    Orbital,
}

#[derive(Resource, Default)]
pub struct OrbitalCemetery {
    pub coffin_count: u32,
}

pub fn process_corpses_system(
    mut commands: Commands,
    policy: Option<Res<BurialPolicy>>,
    mut cemetery: ResMut<OrbitalCemetery>,
    corpse_query: Query<Entity, With<Corpse>>,
) {
    if let Some(p) = policy {
        if matches!(*p, BurialPolicy::Orbital) {
            for entity in corpse_query.iter() {
                commands.entity(entity).despawn();
                cemetery.coffin_count += 1;
            }
        }
    }
}

#[derive(Component)]
pub struct LaunchSequence {
    pub base_risk: f32,
}

pub fn calculate_launch_risk_system(
    cemetery: Res<OrbitalCemetery>,
    mut launch_query: Query<&mut LaunchSequence>,
) {
    // 1% extra risk per 100 coffins
    let extra_risk = (cemetery.coffin_count as f32 / 100.0) * 0.01;
    for mut launch in launch_query.iter_mut() {
        launch.base_risk += extra_risk;
    }
}

#[derive(Event)]
pub struct ClearCemeteryEvent {
    pub coffins_destroyed: u32,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Morale {
    pub value: i32,
}

pub fn clear_cemetery_system(
    mut events: EventReader<ClearCemeteryEvent>,
    mut cemetery: ResMut<OrbitalCemetery>,
    mut pop_query: Query<&mut Morale, With<Pop>>,
) {
    for event in events.read() {
        cemetery.coffin_count = cemetery.coffin_count.saturating_sub(event.coffins_destroyed);

        // Morale penalty: -1 per coffin destroyed
        for mut morale in pop_query.iter_mut() {
            morale.value -= event.coffins_destroyed as i32;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Risk Calculation**: In a real implementation, the launch risk shouldn't be recalculated additively every frame. It should either replace the base risk completely based on current density, or the system should calculate the final chance at the moment of launch.
- **Morale Penalty Scaling**: Subtracting 1 morale per coffin might lead to -5000 morale instantly. Needs a cap or a timed "Desecration" moodlet.
- **Debris Objects**: Instead of a simple integer `coffin_count`, it might be more interesting to spawn actual debris entities in the Layer 2 orbital map.

## Acceptance Criteria

- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new Celestial Cemeteries code.
- [ ] Orbital burial removes corpses from Layer 1 and increases orbital risk in Layer 2.

## Technical Guidance

- Integrate `process_corpses_system` into the Layer 1 pop lifecycle systems.
- Consider making `OrbitalCemetery` a component on the Planet entity rather than a global resource if there are multiple planets.

## Questions

*Builder: add questions here if spec is unclear.*
