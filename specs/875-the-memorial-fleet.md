# The Memorial Fleet

## 1. Overview
The Memorial Fleet allows players to construct heavily armored "Memorial Ships" out of "Tragedy Scrap" generated from severely damaged or destroyed Layer 1 buildings. These ships grant massive Morale auras to nearby allied fleets. However, if a Memorial Ship is destroyed in combat, the entire empire suffers a crippling "Shattered Legacy" morale penalty. This creates a high-risk, high-reward flagship mechanic that weaponizes the emotional core of the civilization.

## 2. Dependencies
- `031-pop-morale.md` (for Morale buffs and penalties)
- `157-ship-classes.md` (for ship construction and types)
- `159-fleet-combat.md` (for combat resolution and ship destruction events)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_building_destruction_generates_tragedy_scrap() {
    let mut app = App::new();
    // Setup building
    // Trigger destruction
    // Assert TragedyScrap item is spawned at location
}

#[test]
fn test_memorial_ship_grants_morale_aura() {
    let mut app = App::new();
    // Setup MemorialShip entity
    // Setup allied fleet in range
    // Run systems
    // Assert allied fleet has MemorialAura morale buff
}

#[test]
fn test_memorial_ship_destruction_causes_shattered_legacy() {
    let mut app = App::new();
    // Setup MemorialShip entity
    // Setup allied colonies
    // Trigger ship destruction
    // Run systems
    // Assert allied colonies receive ShatteredLegacy morale penalty
}

#[test]
fn test_shattered_legacy_penalty_decays_over_time() {
    let mut app = App::new();
    // Setup colony with ShatteredLegacy penalty
    // Advance time
    // Assert penalty is reduced or removed
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation to pass the tests

#[derive(Component)]
pub struct TragedyScrap;

#[derive(Component)]
pub struct MemorialShip {
    pub aura_radius: f32,
    pub aura_strength: f32,
}

#[derive(Component)]
pub struct ShatteredLegacy {
    pub penalty_strength: f32,
    pub timer: Timer,
}

pub fn generate_tragedy_scrap_system(
    mut commands: Commands,
    mut destruction_events: EventReader<BuildingDestroyedEvent>,
) {
    for event in destruction_events.read() {
        commands.spawn((
            TragedyScrap,
            Transform::from_translation(event.position),
        ));
    }
}

pub fn apply_memorial_aura_system(
    memorial_ships: Query<(&Transform, &MemorialShip)>,
    mut allied_fleets: Query<(&Transform, &mut Morale), With<AlliedFleet>>,
) {
    for (ship_transform, memorial_ship) in memorial_ships.iter() {
        for (fleet_transform, mut morale) in allied_fleets.iter_mut() {
            if ship_transform.translation.distance(fleet_transform.translation) <= memorial_ship.aura_radius {
                morale.add_modifier(MoodModifier {
                    source: "Memorial Aura".to_string(),
                    value: memorial_ship.aura_strength,
                    duration: None,
                });
            }
        }
    }
}

pub fn handle_memorial_ship_destruction_system(
    mut commands: Commands,
    mut destruction_events: EventReader<ShipDestroyedEvent>,
    memorial_ships: Query<&MemorialShip>,
    mut colonies: Query<(Entity, &mut Morale), With<Colony>>,
) {
    for event in destruction_events.read() {
        if memorial_ships.get(event.ship_entity).is_ok() {
            for (colony_entity, mut morale) in colonies.iter_mut() {
                morale.add_modifier(MoodModifier {
                    source: "Shattered Legacy".to_string(),
                    value: -50.0,
                    duration: Some(Timer::from_seconds(300.0, TimerMode::Once)),
                });
                commands.entity(colony_entity).insert(ShatteredLegacy {
                    penalty_strength: -50.0,
                    timer: Timer::from_seconds(300.0, TimerMode::Once),
                });
            }
        }
    }
}

pub fn decay_shattered_legacy_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ShatteredLegacy)>,
) {
    for (entity, mut legacy) in query.iter_mut() {
        legacy.timer.tick(time.delta());
        if legacy.timer.finished() {
            commands.entity(entity).remove::<ShatteredLegacy>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: The `apply_memorial_aura_system` iterates over all allied fleets for every memorial ship, which is an O(N*M) operation. This could be optimized using spatial partitioning or by updating the aura less frequently.
- **API Improvements**: The `ShatteredLegacy` component could be integrated more cleanly with the existing `Morale` system to handle decay automatically rather than requiring a dedicated system and component.
- **Integration Points**: Ensure that `TragedyScrap` can be hauled and stored like other resources, and that it is consumed correctly during ship construction.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Building destruction generates `TragedyScrap`
- [ ] `MemorialShip` grants a Morale buff to allied fleets in range
- [ ] `MemorialShip` destruction applies a temporary Morale penalty to all colonies

## 7. Technical Guidance
- **Gotchas**: Ensure that the aura effect is not stacked multiple times if a fleet is within range of multiple memorial ships, unless intended.
- **Gotchas**: Handle edge cases where a colony is destroyed while suffering from the `ShatteredLegacy` penalty to prevent panics.

## 8. Questions
*Builder: add questions here if spec is unclear.*