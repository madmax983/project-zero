# Nebulae

## 1. Overview
The fog of war in space. Nebulae are regions of the System Map with distinct environmental effects. "Ion Clouds" disable shields and sensors, "Dust Clouds" slow down movement, and "Protoplanetary Disks" damage hulls over time. This creates strategic opportunities for ambush, escape, or hazardous navigation. A smaller fleet can dive into an Ion Cloud to evade or ambush a larger force at point-blank range, balancing the safety of open space against the tactical advantages and hazards of the nebula.

## 2. Dependencies
- Layer 2 System Map
- Fleet Movement and Combat Systems
- Sensor Systems

## 3. RED Phase: Tests First
```rust
#[test]
fn test_nebula_ion_cloud_disables_sensors() {
    let mut app = bevy::app::App::new();

    // Arrange: Spawn an Ion Cloud Nebula region and a Fleet with active sensors
    let nebula = app.world_mut().spawn((Nebula::IonCloud, SpatialVolume::new(10.0))).id();
    let fleet = app.world_mut().spawn((Fleet, Transform::default(), Sensors { range: 100.0 }, Shields::default())).id();

    // Act: Move the Fleet into the Ion Cloud region
    app.world_mut().entity_mut(fleet).get_mut::<Transform>().unwrap().translation = Vec3::ZERO;
    app.update();

    // Assert: Verify the Fleet's sensor range is reduced to 0 and shields are disabled
    let sensors = app.world().get::<Sensors>(fleet).unwrap();
    let shields = app.world().get::<Shields>(fleet).unwrap();
    assert_eq!(sensors.range, 0.0);
    assert!(shields.is_disabled());
}

#[test]
fn test_nebula_dust_cloud_slows_movement() {
    let mut app = bevy::app::App::new();

    // Arrange: Spawn a Dust Cloud Nebula region and a Fleet
    let nebula = app.world_mut().spawn((Nebula::DustCloud, SpatialVolume::new(10.0))).id();
    let fleet = app.world_mut().spawn((Fleet, Transform::default(), MovementSpeed { base: 10.0, current: 10.0 })).id();

    // Act: Move the Fleet through the Dust Cloud
    app.world_mut().entity_mut(fleet).get_mut::<Transform>().unwrap().translation = Vec3::ZERO;
    app.update();

    // Assert: Verify the Fleet's movement speed is reduced while inside the cloud
    let speed = app.world().get::<MovementSpeed>(fleet).unwrap();
    assert!(speed.current < speed.base);
}

#[test]
fn test_nebula_protoplanetary_disk_hull_damage() {
    let mut app = bevy::app::App::new();

    // Arrange: Spawn a Protoplanetary Disk Nebula region and a Fleet
    let nebula = app.world_mut().spawn((Nebula::ProtoplanetaryDisk, SpatialVolume::new(10.0))).id();
    let fleet = app.world_mut().spawn((Fleet, Transform::default(), Hull { max: 100.0, current: 100.0 })).id();

    // Act: Advance time while the Fleet is in the Disk
    app.world_mut().entity_mut(fleet).get_mut::<Transform>().unwrap().translation = Vec3::ZERO;
    app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs(1));
    app.update();

    // Assert: Verify the Fleet's hull takes incremental damage over time
    let hull = app.world().get::<Hull>(fleet).unwrap();
    assert!(hull.current < hull.max);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal implementation for Nebula regions on Layer 2 and their effects on Fleet components (Sensors, Speed, Hull).
```

## 5. REFACTOR Phase: Quality & Design
- Create an Enum or Trait for different Nebula types and their specific systemic effects.
- Ensure efficient overlap detection for Fleets entering/exiting Nebula regions.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Nebula regions should be represented as spatial volumes on Layer 2. Systems checking for movement, sensors, and health should query if entities are within these volumes.

## 8. Questions
*Builder: add questions here if spec is unclear.*
