# Weaponized Ecology

## 1. Overview
Turning the planet itself into an unassailable fortress that eats your enemies. Players can bio-engineer native flora/fauna to be hyper-aggressive toward non-native biologicals or specific Layer 3 faction signatures. When an invasion occurs, launching the "Death-Spore" protocol dissolves enemy landing forces over days. However, this permanently alters the local biome, drastically reducing habitability and farming efficiency for the colony's own Pops.

## 2. Dependencies
- Needs `Grid` and `Flora` / `Fauna` concepts in `src/layer1/`.
- Needs `InvasionEvent` or hostile force mechanics.
- Needs `Habitability` or `Biome` state in `src/layer1/`.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_death_spore_protocol_destroys_invaders() {
    let mut app = App::new();
    // Setup Planet with Weaponized Ecology capable Flora

    // Setup Invading Force
    let invader = app.world.spawn((Invader, Health { current: 100.0, max: 100.0 })).id();

    // Act: Trigger protocol
    app.world.send_event(TriggerWeaponizedEcologyEvent);
    app.update();

    // Simulate time passing
    app.add_systems(Update, process_weaponized_ecology);
    app.update();

    // Assert: Invaders take massive damage or are destroyed
    let invader_health = app.world.get::<Health>(invader).unwrap();
    assert!(invader_health.current < 100.0);
}

#[test]
fn test_weaponized_ecology_permanently_reduces_habitability() {
    let mut app = App::new();
    // Setup Planet state
    app.world.insert_resource(PlanetaryHabitability { score: 100.0 });

    // Act: Trigger protocol
    app.world.send_event(TriggerWeaponizedEcologyEvent);
    app.update();
    app.add_systems(Update, process_weaponized_ecology);
    app.update();

    // Assert: Habitability is permanently reduced
    let habitability = app.world.resource::<PlanetaryHabitability>();
    assert!(habitability.score < 100.0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Event)]
pub struct TriggerWeaponizedEcologyEvent;

#[derive(Component)]
pub struct Invader;

#[derive(Resource)]
pub struct PlanetaryHabitability {
    pub score: f32,
}

#[derive(Resource, Default)]
pub struct WeaponizedEcologyActive(pub bool);

pub fn trigger_ecology_system(
    mut events: EventReader<TriggerWeaponizedEcologyEvent>,
    mut active: ResMut<WeaponizedEcologyActive>,
) {
    for _ in events.read() {
        active.0 = true;
    }
}

pub fn process_weaponized_ecology(
    active: Res<WeaponizedEcologyActive>,
    mut invaders: Query<&mut Health, With<Invader>>,
    mut habitability: ResMut<PlanetaryHabitability>,
) {
    if active.0 {
        // Damage invaders
        for mut health in invaders.iter_mut() {
            health.current -= 50.0; // Dissolving effect
        }

        // Degrade environment
        habitability.score = (habitability.score - 10.0).max(0.0);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Gradual Effect**: Instead of instant destruction, apply a Damage over Time (DoT) debuff to invaders to simulate the jungle slowly consuming them.
- **Visual Feedback**: Change grid tile visual properties (e.g., turning terrain toxic green or adding spore particle effects) to represent the biome mutation.
- **Pop Consequences**: Introduce a requirement for Pops to wear hazmat gear when working outside after the protocol is used, increasing their stress or slowing movement.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] The Death-Spore protocol damages or kills Layer 3 invaders
- [ ] Activating the protocol permanently decreases planetary habitability

## 7. Technical Guidance
- Integrate the habitability reduction with existing farming/food production logic so the player feels the long-term economic sting of using this ultimate defense.
- You may need to add a "Toxic" or "Mutated" biome tag to the `TerrainGrid`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
