# Overview
What: Introduce "The Propaganda Graffitists", a mechanic where pops with high creativity and high unrest covertly tag buildings with rebellious graffiti. This acts as an aura that lowers work efficiency but boosts the mood of passing pops.
Why: It adds life to the colony and creates a tension between sterilizing the environment for pure efficiency vs. letting pops vent frustration to maintain societal stability through art and humor.

# Dependencies
- Needs `004-pop-entity.md` for `Pop` and `Traits` (Creativity trait).
- Needs `031-pop-morale.md` or `127-stress-breakdowns.md` for tracking Unrest/Stress.
- Needs `006-building-placement.md` for `Building` targets.
- Needs `016-utility-ai-system.md` for the graffiti action and sanitation drone response.

# RED Phase: Tests First
```rust
#[test]
fn test_creative_pop_creates_graffiti_under_high_stress() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup building
    let building_id = app.world.spawn((Building { building_type: BuildingType::Farm }, GridPosition { x: 5, y: 5 })).id();

    // Setup creative pop with high stress
    let mut traits = Traits::default();
    traits.add(Trait::Creative); // Assuming this trait exists
    app.world.spawn((Pop, traits, StressTracker { accumulated_stress: 90.0 }, GridPosition { x: 5, y: 5 }));

    // Run graffiti generation system
    propaganda_graffiti_system(&mut app.world);

    // Check if graffiti component was added to the building
    assert!(app.world.get::<RebelliousGraffiti>(building_id).is_some(), "Highly stressed creative pop should tag the building");
}

#[test]
fn test_graffiti_aura_effects() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup building with graffiti
    let building_id = app.world.spawn((
        Building { building_type: BuildingType::Farm },
        RebelliousGraffiti { intensity: 1.0 },
        GridPosition { x: 5, y: 5 }
    )).id();

    // Setup a worker pop nearby
    let pop_id = app.world.spawn((Pop, GridPosition { x: 5, y: 5 }, Morale::default(), Efficiency { current: 1.0 })).id();

    // Run aura effect system
    graffiti_aura_system(&mut app.world);

    // Check effects
    let eff = app.world.get::<Efficiency>(pop_id).unwrap();
    let morale = app.world.get::<Morale>(pop_id).unwrap();

    assert!(eff.current < 1.0, "Graffiti should lower efficiency");
    assert!(morale.modifiers.iter().any(|m| m.value > 0.0 && m.source == "Venting: Rebellious Graffiti"), "Graffiti should boost morale");
}
```

# GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct RebelliousGraffiti {
    pub intensity: f32,
}

pub fn propaganda_graffiti_system(world: &mut World) {
    let mut to_tag = Vec::new();

    // Find stressed creative pops and their positions
    let mut creative_positions = Vec::new();
    for (_, traits, stress, pos) in world.query::<(Entity, &Traits, &StressTracker, &GridPosition)>().iter(world) {
        if stress.accumulated_stress > 80.0 && traits.has(Trait::Creative) { // Assuming Trait::Creative exists
            creative_positions.push(*pos);
        }
    }

    // Find buildings at those positions
    for (entity, _, pos) in world.query::<(Entity, &Building, &GridPosition)>().iter(world) {
        if creative_positions.contains(pos) && world.get::<RebelliousGraffiti>(entity).is_none() {
            to_tag.push(entity);
        }
    }

    for entity in to_tag {
        if let Some(mut cmds) = world.get_entity_mut(entity) {
            cmds.insert(RebelliousGraffiti { intensity: 1.0 });
        }
    }
}

pub fn graffiti_aura_system(world: &mut World) {
    let mut graffiti_positions = Vec::new();
    for (graffiti, pos) in world.query::<(&RebelliousGraffiti, &GridPosition)>().iter(world) {
        graffiti_positions.push((*pos, graffiti.intensity));
    }

    if graffiti_positions.is_empty() { return; }

    for (pop_pos, mut eff, mut morale) in world.query::<(&GridPosition, &mut Efficiency, &mut Morale)>().iter_mut(world) {
        for (g_pos, intensity) in &graffiti_positions {
            if pop_pos.distance_chebyshev(*g_pos) <= 2 {
                eff.current *= 0.9; // 10% penalty
                morale.modifiers.push(crate::layer1::morale::MoraleModifier {
                    value: 2.0 * intensity,
                    duration: 50,
                    source: "Venting: Rebellious Graffiti".to_string(),
                });
                break; // Only apply once
            }
        }
    }
}
```

# REFACTOR Phase: Quality & Design
- Integrate these systems into `layer1::social::graffiti.rs`.
- Define a proper `SanitationDrone` task or utility action to clean graffiti, removing the `RebelliousGraffiti` component over time.
- If `Trait::Creative` doesn't exist, map it to `Trait::Artistic` or similar. Add it to `Trait` enum if not.
- Ensure the morale modifier stacking logic is sound.

# Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

# Technical Guidance
- `Efficiency` might not be a standard generic component. If it's specific to an `Action` or `Assignment`, modify the task execution speed instead.
- You can leverage `ActiveAuras` or implement the proximity check directly.

# Questions
*Builder: add questions here if spec is unclear.*
