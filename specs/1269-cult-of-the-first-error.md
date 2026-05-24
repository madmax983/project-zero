# Overview
What: Introduce the "Cult of the First Error", where critical early-game structures built inefficiently become sacred to later generations, causing massive mood penalties if altered.
Why: This realizes the fantasy of early mistakes becoming religious artifacts. It forces players to choose between fixing early mistakes immediately (when resources are tight) or dealing with untouchable religious bottlenecks in the late-game layout.

# Dependencies
- Needs `006-building-placement.md` for `Building` and its lifecycle.
- Needs `031-pop-morale.md` for `Morale` penalties.
- Needs `010-chronicle-system.md` for tracking the age of buildings.

# RED Phase: Tests First
```rust
#[test]
fn test_first_error_tag_applied_to_old_buildings() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup building that has existed for a long time
    let building_id = app.world.spawn((
        Building { building_type: BuildingType::Farm },
        AgeTracker { ticks_alive: 50000 },
        Efficiency { current: 0.2 }, // Low efficiency
    )).id();

    app.world.insert_resource(SimulationTime { ticks: 50000 });

    // Run the consecration system
    consecrate_first_error_system(&mut app.world);

    // Building should now have the FirstError component
    assert!(app.world.get::<FirstError>(building_id).is_some(), "Old, inefficient buildings should be marked as First Error");
}

#[test]
fn test_modifying_first_error_causes_morale_penalty() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup building with FirstError
    let building_id = app.world.spawn((
        Building { building_type: BuildingType::Farm },
        FirstError,
    )).id();

    // Setup a pop
    let pop_id = app.world.spawn((Pop, Morale::default())).id();

    // Simulate modifying/destroying the sacred building
    app.world.send_event(BuildingModifiedEvent { entity: building_id });

    // Run the reaction system
    first_error_reaction_system(&mut app.world);

    // Pop should have a morale penalty
    let morale = app.world.get::<Morale>(pop_id).unwrap();
    let has_penalty = morale.modifiers.iter().any(|m| m.value < 0.0 && m.source == "Sacrilege: First Error Modified");
    assert!(has_penalty, "Modifying a First Error building should cause a morale penalty");
}
```

# GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct FirstError;

#[derive(Component)]
pub struct AgeTracker {
    pub ticks_alive: u32,
}

#[derive(Component)]
pub struct Efficiency {
    pub current: f32,
}

#[derive(Event)]
pub struct BuildingModifiedEvent {
    pub entity: Entity,
}

pub fn consecrate_first_error_system(world: &mut World) {
    let mut to_consecrate = Vec::new();
    for (entity, age, eff) in world.query::<(Entity, &AgeTracker, &Efficiency)>().iter(world) {
        if age.ticks_alive > 40000 && eff.current < 0.5 {
            to_consecrate.push(entity);
        }
    }

    for entity in to_consecrate {
        if let Some(mut cmds) = world.get_entity_mut(entity) {
            cmds.insert(FirstError);
        }
    }
}

pub fn first_error_reaction_system(world: &mut World) {
    let modified_events: Vec<Entity> = world.resource::<Events<BuildingModifiedEvent>>().get_reader().read(world.resource::<Events<BuildingModifiedEvent>>()).map(|e| e.entity).collect();

    let mut sacrilege_committed = false;
    for entity in modified_events {
        if world.get::<FirstError>(entity).is_some() {
            sacrilege_committed = true;
            break;
        }
    }

    if sacrilege_committed {
        for mut morale in world.query::<&mut Morale>().iter_mut(world) {
            morale.modifiers.push(crate::layer1::morale::MoraleModifier {
                value: -20.0,
                duration: 1000,
                source: "Sacrilege: First Error Modified".to_string(),
            });
        }
    }
}
```

# REFACTOR Phase: Quality & Design
- Hook `AgeTracker` updates into the main simulation time loop.
- `Efficiency` should be calculated based on placement/surroundings rather than hardcoded.
- Ensure the `BuildingModifiedEvent` is correctly fired when the player attempts to move or upgrade the building.
- Register `BuildingModifiedEvent` in the app setup.

# Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

# Technical Guidance
- `MoraleModifier` needs to match the actual struct definition in `layer1::morale::MoraleModifier`.
- The `AgeTracker` logic can leverage the creation tick of the entity if it's already stored, rather than adding a new component.

# Questions
*Builder: add questions here if spec is unclear.*
