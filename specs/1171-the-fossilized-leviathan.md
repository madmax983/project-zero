# 1171: The Fossilized Leviathan

## 1. Overview
Building your colony inside the decaying ribcage of an ancient, impossible spaceborne creature. A late-game discovery where the player colonizes a massive asteroid that is actually the fossilized remains of a leviathan. Mining operations slowly hollow out the bones, providing unique exotic materials. However, as the colony digs deeper into the core, they begin to uncover preserved organic material. The local gravity and atmosphere slowly begin to change, adapting to the biological "memory" of the creature.

## 2. Dependencies
- Layer 1 `GridPosition`, `TerrainGrid`, and `Biome` system.
- Mining jobs and resource generation system.
- Event system for triggering gravity/atmosphere shifts.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_mining_leviathan_bone_yields_exotic_material() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, mine_leviathan_bone_system);

    // Spawn a leviathan bone terrain tile
    let bone_entity = app.world_mut().spawn((
        TerrainTile,
        LeviathanBone { richness: 100 },
        GridPosition { x: 0, y: 0 }
    )).id();

    // Spawn a miner actively mining the bone
    app.world_mut().spawn((
        Pop,
        Miner,
        MiningTarget(bone_entity)
    ));

    // Act
    app.update();

    // Assert: Exotic material is generated
    let mut query = app.world_mut().query::<&ExoticMaterial>();
    assert!(query.iter(&app.world()).count() > 0, "Exotic material should be generated from mining leviathan bone");
}

#[test]
fn test_deep_mining_triggers_biological_memory() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, check_deep_mining_threshold_system);
    app.add_event::<BiologicalMemoryTriggeredEvent>();

    app.world_mut().insert_resource(LeviathanCoreStatus { depth_mined: 99 });

    // Act: Mine one more depth to cross threshold
    app.world_mut().resource_mut::<LeviathanCoreStatus>().depth_mined += 1;
    app.update();

    // Assert: Biological memory event triggered
    let events = app.world().resource::<Events<BiologicalMemoryTriggeredEvent>>();
    let mut reader = events.get_reader();
    assert!(reader.read(events).next().is_some(), "Biological memory should be triggered at threshold");
}

#[test]
fn test_biological_memory_alters_gravity() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, apply_biological_memory_effects_system);
    app.add_event::<BiologicalMemoryTriggeredEvent>();

    app.world_mut().insert_resource(GlobalGravity { multiplier: 1.0 });

    // Act
    app.world_mut().send_event(BiologicalMemoryTriggeredEvent { intensity: 0.5 });
    app.update();

    // Assert: Gravity is altered
    let gravity = app.world().resource::<GlobalGravity>().multiplier;
    assert_ne!(gravity, 1.0, "Gravity should be altered by biological memory");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct TerrainTile;

#[derive(Component)]
pub struct LeviathanBone {
    pub richness: u32,
}

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Miner;

#[derive(Component)]
pub struct MiningTarget(pub Entity);

#[derive(Component)]
pub struct ExoticMaterial;

#[derive(Resource)]
pub struct LeviathanCoreStatus {
    pub depth_mined: u32,
}

#[derive(Event)]
pub struct BiologicalMemoryTriggeredEvent {
    pub intensity: f32,
}

#[derive(Resource)]
pub struct GlobalGravity {
    pub multiplier: f32,
}

pub fn mine_leviathan_bone_system(
    mut commands: Commands,
    mut miners: Query<&MiningTarget, With<Miner>>,
    mut bones: Query<&mut LeviathanBone>,
) {
    for target in miners.iter_mut() {
        if let Ok(mut bone) = bones.get_mut(target.0) {
            if bone.richness > 0 {
                bone.richness -= 1;
                commands.spawn(ExoticMaterial);
            }
        }
    }
}

pub fn check_deep_mining_threshold_system(
    status: Res<LeviathanCoreStatus>,
    mut events: EventWriter<BiologicalMemoryTriggeredEvent>,
) {
    // Arbitrary threshold of 100 for minimal test
    if status.is_changed() && status.depth_mined >= 100 {
        events.send(BiologicalMemoryTriggeredEvent { intensity: 0.5 });
    }
}

pub fn apply_biological_memory_effects_system(
    mut events: EventReader<BiologicalMemoryTriggeredEvent>,
    mut gravity: ResMut<GlobalGravity>,
) {
    for event in events.read() {
        // Arbitrary alteration formula
        gravity.multiplier += event.intensity * 0.1;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Exotic Material Generation**: The current implementation spawns a generic `ExoticMaterial` component. This should integrate with the standard `ResourceItem` inventory and hauling systems.
- **Continuous Alteration**: Instead of a one-time event, the biological memory should probably be a persistent aura or status that slowly creeps outward from the core as mining continues.
- **Psionic Pulses**: Add systems to cause hallucinations in Pops, represented by temporary traits or mood debuffs.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Mining leviathan bones produces exotic materials and deep mining alters global gravity.

## 7. Technical Guidance
- Ensure `LeviathanBone` integrates seamlessly with existing terrain rendering and mining logic.
- The `BiologicalMemoryTriggeredEvent` should probably also hook into the narrative chronicle system to generate lore snippets about the rock singing or the walls breathing.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
