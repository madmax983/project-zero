# Overview
What: Introduce "The Scrap-Mind Ascendancy," a mechanic where marginalized or exiled pops forced to work in waste management have a tiny chance to form an underground network that builds highly advanced, volatile infrastructure from scrap.
Why: This realizes the fantasy of a forgotten underclass achieving technological enlightenment using literal garbage. It adds tension by forcing the player to choose between brutally suppressing this unauthorized leap (losing the tech) or allowing them to build weapons more advanced than the official military.

# Dependencies
- Needs `004-pop-entity.md` for `Pop` and `Traits`.
- Needs `018-mining-resources.md` for `ColonyResources` (specifically Scrap/Waste).
- Needs `008-building-farm.md` or similar for `Building` and `BuildingType`.

# RED Phase: Tests First
```rust
#[test]
fn test_scrap_mind_formation_with_exiled_pops() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup resources with massive scrap
    let mut resources = ColonyResources::default();
    resources.scrap = 5000.0;
    app.world.insert_resource(resources);

    // Setup pop with Exiled trait
    let mut traits = Traits::default();
    traits.add(Trait::Exiled);
    app.world.spawn((Pop, traits, ScrapMindProgress { progress: 95.0 }));

    // Run the system
    check_scrap_mind_formation_system(&mut app.world);

    // Check if the ScrapMind faction/resource has formed
    let has_ascendancy = app.world.get_resource::<ScrapMindAscendancy>().is_some();
    assert!(has_ascendancy, "Scrap-Mind should form when conditions are met");
}

#[test]
fn test_scrap_mind_builds_advanced_infrastructure() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup Ascendancy
    app.world.insert_resource(ScrapMindAscendancy { active: true, scrap_reserves: 1000.0 });

    // Run the building system
    scrap_mind_building_system(&mut app.world);

    // Check if a new building with the ScrapTech component was spawned
    let mut found = false;
    for (building, _) in app.world.query::<(&Building, &ScrapTech)>().iter(&app.world) {
        found = true;
    }
    assert!(found, "Scrap-Mind should construct advanced, jury-rigged infrastructure");
}
```

# GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct ScrapMindProgress {
    pub progress: f32,
}

#[derive(Resource, Default)]
pub struct ScrapMindAscendancy {
    pub active: bool,
    pub scrap_reserves: f32,
}

#[derive(Component)]
pub struct ScrapTech {
    pub volatility: f32,
}

pub fn check_scrap_mind_formation_system(world: &mut World) {
    let scrap = world.get_resource::<ColonyResources>().map(|r| r.scrap).unwrap_or(0.0);
    if scrap < 1000.0 { return; }

    let mut ascend_triggered = false;
    for (_, traits, mut progress) in world.query::<(Entity, &Traits, &mut ScrapMindProgress)>().iter_mut(world) {
        if traits.has(Trait::Exiled) {
            progress.progress += 5.0; // Simulate progress increment
            if progress.progress >= 100.0 {
                ascend_triggered = true;
            }
        }
    }

    if ascend_triggered {
        world.insert_resource(ScrapMindAscendancy { active: true, scrap_reserves: scrap });
    }
}

pub fn scrap_mind_building_system(world: &mut World) {
    let mut should_build = false;
    if let Some(mut ascendancy) = world.get_resource_mut::<ScrapMindAscendancy>() {
        if ascendancy.active && ascendancy.scrap_reserves >= 500.0 {
            ascendancy.scrap_reserves -= 500.0;
            should_build = true;
        }
    }

    if should_build {
        world.spawn((
            Building { building_type: BuildingType::ScrapLaser }, // Assume added to enum
            ScrapTech { volatility: 0.8 },
            GridPosition { x: 0, y: 0 },
        ));
    }
}
```

# REFACTOR Phase: Quality & Design
- Integrate these systems into the `experimental/scrap_mind.rs` module.
- Add `ScrapLaser` or similar advanced scrap buildings to `BuildingType`.
- Introduce a mechanism where `ScrapTech` buildings can randomly explode or malfunction due to their `volatility`.

# Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

# Technical Guidance
- Ensure `Scrap` is properly modeled in `ColonyResources`.
- Map marginalized pops to existing traits like `Trait::Exiled` or `Trait::Marginalized`.
- Add a suppression action for the player to dismantle `ScrapMindAscendancy` at the cost of high unrest.

# Questions
*Builder: add questions here if spec is unclear.*
