# The Petrification Protocol

## 1. Overview
A controversial medical-architectural technology allows the colony to calcify terminally ill or heavily injured Pops into living statues. This preserves their consciousness and converts their bodies into high-strength building materials that provide powerful passive buffs to nearby workers. However, using literal human architecture carries a horrific psychological cost, permanently traumatizing Pops who live and work inside these structures.

## 2. Dependencies
- `048-building-system.md` (for structural integrity and building materials)
- `112-pop-memory.md` (for generating psychological trauma)
- `019-pop-needs.md` (for handling terminal illness/injury states)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_petrify_terminally_ill_pop() {
    let mut app = App::new();
    // Setup terminally ill Pop entity
    // Trigger Petrification command
    // Assert Pop loses `Moveable` and gains `PetrifiedMaterial` component
    // Assert Pop retains consciousness/identity components
}

#[test]
fn test_petrified_material_boosts_structural_integrity() {
    let mut app = App::new();
    // Setup building constructed with standard materials vs. PetrifiedMaterial
    // Apply damage (e.g., fire) to both
    // Assert building with PetrifiedMaterial has significantly higher health/durability
}

#[test]
fn test_petrified_architecture_provides_passive_buff() {
    let mut app = App::new();
    // Setup worker Pop near a petrified structure
    // Assert worker Pop gains productivity/efficiency buff
}

#[test]
fn test_petrified_architecture_causes_trauma() {
    let mut app = App::new();
    // Setup building using PetrifiedMaterial taking damage
    // Assert living wall "screams" (triggers an audio/psychological event)
    // Assert nearby Pops gain permanent psychological trauma Memory
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct TerminallyIll;

#[derive(Component)]
pub struct PetrifiedMaterial {
    pub structural_bonus: f32,
    pub buff_radius: f32,
}

#[derive(Component)]
pub struct BuildingHealth(pub f32);

#[derive(Event)]
pub struct PetrifyPopEvent(pub Entity);

#[derive(Event)]
pub struct LivingWallScreamEvent {
    pub location: Vec3,
}

pub fn petrification_system(
    mut commands: Commands,
    mut events: EventReader<PetrifyPopEvent>,
    query: Query<Entity, With<TerminallyIll>>,
) {
    for event in events.read() {
        if query.contains(event.0) {
            // Convert to material
            commands.entity(event.0)
                .remove::<Moveable>()
                .insert(PetrifiedMaterial {
                    structural_bonus: 500.0,
                    buff_radius: 20.0,
                });
        }
    }
}

pub fn apply_petrified_buff_system(
    materials: Query<(&PetrifiedMaterial, &Transform)>,
    mut workers: Query<(&mut Productivity, &Transform), Without<PetrifiedMaterial>>,
) {
    for (material, mat_transform) in materials.iter() {
        for (mut prod, worker_transform) in workers.iter_mut() {
            if worker_transform.translation.distance(mat_transform.translation) <= material.buff_radius {
                prod.0 *= 1.25; // 25% passive buff
            }
        }
    }
}

pub fn living_wall_trauma_system(
    mut commands: Commands,
    mut events: EventReader<LivingWallScreamEvent>,
    pops: Query<(Entity, &Transform), With<Pop>>,
) {
    for event in events.read() {
        for (entity, transform) in pops.iter() {
            if transform.translation.distance(event.location) < 30.0 {
                // Inflict trauma
                commands.entity(entity).insert(PsychologicalTrauma);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoded `1.25` multiplier and `30.0` radius should be extracted to a centralized configuration resource.
- **Refactoring Opportunities**: Make `PetrifiedMaterial` directly usable in the standard building construction queue, allowing players to select it as an alternative to steel or concrete.
- **Integration Points**: Ensure that `LivingWallScreamEvent` hooks into the global audio manager so the player hears the consequence of their actions.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Terminally ill Pops can be successfully petrified
- [ ] Petrified structures grant structural bonuses and worker buffs
- [ ] Damage to petrified structures triggers trauma in nearby Pops

## 7. Technical Guidance
- **Code Structure**: Ensure the consciousness data (Memories, Name) remains intact on the entity so players can still inspect the "wall" and read the history of the Pop it used to be.
- **Gotchas**: Be careful with stacking multiple petrified buffs; ensure productivity multipliers cap correctly.

## 8. Questions
*Builder: add questions here if spec is unclear.*
