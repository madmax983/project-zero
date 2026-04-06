# Specification: Chronobiological Desync

**1. Overview**
In Layer 1, the biological rhythms of colonists clash with the physical rotation of their new planet. If a world's day isn't exactly 24 Earth-hours, pops suffer "Desync", experiencing rapidly increasing fatigue, stress, and lowered movement speed. The player must either medicate their colonists, genetically adapt them, or force artificial day/night cycles using lighting grids in windowless bunkers.

**2. Dependencies**
- `065-day-night-cycle` (Base planetary rotation logic)
- `053-lighting-system` (To create artificial day/night)
- `417-chemical-regulation` (Circadian meds)
- `565-gene-splicing` (For adaptive traits)

**3. RED Phase: Tests First**
```rust
#[test]
fn test_desync_accumulation_on_long_day() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Arrange: Planet has 30 hour day
    let mut time = PlanetaryTime { hours_per_day: 30.0, current_hour: 15.0 };
    app.world.insert_resource(time);

    let pop_entity = app.world.spawn((
        Pop,
        Chronobiology { desync_level: 0.0 },
        Needs { rest: 100.0, stress: 0.0 },
    )).id();

    // Act: Tick time forward 24 "Earth" hours, but still daylight on the planet
    app.world.resource_mut::<Events<TimeTickEvent>>().send(TimeTickEvent { hours: 24.0 });
    app.update();

    // Assert: Desync builds because the sun is still up when their body wants sleep
    let biology = app.world.get::<Chronobiology>(pop_entity).unwrap();
    assert!(biology.desync_level > 20.0, "Desync should accumulate on non-24h worlds");

    let needs = app.world.get::<Needs>(pop_entity).unwrap();
    assert!(needs.stress > 10.0, "Desync causes stress");
}

#[test]
fn test_artificial_lighting_prevents_desync() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Arrange: 30 hour day, but Pop is inside a bunker with an Artificial Cycle
    let mut time = PlanetaryTime { hours_per_day: 30.0, current_hour: 20.0 }; // Real sun is up
    app.world.insert_resource(time);

    let bunker = app.world.spawn(ArtificialCycle { current_phase: LightPhase::Night }).id();

    let pop_entity = app.world.spawn((
        Pop,
        Chronobiology { desync_level: 0.0 },
        Location { room: Some(bunker) },
    )).id();

    // Act: Tick time
    app.world.resource_mut::<Events<TimeTickEvent>>().send(TimeTickEvent { hours: 8.0 });
    app.update();

    // Assert: No desync because the room lights told their body it was night
    let biology = app.world.get::<Chronobiology>(pop_entity).unwrap();
    assert_eq!(biology.desync_level, 0.0, "Artificial lighting should prevent desync");
}

#[test]
fn test_adaptive_trait_immunity() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Arrange: 10 hour day
    let mut time = PlanetaryTime { hours_per_day: 10.0, current_hour: 5.0 };
    app.world.insert_resource(time);

    let pop_entity = app.world.spawn((
        Pop,
        Chronobiology { desync_level: 0.0 },
        Traits::from_vec(vec![TraitType::Adaptive]), // Immune
    )).id();

    // Act: Tick time
    app.world.resource_mut::<Events<TimeTickEvent>>().send(TimeTickEvent { hours: 24.0 });
    app.update();

    // Assert: No desync
    let biology = app.world.get::<Chronobiology>(pop_entity).unwrap();
    assert_eq!(biology.desync_level, 0.0, "Adaptive trait prevents chronobiological desync");
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
// In src/layer1/chronobiology.rs

#[derive(Component)]
pub struct Chronobiology {
    pub desync_level: f32, // 0.0 to 100.0
}

pub struct PlanetaryTime {
    pub hours_per_day: f32,
    pub current_hour: f32,
}

pub enum LightPhase {
    Day,
    Night,
}

#[derive(Component)]
pub struct ArtificialCycle {
    pub current_phase: LightPhase,
}

pub struct TimeTickEvent {
    pub hours: f32,
}

pub fn calculate_desync(
    mut events: EventReader<TimeTickEvent>,
    time: Res<PlanetaryTime>,
    mut pops: Query<(&mut Chronobiology, &mut Needs, Option<&Traits>, Option<&Location>)>,
    bunkers: Query<&ArtificialCycle>,
) {
    for event in events.read() {
        // If the planet has a 24 hour day, no desync occurs naturally.
        let is_abnormal_planet = (time.hours_per_day - 24.0).abs() > 0.1;

        for (mut biology, mut needs, traits, location) in pops.iter_mut() {
            // Check immunity
            if let Some(t) = traits {
                if t.contains(&TraitType::Adaptive) {
                    continue;
                }
            }

            // Check if in an artificial cycle room
            let mut is_artificial = false;
            if let Some(loc) = location {
                if let Some(room_entity) = loc.room {
                    if bunkers.get(room_entity).is_ok() {
                        is_artificial = true;
                    }
                }
            }

            if is_abnormal_planet && !is_artificial {
                // Simplified calculation: just accumulate desync based on time passed
                biology.desync_level += event.hours * 1.5;
                biology.desync_level = biology.desync_level.clamp(0.0, 100.0);

                // Apply stress
                needs.stress += biology.desync_level * 0.1;
            } else if is_artificial {
                // Slowly recover desync inside a bunker
                biology.desync_level -= event.hours * 2.0;
                biology.desync_level = biology.desync_level.max(0.0);
            }
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Tie `ArtificialCycle` directly into the existing `LightingGrid` so players actually have to wire timers to their lamps.
- Add "Circadian Meds" to the `Apothecary` crafting queue as an alternative to bunkers.
- Connect Desync directly to `MovementSpeed` so affected pops literally drag their feet.
- Add UI indicators showing the discrepancy between the planet's sun and the colonist's internal clock.

**6. Acceptance Criteria**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] Pops gain desync on non-24h worlds
- [ ] Adaptive pops are immune
- [ ] Artificial lighting prevents/recovers desync

**7. Technical Guidance**
- Desync should be an invisible modifier that slowly ramps up over days, not an immediate debuff.
- The `PlanetaryTime` logic must correctly parse whether the sun is up or down based on the `hours_per_day` modifier set during world generation.
- Consider adding a Chronicle Event if >50% of the colony reaches critical desync, describing them as "Sleepwalking Zombies."

**8. Questions**
*Builder: add questions here if spec is unclear.*
