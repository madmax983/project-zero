# Specification: Digital Immortality

**1. Overview**
In Layer 1, advanced societies can transfer a Pop's consciousness into the "Mainframe", converting them from a biological entity with needs (hunger, rest, social) into a "Ghost"—a purely data-driven entity. Ghosts consume enormous amounts of power instead of food, offer extreme passive skill bonuses, and can potentially "hack" colony systems when bored.

**2. Dependencies**
- `036-pop-memory` (For transferring consciousness and traits)
- `042-energy-system` (High energy upkeep for Mainframe)
- `126-blackout-protocol` (Loss of power should threaten Ghosts)
- `148-helpful-ai` (Basis for digital entities in the colony)

**3. RED Phase: Tests First**
```rust
#[test]
fn test_mind_upload_transfer() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Arrange: Spawn a Master Engineer with high skills
    let engineer_entity = app.world.spawn((
        Pop,
        Name::new("Master Vance"),
        Skills { engineering: 95 },
        Needs { hunger: 50.0, rest: 20.0 }, // Biological needs
    )).id();

    // Mainframe structure must exist
    let mainframe = app.world.spawn((
        Mainframe { capacity: 10 },
        PowerReceiver { demand: 500 },
    )).id();

    // Act: Issue Mind Upload
    app.world.resource_mut::<Events<MindUploadEvent>>().send(MindUploadEvent {
        target_pop: engineer_entity,
        destination_mainframe: mainframe,
    });
    app.update();

    // Assert: Pop is now a Ghost and biological needs are removed
    assert!(app.world.get::<Ghost>(engineer_entity).is_some(), "Pop should become a Ghost");
    assert!(app.world.get::<Needs>(engineer_entity).is_none(), "Ghosts have no biological needs");
    let skills = app.world.get::<Skills>(engineer_entity).unwrap();
    assert_eq!(skills.engineering, 95, "Skills should be preserved after upload");
}

#[test]
fn test_ghost_power_drain() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Arrange: Ghost in mainframe
    let mainframe = app.world.spawn((
        Mainframe { capacity: 10 },
        PowerReceiver { demand: 500 },
    )).id();
    app.world.spawn((Ghost, ResidentOf(mainframe)));

    // Act: Process power demands
    app.update();

    // Assert: Mainframe power demand increases significantly with Ghosts
    let receiver = app.world.get::<PowerReceiver>(mainframe).unwrap();
    assert!(receiver.demand > 500, "Ghosts should massively increase power drain");
}

#[test]
fn test_ghost_insanity_hack() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Arrange: Bored Ghost
    let door_entity = app.world.spawn(Door { locked: false }).id();
    let ghost = app.world.spawn((
        Ghost,
        Boredom { level: 100.0 }, // Max boredom
    )).id();

    // Act: Tick Ghost AI
    app.world.resource_mut::<Events<GhostHackEvent>>().send(GhostHackEvent {
        hacker: ghost,
        target: door_entity,
    });
    app.update();

    // Assert: Ghost locked the door for amusement
    let door = app.world.get::<Door>(door_entity).unwrap();
    assert!(door.locked, "Ghost should hack and lock the door");
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
// In src/layer1/digital_immortality.rs

#[derive(Component)]
pub struct Ghost;

#[derive(Component)]
pub struct Mainframe {
    pub capacity: u32,
}

#[derive(Component)]
pub struct Boredom {
    pub level: f32,
}

pub struct MindUploadEvent {
    pub target_pop: Entity,
    pub destination_mainframe: Entity,
}

pub fn handle_mind_upload(
    mut commands: Commands,
    mut events: EventReader<MindUploadEvent>,
    query: Query<(Entity, &Skills, &Name)>,
) {
    for event in events.read() {
        if let Ok((entity, skills, name)) = query.get(event.target_pop) {
            // Strip biology, add Ghost
            commands.entity(entity)
                .remove::<Needs>()
                .remove::<Health>()
                .insert(Ghost)
                .insert(Boredom { level: 0.0 });
            // Skills remain!
        }
    }
}

pub fn ghost_power_consumption(
    ghosts: Query<&Ghost>,
    mut mainframes: Query<&mut PowerReceiver, With<Mainframe>>,
) {
    let ghost_count = ghosts.iter().count() as u32;
    for mut receiver in mainframes.iter_mut() {
        receiver.demand = 500 + (ghost_count * 200); // Massive power per ghost
    }
}

pub struct GhostHackEvent {
    pub hacker: Entity,
    pub target: Entity,
}

pub fn process_ghost_hacks(
    mut events: EventReader<GhostHackEvent>,
    mut doors: Query<&mut Door>,
) {
    for event in events.read() {
        if let Ok(mut door) = doors.get_mut(event.target) {
            door.locked = !door.locked; // Toggle locks for fun
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Move `Boredom` into a unified `DigitalPsychology` component so Ghosts can have their own needs distinct from biological ones.
- Tie Ghost power failure to permanent data deletion (death) if the blackout lasts too long.
- Allow Ghosts to be downloaded into "Sleeves" (robotic bodies) for physical labor, combining them with `MachineAwakening` concepts.

**6. Acceptance Criteria**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] Pops can upload their minds to a Mainframe
- [ ] Ghosts consume immense power and lose biological Needs
- [ ] Ghosts can hack colony infrastructure when bored

**7. Technical Guidance**
- Ghosts should use the existing Utility AI framework but with entirely different action sets (e.g., Optimize Grid, Hack Door, Read Archives).
- Ensure the `MindUploadEvent` triggers a Chronicle entry since it effectively "kills" the biological pop.

**8. Questions**
*Builder: add questions here if spec is unclear.*
