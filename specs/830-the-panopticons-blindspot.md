# The Panopticon's Blindspot

## 1. Overview
You see everything, but the things you don't see are terrifying. Constructing a "Total Surveillance Grid" provides a global buff that reduces standard crime and unrest across the system to zero. However, this absolute control generates tiny, invisible "Blindspots" on the grid (1-2 tiles). Dissidents and criminals intuitively find these spots, leading to hyper-concentrated illicit activities that eventually erupt into severe, localized crises (like spontaneous armed rebellions from a single supply closet).

## 2. Dependencies
- Needs `Grid` and `Building` mechanics in `src/layer1/`.
- Needs `Crime` or `Unrest` metrics in `src/layer1/needs.rs` or `src/layer1/society.rs`.
- Needs event generation for rebellions or crises.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_surveillance_grid_eliminates_global_crime_but_spawns_blindspots() {
    let mut app = App::new();
    // Setup Colony with Crime
    app.world.insert_resource(ColonyCrime { level: 50.0 });

    // Act: Build Total Surveillance Grid
    app.world.spawn((Building, SurveillanceGrid));
    app.add_systems(Update, process_surveillance_grid);
    app.update();

    // Assert: Global crime is zero
    assert_eq!(app.world.resource::<ColonyCrime>().level, 0.0);

    // Assert: At least one Blindspot was created on the grid
    let mut blindspot_query = app.world.query::<&Blindspot>();
    assert!(blindspot_query.iter(&app.world).count() > 0);
}

#[test]
fn test_blindspot_accumulates_hyper_crime_and_triggers_crisis() {
    let mut app = App::new();
    // Setup Blindspot
    let blindspot = app.world.spawn((Position { x: 10, y: 10 }, Blindspot { accumulated_crime: 90.0 })).id();

    // Act: Tick blindspot accumulation
    app.add_systems(Update, process_blindspot_accumulation);
    app.update();

    // Assert: Accumulated crime exceeds threshold, triggering a crisis event
    let mut events = app.world.resource_mut::<Events<CrisisEvent>>();
    let crisis_events: Vec<_> = events.drain().collect();
    assert_eq!(crisis_events.len(), 1);

    // The blindspot should reset or be removed after eruption
    let updated_blindspot = app.world.get::<Blindspot>(blindspot).unwrap();
    assert_eq!(updated_blindspot.accumulated_crime, 0.0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct SurveillanceGrid;

#[derive(Component)]
pub struct Blindspot {
    pub accumulated_crime: f32,
}

#[derive(Resource, Default)]
pub struct ColonyCrime {
    pub level: f32,
}

#[derive(Event)]
pub struct CrisisEvent {
    pub location: Position,
    pub severity: f32,
}

pub fn process_surveillance_grid(
    mut commands: Commands,
    grid_query: Query<&SurveillanceGrid>,
    mut crime: ResMut<ColonyCrime>,
    blindspot_query: Query<Entity, With<Blindspot>>,
) {
    if grid_query.iter().count() > 0 {
        crime.level = 0.0;

        // Spawn a blindspot if none exist
        if blindspot_query.iter().count() == 0 {
            commands.spawn((Position { x: 0, y: 0 }, Blindspot { accumulated_crime: 0.0 }));
        }
    }
}

pub fn process_blindspot_accumulation(
    mut commands: Commands,
    mut blindspots: Query<(&Position, &mut Blindspot)>,
    mut crisis_events: EventWriter<CrisisEvent>,
) {
    for (pos, mut blindspot) in blindspots.iter_mut() {
        blindspot.accumulated_crime += 15.0; // Rapid accumulation

        if blindspot.accumulated_crime >= 100.0 {
            crisis_events.send(CrisisEvent {
                location: *pos,
                severity: blindspot.accumulated_crime,
            });
            blindspot.accumulated_crime = 0.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Hidden UI**: Blindspots should be invisible to the player on the standard UI overlays, requiring them to notice Pops pathing weirdly to that specific tile to discover it.
- **Crisis Variety**: The `CrisisEvent` should randomly resolve into an armed rebellion, a massive black-market resource drain, or a rogue AI awakening, depending on colony technology.
- **Mitigation**: Allow players to manually dispatch "Enforcer" Pops to investigate suspected tiles to clear the blindspot manually before it erupts.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Surveillance Grid reduces standard crime to zero
- [ ] Blindspots spawn, invisibly accumulate crime, and trigger localized crisis events

## 7. Technical Guidance
- When modifying pathfinding, give Pops with the `Criminal` or `Dissident` trait a strong Utility AI weight to occasionally visit the Blindspot tile.
- Ensure the `CrisisEvent` integrates with existing combat or disruption systems to actually impact the colony.

## 8. Questions
*Builder: add questions here if spec is unclear.*
