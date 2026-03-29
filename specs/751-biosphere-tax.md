# 751 - The Biosphere Tax

## 1. Overview
Introduce `GlobalPollution` resource. As industrial output creates pollution, the planet adapts violently. Flora mutates into `ArmoredFlora` that requires power tools to clear, and predators gain `ArmorPlated` traits. Extreme pollution levels trigger `AcidStormEvent`s to wash away the smog.

## 2. Dependencies
- `019` Forestry System
- `063` Atmospheric Simulation
- `092` Antagonistic Flora

## 3. RED Phase: Tests First
```rust
#[test]
fn test_pollution_threshold_mutates_flora() {
    let mut app = setup_test_app();

    // Add global pollution above threshold
    app.world_mut().insert_resource(GlobalPollution { level: 1500 });

    let flora = app.world_mut().spawn((
        Flora { is_mutated: false },
        GridPosition { x: 5, y: 5 },
    )).id();

    // Run pollution reaction system
    app.update();

    // Flora should now be mutated and armored
    let mutated_flora = app.world().get::<Flora>(flora).unwrap();
    assert!(mutated_flora.is_mutated);
    assert!(app.world().get::<ArmoredFlora>(flora).is_some());
}

#[test]
fn test_acid_storm_washes_away_pollution() {
    let mut app = setup_test_app();

    app.world_mut().insert_resource(GlobalPollution { level: 2000 });

    // Trigger acid storm
    app.world_mut().send_event(AcidStormEvent { duration: 5 });
    app.update();

    // Pollution should be reduced
    let pollution = app.world().get_resource::<GlobalPollution>().unwrap();
    assert!(pollution.level < 2000);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Resource, Default)]
pub struct GlobalPollution {
    pub level: i32,
}

#[derive(Event)]
pub struct AcidStormEvent {
    pub duration: i32,
}

#[derive(Component)]
pub struct Flora {
    pub is_mutated: bool,
}

#[derive(Component)]
pub struct ArmoredFlora;

pub fn biosphere_adaptation_system(
    mut commands: Commands,
    pollution: Res<GlobalPollution>,
    mut query: Query<(Entity, &mut Flora), Without<ArmoredFlora>>,
) {
    if pollution.level > 1000 {
        for (entity, mut flora) in query.iter_mut() {
            flora.is_mutated = true;
            commands.entity(entity).insert(ArmoredFlora);
        }
    }
}

pub fn process_acid_storm_system(
    mut events: EventReader<AcidStormEvent>,
    mut pollution: ResMut<GlobalPollution>,
) {
    for event in events.read() {
        if event.duration > 0 {
            // Storm reduces pollution
            pollution.level = (pollution.level - 500).max(0);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Make the mutation threshold configurable and possibly localized (based on `AtmosphereGrid` smog diffusion) instead of purely global.
- Give `ArmoredFlora` higher hit points or require specific tool tags (e.g., `Tool::PowerSaw`) to chop.
- `AcidStormEvent` should also damage unshielded buildings/pops while washing the pollution.

## 6. Acceptance Criteria
- [ ] High `GlobalPollution` causes `Flora` to gain the `ArmoredFlora` component.
- [ ] `AcidStormEvent` reduces global pollution levels.
- [ ] Tests pass and `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for pollution mutation systems.

## 7. Technical Guidance
- The pollution metric should tie directly into the `AtmosphereGrid`'s total smog calculation.
- Flora mutation should be gradual—perhaps not all flora mutates instantly at the threshold, but has a random chance per tick based on how far past the threshold the pollution is.

## 8. Questions
- *Builder: Should fauna also mutate immediately, or should we focus on flora first for this spec? (Focus on flora/storms first).*
