# Planetary Defense Grid

## 1. Overview
**Layer:** Cross-layer (Layer 1 -> 2)
**Fantasy:** Turning your home into a fortress.
**Mechanic:** Build massive "Surface-to-Orbit" cannons on Layer 1. They automatically fire at hostile ships in Layer 2 orbit. Requires line-of-sight (no roof) and massive power.
**Emergence:** A missed shot from your cannon debris rains down on a neighboring friendly colony, causing a diplomatic incident.
**Tension:** Placement: Spread out (harder to defend on ground) or clustered (vulnerable to orbital strike)?

## 2. Dependencies
- Energy/Power Grid (`PowerConsumer`)
- Line of Sight/Placement restrictions (No roof over tile)
- Layer 2 Fleet targeting and Combat

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_cannon_requires_no_roof_to_fire() {
    let mut app = App::new();
    // Setup roofed cannon
    let cannon = app.world_mut().spawn((
        DefenseCannon { range: 1000.0, damage: 50 },
        PowerConsumer { needed: 100, received: 100 },
        Position { x: 5, y: 5, z: 0 },
        HasRoof, // Roofed
    )).id();

    let hostile_ship = app.world_mut().spawn((
        Fleet { allegiance: Allegiance::Hostile },
        Position { x: 5, y: 5, z: 50 }, // Orbit
    )).id();

    app.update();

    // Check if a shot was fired
    let events = app.world().resource::<Events<CannonFireEvent>>();
    let mut reader = events.get_reader();
    let fired = reader.read(events).any(|e| e.shooter == cannon);

    assert!(!fired, "Cannon should not fire if roofed");
}

#[test]
fn test_cannon_hits_hostile_fleet() {
    let mut app = App::new();
    // Setup active, unroofed cannon
    let cannon = app.world_mut().spawn((
        DefenseCannon { range: 1000.0, damage: 50 },
        PowerConsumer { needed: 100, received: 100 },
        Position { x: 5, y: 5, z: 0 },
    )).id();

    let hostile_ship = app.world_mut().spawn((
        Fleet { allegiance: Allegiance::Hostile, health: 100 },
        Position { x: 5, y: 5, z: 50 }, // Orbit
    )).id();

    app.update();

    // Fleet took damage
    let health = app.world().get::<Fleet>(hostile_ship).unwrap().health;
    assert!(health < 100, "Fleet should have taken damage from cannon");
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct DefenseCannon {
    pub range: f32,
    pub damage: i32,
}

#[derive(Component)]
pub struct HasRoof;

#[derive(Event)]
pub struct CannonFireEvent {
    pub shooter: Entity,
    pub target: Entity,
}

pub fn planetary_defense_system(
    mut commands: Commands,
    mut events: EventWriter<CannonFireEvent>,
    cannons: Query<(Entity, &DefenseCannon, &PowerConsumer), Without<HasRoof>>,
    mut fleets: Query<(Entity, &mut Fleet, &Position)>,
) {
    for (entity, cannon, power) in cannons.iter() {
        if power.received < power.needed {
            continue;
        }

        for (target, mut fleet, pos) in fleets.iter_mut() {
            if fleet.allegiance == Allegiance::Hostile {
                let distance = pos.z; // Simplified orbital distance
                if distance <= cannon.range {
                    events.send(CannonFireEvent { shooter: entity, target });
                    fleet.health -= cannon.damage;
                    break; // Fired once per tick
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Implement missed shot logic (debris events on neighboring tiles or factions).
- Implement a cooldown/reload timer using `Timer`.
- Check if line of sight is obstructed by terrain elevation (e.g., in a valley).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Defense Cannon fires at Layer 2 hostile fleets in range if powered.
- [ ] Defense Cannon does not fire if the tile has a `HasRoof` component.

## 7. Technical Guidance
- The "No Roof" check is critical. Planners should consider whether a cannon needs a retractable roof (costly) or just leave it exposed to Layer 1 weather hazards.
- Ensure the `damage` application ties into the Layer 2 fleet combat resolution systems seamlessly.

## 8. Questions
*Builder: add questions here if spec is unclear.*
