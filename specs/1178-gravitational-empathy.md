# 1178: Gravitational Empathy

## 1. Overview
The sheer mass of the planet physically weighs on the minds of the colonists. On high-gravity worlds, Pops develop "Gravitational Empathy"—a latent psychic connection where collective mood alters the local gravity field. If happy, gravity decreases (boosting speed/output). If depressed, gravity increases (crushing buildings and slowing movement).

## 2. Dependencies
- Layer 1 `Mood` system (calculating collective/average mood).
- Layer 1 `GlobalGravity` resource.
- Physics/Movement system modifiers.
- Building durability/destruction mechanics.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_collective_mood_alters_gravity() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, update_gravitational_empathy_system);

    app.world_mut().insert_resource(GlobalGravity { base: 2.0, current: 2.0 });
    app.world_mut().insert_resource(WorldProperty { is_high_gravity: true });

    // Spawn an extremely depressed population
    for _ in 0..10 {
        app.world_mut().spawn((Pop, Mood(0.0))); // 0% happiness
    }

    // Act
    app.update();

    // Assert: Gravity should increase due to depression
    let current_grav = app.world().resource::<GlobalGravity>().current;
    assert!(current_grav > 2.0, "Gravity should increase when population is depressed");
}

#[test]
fn test_crushing_gravity_damages_buildings() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, apply_crushing_gravity_system);

    // Set extreme gravity
    app.world_mut().insert_resource(GlobalGravity { base: 2.0, current: 4.0 });

    // Spawn a fragile building
    let building = app.world_mut().spawn((
        Building,
        Durability { current: 100.0, max: 100.0 }
    )).id();

    // Act
    app.update();

    // Assert: Building takes damage
    let current_dur = app.world().get::<Durability>(building).unwrap().current;
    assert!(current_dur < 100.0, "Building should take damage from extreme gravity");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Mood(pub f32); // 0.0 to 100.0

#[derive(Resource)]
pub struct GlobalGravity {
    pub base: f32,
    pub current: f32,
}

#[derive(Resource)]
pub struct WorldProperty {
    pub is_high_gravity: bool,
}

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct Durability {
    pub current: f32,
    pub max: f32,
}

pub fn update_gravitational_empathy_system(
    mut gravity: ResMut<GlobalGravity>,
    world_prop: Option<Res<WorldProperty>>,
    pops: Query<&Mood, With<Pop>>,
) {
    if let Some(prop) = world_prop {
        if !prop.is_high_gravity || pops.is_empty() {
            return;
        }

        let total_mood: f32 = pops.iter().map(|m| m.0).sum();
        let avg_mood = total_mood / pops.iter().count() as f32;

        // Baseline mood is 50.0.
        // Mood < 50 increases gravity, Mood > 50 decreases it.
        let mood_modifier = (50.0 - avg_mood) * 0.01;

        gravity.current = gravity.base + mood_modifier;
    }
}

pub fn apply_crushing_gravity_system(
    gravity: Res<GlobalGravity>,
    mut buildings: Query<&mut Durability, With<Building>>,
) {
    // Only apply damage if current gravity is exceptionally high (e.g. > 3.0)
    if gravity.current > 3.0 {
        let damage = (gravity.current - 3.0) * 5.0; // scale damage by severity
        for mut dur in buildings.iter_mut() {
            dur.current -= damage;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Time Delta**: `apply_crushing_gravity_system` damage must be scaled by `Time::delta_secs()` so it doesn't destroy the colony in a single tick.
- **Speed Modifiers**: Provide a mechanism where `GlobalGravity.current` directly impacts pop movement speed and work speed.
- **Visuals**: Add screen shake or a heavy atmospheric tint when gravity starts to crush the colony.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] On high gravity worlds, average pop mood dynamically alters the current gravity.
- [ ] Extremely high gravity damages building durability.

## 7. Technical Guidance
- Ensure that destroying a building via `Durability <= 0` triggers the correct clean-up events (e.g., leaving ruins, dropping inventory).

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
