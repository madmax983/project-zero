# 627: Digital Immortality

## 1. Overview
"Mind Upload" tech enables a Pop to save their skills and personality to the Mainframe before death. They become "Ghosts," which consume massive power instead of food/rest but provide passive bonuses. Ghosts can also be downloaded into robot "Sleeves." This introduces extreme late-game moral choices and power drains.

## 2. Dependencies
- `034` Pop Health and Damage
- `042` Energy System
- `411` Machine Awakening

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_mind_upload_preserves_skills_and_traits() {
    // Arrange: Setup test data
    let mut app = App::new();
    let pop = app.world.spawn((Pop, Skills { mining: 10 })).id();
    let mainframe = app.world.spawn(Mainframe).id();

    // Act: Call the feature
    perform_upload(&mut app.world, pop, mainframe);

    // Assert: Verify expected behavior
    let ghosts = app.world.query::<&GhostTrait>().iter(&app.world).count();
    assert_eq!(ghosts, 1);

    // Original physical pop is destroyed
    assert!(app.world.get_entity(pop).is_none());
}

#[test]
fn test_ghost_consumes_power_but_no_food() {
    // Arrange: Setup test data
    let mut app = App::new();
    let ghost = app.world.spawn((GhostTrait, EnergyDrain(50))).id();
    app.insert_resource(PowerGrid { available: 100 });

    // Act: Call the feature
    tick_ghosts(&mut app.world);

    // Assert: Verify expected behavior
    let grid = app.world.get_resource::<PowerGrid>().unwrap();
    assert_eq!(grid.available, 50);
}

#[test]
fn test_ghost_without_power_suffers_insanity() {
    // Test boundary conditions
    let mut app = App::new();
    let ghost = app.world.spawn((GhostTrait, EnergyDrain(50), Insanity(0))).id();
    app.insert_resource(PowerGrid { available: 10 }); // Not enough power

    // Act
    tick_ghosts(&mut app.world);

    // Assert
    let insanity = app.world.get::<Insanity>(ghost).unwrap();
    assert!(insanity.0 > 0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN

pub struct GhostTrait;
pub struct EnergyDrain(pub u32);
pub struct Insanity(pub u32);

pub fn perform_upload(world: &mut World, pop: Entity, mainframe: Entity) {
    let skills = world.get::<Skills>(pop).unwrap().clone();
    world.despawn(pop);
    world.spawn((GhostTrait, skills, EnergyDrain(50), Insanity(0)));
}

pub fn tick_ghosts(world: &mut World) {
    let mut grid = world.get_resource_mut::<PowerGrid>().unwrap();

    let mut query = world.query::<(&EnergyDrain, &mut Insanity)>();
    for (drain, mut insanity) in query.iter_mut(world) {
        if grid.available >= drain.0 {
            grid.available -= drain.0;
        } else {
            insanity.0 += 10;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** Standardize `PowerGrid` consumption by having `EnergyDrain` register with a central `PowerDistribution` system instead of consuming power eagerly during iteration.
- **Code Smells:** `Insanity` accumulation is linear and immediate. Introduce a `DataCorruption` buffer or event that slowly escalates.
- **Design:** Ensure Ghosts still interact with the social/rumor systems (e.g., haunting the Mainframe, broadcasting confusing or brilliant orders to standard Pops).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- The physical Pop must be `despawn`'d or marked as `Dead` with an indicator of upload.
- Ghosts must be exempt from all `metabolism_system` needs (Hunger, Rest) but have a high priority on `PowerGrid` allocation.
- Introduce `GhostSleeve` components to allow ghosts to occupy mechanical bodies, gaining physical capabilities while retaining their traits.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
