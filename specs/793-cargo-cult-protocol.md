# 793 - Cargo Cult Protocol

## 1. Overview
If an isolated Layer 1 outpost survives a disaster via an emergency drop-pod from orbit, the surviving Pops might develop a "Cargo Cult" belief. They begin interpreting mundane logistical operations (like routine supply shuttles) as divine favor, constructing elaborate, non-functional mock-landing pads out of scrap to attract more 'blessings.' They may even hijack incoming shuttles.

## 2. Dependencies
- Transport/Supply system (`src/layer2/transport.rs` or similar)
- Traits and Beliefs (`src/layer1/traits.rs`)
- Building / Zones (`src/layer1/building.rs`)

## 3. RED Phase: Tests First

```rust
// tests/layer1/cargo_cult_protocol_tests.rs

use bevy::prelude::*;
use scale::layer1::cargo_cult::*;
use scale::layer2::transport::{SupplyDropEvent, ShuttleEntity};
use scale::layer1::pop::Pop;
use scale::layer1::building::Building;

#[test]
fn test_cargo_cult_trait_acquired_after_emergency_drop() {
    let mut app = App::new();
    app.add_event::<SupplyDropEvent>();
    app.add_systems(Update, handle_emergency_drop_event);

    // Spawn isolated outpost pop
    let pop_entity = app.world.spawn((Pop, Transform::from_xyz(0.0, 0.0, 0.0))).id();

    // Trigger emergency drop event nearby
    app.world.send_event(SupplyDropEvent { position: Vec3::ZERO, is_emergency: true });
    app.update();

    // Pop should acquire CargoCult trait
    assert!(app.world.get::<CargoCultTrait>(pop_entity).is_some());
}

#[test]
fn test_mock_landing_pad_construction() {
    let mut app = App::new();
    app.add_systems(Update, construct_mock_landing_pad_system);

    // Spawn pop with Cargo Cult trait
    let pop_entity = app.world.spawn((
        Pop,
        CargoCultTrait,
        Transform::from_xyz(0.0, 0.0, 0.0)
    )).id();

    app.update(); // Trigger construction intent

    // Verify a MockShrine (landing pad) entity is spawned or designated
    let mut shrine_query = app.world.query_filtered::<Entity, With<MockShrine>>();
    assert_eq!(shrine_query.iter(&app.world).count(), 1);
}

#[test]
fn test_supply_shuttle_hijacked() {
    let mut app = App::new();
    app.add_systems(Update, hijack_supply_shuttle_system);

    // Spawn MockShrine
    let shrine_entity = app.world.spawn((MockShrine, Transform::from_xyz(0.0, 0.0, 0.0))).id();

    // Spawn incoming shuttle targeting a different pad
    let shuttle_entity = app.world.spawn((
        ShuttleEntity,
        TargetDestination { position: Vec3::new(10.0, 0.0, 0.0) },
        Transform::from_xyz(0.0, 5.0, 0.0)
    )).id();

    app.update();

    // Shuttle destination should be hijacked to the MockShrine's position
    let target = app.world.get::<TargetDestination>(shuttle_entity).unwrap();
    assert_eq!(target.position, Vec3::ZERO);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/cargo_cult.rs

use bevy::prelude::*;
use crate::layer2::transport::{SupplyDropEvent, ShuttleEntity, TargetDestination};
use crate::layer1::pop::Pop;

#[derive(Component)]
pub struct CargoCultTrait;

#[derive(Component)]
pub struct MockShrine;

pub fn handle_emergency_drop_event(
    mut events: EventReader<SupplyDropEvent>,
    mut pops: Query<(Entity, &Transform), (With<Pop>, Without<CargoCultTrait>)>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.is_emergency {
            for (pop_ent, pop_transform) in pops.iter_mut() {
                if event.position.distance(pop_transform.translation) < 20.0 {
                    commands.entity(pop_ent).insert(CargoCultTrait);
                }
            }
        }
    }
}

pub fn construct_mock_landing_pad_system(
    pops: Query<&Transform, With<CargoCultTrait>>,
    shrines: Query<(), With<MockShrine>>,
    mut commands: Commands,
) {
    // Only build one shrine for simplicity in minimal implementation
    if shrines.is_empty() {
        if let Some(pop_transform) = pops.iter().next() {
            commands.spawn((
                MockShrine,
                Transform::from_translation(pop_transform.translation + Vec3::new(2.0, 0.0, 0.0))
            ));
        }
    }
}

pub fn hijack_supply_shuttle_system(
    shrines: Query<&Transform, With<MockShrine>>,
    mut shuttles: Query<(&Transform, &mut TargetDestination), With<ShuttleEntity>>,
) {
    for (shuttle_transform, mut target) in shuttles.iter_mut() {
        for shrine_transform in shrines.iter() {
            // Very simplistic hijack logic: if close enough to a shrine, divert
            if shuttle_transform.translation.distance(shrine_transform.translation) < 50.0 {
                target.position = shrine_transform.translation;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Event Handling**: `SupplyDropEvent` might need to be more specific to an outpost or zone rather than just checking distance globally.
- **Construction Logic**: Pops shouldn't instantly spawn a `MockShrine`. It should generate a construction designation that Cultist pops fulfill using scrap materials.
- **Hijacking Probability**: Hijacking shouldn't be 100% guaranteed. It could involve a skill check or a probability based on the shrine's 'attractiveness'.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] `CargoCultTrait` is acquired after emergency supply drops.
- [ ] Pops with `CargoCultTrait` initiate construction of `MockShrine`s.
- [ ] `MockShrine`s can hijack the destination of incoming shuttles.

## 7. Technical Guidance

- Ensure `SupplyDropEvent` has a flag or context to identify emergency vs routine.
- The hijacking system should intercept the shuttle's routing before it finalizes its landing sequence.

## 8. Questions

*Builder: add questions here if spec is unclear.*
