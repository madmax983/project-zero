# Spec 1108: The Rearguard

## 1. Overview
**Layer:** 1 -> 2
**Fantasy:** Someone has to hold the line so the rest can escape.
**Mechanic:** During a planetary evacuation, the player can designate a "Rearguard." These Pops receive massive combat and morale buffs (The Last Stand), but their pathfinding to escape pods is disabled. They are explicitly sacrificed to delay the invading forces while the evacuation timer ticks down.
**Emergence:** You assign your oldest, most experienced veterans to the Rearguard. They hold off a massive invasion fleet just long enough for the colony ships to launch, turning a total wipeout into a heroic, pyrrhic victory.
**Tension:** Do you try to save everyone and risk the colony ships being destroyed on the pad, or do you condemn your best defenders to certain death to guarantee the survival of the civilians?

## 2. Dependencies
- Layer 1 Combat/Militia Systems
- Layer 1 Pathfinding/Escape Pods
- Layer 1 Pop Morale/Traits

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_rearguard_receives_massive_combat_buffs() {
    // Arrange: Spawn a Pop and designate them as Rearguard
    // Act: Run the rearguard buff system
    // Assert: Pop's combat stats and morale are significantly increased
}

#[test]
fn test_rearguard_pathfinding_to_escape_pods_disabled() {
    // Arrange: Spawn a Rearguard Pop and an Escape Pod
    // Act: Attempt to pathfind the Pop to the Escape Pod
    // Assert: Pathfinding fails or is blocked for Rearguard Pops
}

#[test]
fn test_rearguard_delays_enemy_advance() {
    // Arrange: Spawn a Rearguard Pop engaging an Enemy
    // Act: Run the combat simulation
    // Assert: Enemy progress or action speed is significantly reduced
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN

#[derive(Component)]
pub struct Rearguard;

#[derive(Component)]
pub struct CombatStats {
    pub attack: f32,
    pub defense: f32,
}

#[derive(Component)]
pub struct Morale {
    pub level: f32,
}

#[derive(Component)]
pub struct Pathfinding {
    pub target: Option<Entity>,
}

#[derive(Component)]
pub struct EscapePod;

pub fn rearguard_buff_system(
    mut query: Query<(&Rearguard, &mut CombatStats, &mut Morale)>,
) {
    for (_, mut stats, mut morale) in query.iter_mut() {
        stats.attack *= 2.0; // Massive combat buff
        stats.defense *= 2.0;
        morale.level = 100.0; // Unbreakable morale
    }
}

pub fn rearguard_pathfinding_restriction_system(
    mut query: Query<(&Rearguard, &mut Pathfinding)>,
    escape_pods: Query<Entity, With<EscapePod>>,
) {
    for (_, mut pathfinding) in query.iter_mut() {
        if let Some(target) = pathfinding.target {
            if escape_pods.get(target).is_ok() {
                pathfinding.target = None; // Block pathfinding to escape pods
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Visuals:** Rearguard pops should have a distinct visual aura or marker to emphasize their heroic sacrifice.
- **Legacy:** If the colony ships escape, the surviving civilization should gain a permanent "Martyrs' Legacy" buff based on the experience level of the sacrificed Rearguard.
- **Evacuation Timer:** Link the duration the Rearguard survives directly to the success probability of the escaping ships.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Rearguard designation applies massive combat and morale buffs.
- [ ] Rearguard pops cannot pathfind or interact with escape pods/colony ships.

## 7. Technical Guidance
- Integrate closely with the existing `MilitiaSystem` and `Evacuation` mechanics.
- Ensure the `Rearguard` component overrides standard retreat or panic behaviors in the `UtilityAI`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
