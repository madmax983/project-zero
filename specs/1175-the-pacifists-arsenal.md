# 1175: The Pacifist's Arsenal

## 1. Overview
Weaponizing extreme non-violence to dismantle an enemy's will to fight. A civilization committed to extreme pacifism can research "Empathy Broadcasts." Instead of building warships, they build massive, unarmed broadcast ships. When these ships enter an enemy system, they broadcast overwhelming, neurologically-tailored feelings of guilt, peace, and shared humanity. Enemy fleets caught in the broadcast zone suffer massive "Will to Fight" damage, leading to mutiny.

## 2. Dependencies
- Layer 2/3 Fleet movement and combat system.
- Fleet/Ship components (specifically `WillToFight` or morale).
- Broadcast/Aura mechanics.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_empathy_broadcast_damages_will_to_fight() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, apply_empathy_broadcast_system);

    // Spawn pacifist broadcast ship
    let pacifist_pos = GridPosition { x: 5, y: 5 };
    app.world_mut().spawn((
        Fleet,
        PacifistBroadcast { strength: 20.0, radius: 5.0 },
        pacifist_pos
    ));

    // Spawn enemy combat fleet in range
    let enemy = app.world_mut().spawn((
        Fleet,
        Combatant,
        WillToFight(100.0),
        GridPosition { x: 6, y: 5 }
    )).id();

    // Act
    app.update();

    // Assert: Enemy will to fight is reduced
    let will = app.world().get::<WillToFight>(enemy).unwrap().0;
    assert!(will < 100.0, "Enemy will to fight should be reduced by broadcast");
}

#[test]
fn test_zero_will_triggers_mutiny() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, check_will_to_fight_mutiny_system);

    // Spawn enemy fleet with 0 will to fight
    let enemy = app.world_mut().spawn((
        Fleet,
        Combatant,
        WillToFight(0.0)
    )).id();

    // Act
    app.update();

    // Assert: Fleet is mutinous
    assert!(app.world().get::<Mutinous>(enemy).is_some(), "Fleet should mutiny when will to fight reaches 0");
    // Assert: Fleet is no longer a valid combatant
    assert!(app.world().get::<Combatant>(enemy).is_none(), "Mutinous fleet should lose Combatant status");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Fleet;

#[derive(Component)]
pub struct Combatant;

#[derive(Component)]
pub struct WillToFight(pub f32);

#[derive(Component)]
pub struct PacifistBroadcast {
    pub strength: f32,
    pub radius: f32,
}

#[derive(Component)]
pub struct Mutinous;

#[derive(Component, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn distance(&self, other: &GridPosition) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
}

pub fn apply_empathy_broadcast_system(
    broadcasters: Query<(&PacifistBroadcast, &GridPosition)>,
    mut enemies: Query<(&mut WillToFight, &GridPosition), With<Combatant>>,
) {
    for (broadcast, b_pos) in broadcasters.iter() {
        for (mut will, e_pos) in enemies.iter_mut() {
            if b_pos.distance(e_pos) <= broadcast.radius {
                // Apply damage
                will.0 = (will.0 - broadcast.strength).max(0.0);
            }
        }
    }
}

pub fn check_will_to_fight_mutiny_system(
    mut commands: Commands,
    fleets: Query<(Entity, &WillToFight), With<Combatant>>,
) {
    for (entity, will) in fleets.iter() {
        if will.0 <= 0.0 {
            commands.entity(entity)
                .remove::<Combatant>()
                .insert(Mutinous);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Time scaling**: The broadcast strength should scale with `Time::delta_secs()` so it applies smoothly over time rather than chunking instantly per tick.
- **Faction Checking**: The `apply_empathy_broadcast_system` needs to check faction alignments so it doesn't accidentally neuter allied fleets (unless that's a desired feature of the overwhelming empathy!).
- **Mutiny resolution**: What happens to a `Mutinous` fleet? Do they disband, head home, or join the pacifists? This needs a follow-up state machine.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Empathy broadcasts successfully lower enemy Will to Fight based on distance.
- [ ] Reaching 0 Will to Fight triggers a Mutiny state.

## 7. Technical Guidance
- The aura effect is similar to Layer 1 building auras. Ensure spatial queries are efficient if scaling to hundreds of fleets.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
