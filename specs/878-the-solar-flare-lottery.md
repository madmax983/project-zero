# The Solar Flare Lottery

## 1. Overview
Periodic "Solar Flares" sweep through the system (Layer 2) and strike Layer 1 colonies. These flares cause massive EMP damage to unshielded electronics and inflict severe radiation sickness on exposed Pops. However, the flares also temporarily super-charge the atmosphere with "Flare-Isotopes," a rare, rapidly decaying, and incredibly valuable resource. Players must choose between prioritizing absolute safety (locking everyone in bunkers and shutting down the grid) or risking their prospectors' lives to harvest the isotopes before they vanish, trading long-term health for a massive economic boom.

## 2. Dependencies
- `079-weather-events.md` (for global event triggers)
- `191-radioactive-hearth.md` (for radiation sickness mechanics)
- `042-energy-system.md` (for EMP effects on the grid)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_solar_flare_event_damages_unshielded_buildings() {
    let mut app = App::new();
    // Setup shielded and unshielded electronic buildings
    // Trigger SolarFlareEvent
    // Assert only unshielded building health/durability decreases
}

#[test]
fn test_solar_flare_event_causes_radiation_sickness() {
    let mut app = App::new();
    // Setup exposed Pop and sheltered Pop
    // Trigger SolarFlareEvent
    // Assert exposed Pop gains RadiationSickness
    // Assert sheltered Pop is unaffected
}

#[test]
fn test_flare_isotopes_spawn_and_decay() {
    let mut app = App::new();
    // Trigger SolarFlareEvent
    // Assert FlareIsotopes spawn on the map
    // Advance time
    // Assert FlareIsotopes despawn or value decays
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation to pass the tests

#[derive(Component)]
pub struct Electronic;

#[derive(Component)]
pub struct Shielded;

#[derive(Component)]
pub struct BuildingHealth(pub f32);

#[derive(Component)]
pub struct Exposed;

#[derive(Component)]
pub struct RadiationSickness;

#[derive(Component)]
pub struct FlareIsotope {
    pub value: f32,
    pub timer: Timer,
}

#[derive(Event)]
pub struct SolarFlareEvent;

pub fn solar_flare_emp_system(
    mut events: EventReader<SolarFlareEvent>,
    mut buildings: Query<&mut BuildingHealth, (With<Electronic>, Without<Shielded>)>,
) {
    for _ in events.read() {
        for mut health in buildings.iter_mut() {
            health.0 -= 50.0; // Fixed EMP damage
        }
    }
}

pub fn solar_flare_radiation_system(
    mut commands: Commands,
    mut events: EventReader<SolarFlareEvent>,
    exposed_pops: Query<Entity, (With<Pop>, With<Exposed>)>,
) {
    for _ in events.read() {
        for entity in exposed_pops.iter() {
            commands.entity(entity).insert(RadiationSickness);
        }
    }
}

pub fn spawn_flare_isotopes_system(
    mut commands: Commands,
    mut events: EventReader<SolarFlareEvent>,
) {
    for _ in events.read() {
        // Spawn 10 random isotopes for testing
        for i in 0..10 {
            commands.spawn((
                FlareIsotope {
                    value: 100.0,
                    timer: Timer::from_seconds(120.0, TimerMode::Once),
                },
                Transform::from_translation(Vec3::new(i as f32 * 10.0, 0.0, 0.0)),
            ));
        }
    }
}

pub fn decay_flare_isotopes_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FlareIsotope)>,
) {
    for (entity, mut isotope) in query.iter_mut() {
        isotope.timer.tick(time.delta());
        if isotope.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: The `spawn_flare_isotopes_system` currently spawns nodes in a hardcoded line. It should utilize the `TerrainGrid` to scatter isotopes randomly across accessible, outdoor tiles.
- **Refactoring Opportunities**: Combine the `SolarFlareEvent` readers into a single system that handles all flare effects sequentially if ordering becomes an issue, or keep them separate but cleanly grouped in a `SystemSet`.
- **Integration Points**: Ensure that `FlareIsotope` items can be prioritized by haulers (perhaps via a new "Emergency Haul" designation) before they despawn.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Unshielded electronic buildings suffer damage during a flare
- [ ] Exposed Pops contract radiation sickness during a flare
- [ ] `FlareIsotope` entities spawn during a flare and despawn after a set duration

## 7. Technical Guidance
- **Code Structure**: The `Exposed` component on Pops should be dynamically added/removed based on whether they are inside a building with a roof or inside a designated bunker zone.
- **Gotchas**: Isotope decay should ideally be visible to the player (e.g., a declining value metric) to increase tension before they despawn entirely.

## 8. Questions
*Builder: add questions here if spec is unclear.*