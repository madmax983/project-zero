# 750 - The Generational Grudge

## 1. Overview
Introduce the `Grudge` component. When a Pop is wronged by another Pop or Faction, they gain a memory of it. Upon death, this grudge is passed down to their descendants. Over time, descendent factions with inherited grudges will secretly sabotage each other, or if reaching Layer 3 leadership, will declare war against the rival faction's descendants.

## 2. Dependencies
- `036` Pop Memory
- `047` Pop Relationships
- `068` Pop Factions

## 3. RED Phase: Tests First
```rust
#[test]
fn test_grudge_passed_to_descendant() {
    let mut app = setup_test_app();

    let rival_faction = FactionId(2);
    let parent = app.world_mut().spawn((
        Pop,
        Grudges { targets: vec![rival_faction] },
        FamilyId(1),
    )).id();

    let child = app.world_mut().spawn((
        Pop,
        FamilyId(1),
    )).id();

    // Simulate death of parent
    app.world_mut().send_event(PopDeathEvent { pop: parent });
    app.update();

    // Child should now inherit the grudge
    let child_grudges = app.world().get::<Grudges>(child).unwrap();
    assert!(child_grudges.targets.contains(&rival_faction));
}

#[test]
fn test_sabotage_action_between_grudge_rivals() {
    let mut app = setup_test_app();

    let rival_faction = FactionId(2);
    let worker = app.world_mut().spawn((
        Pop,
        Grudges { targets: vec![rival_faction] },
    )).id();

    let building = app.world_mut().spawn((
        Building,
        OwnedBy { faction: rival_faction },
    )).id();

    // Run AI/sabotage evaluation
    app.world_mut().send_event(EvaluateSabotageEvent { pop: worker, target: building });
    app.update();

    // Building should be marked damaged due to grudge sabotage
    let damaged_building = app.world().get::<Damaged>(building);
    assert!(damaged_building.is_some());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component, Default)]
pub struct Grudges {
    pub targets: Vec<FactionId>,
}

#[derive(Component)]
pub struct FamilyId(pub u32);

#[derive(Event)]
pub struct PopDeathEvent {
    pub pop: Entity,
}

#[derive(Event)]
pub struct EvaluateSabotageEvent {
    pub pop: Entity,
    pub target: Entity,
}

#[derive(Component)]
pub struct ChildMarker;

pub fn inherit_grudges_system(
    mut events: EventReader<PopDeathEvent>,
    parents: Query<(&Grudges, &FamilyId), Without<ChildMarker>>,
    mut children: Query<(&mut Grudges, &FamilyId), With<ChildMarker>>,
) {
    for event in events.read() {
        if let Ok((parent_grudges, parent_family)) = parents.get(event.pop) {
            for (mut child_grudges, child_family) in children.iter_mut() {
                if parent_family.0 == child_family.0 {
                    child_grudges.targets.extend(parent_grudges.targets.iter().copied());
                }
            }
        }
    }
}

pub fn process_grudge_sabotage_system(
    mut commands: Commands,
    mut events: EventReader<EvaluateSabotageEvent>,
    pops: Query<&Grudges>,
    buildings: Query<&OwnedBy>,
) {
    for event in events.read() {
        if let Ok(grudges) = pops.get(event.pop) {
            if let Ok(owner) = buildings.get(event.target) {
                if grudges.targets.contains(&owner.faction) {
                    commands.entity(event.target).insert(Damaged);
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- `Grudges` component should probably use a `HashSet` rather than a `Vec` to prevent duplicate inherited grudges over generations.
- Ensure the sabotage system integrates with Utility AI (so Pops choose to sabotage based on high stress or low observation).
- Add specific `AddChronicleEvent`s when a grudge is formed or when a grudge directly causes a war declaration at Layer 3.

## 6. Acceptance Criteria
- [ ] Dying Pops pass their `Grudges` to other Pops sharing their `FamilyId` or descendent lineage.
- [ ] Pops with grudges have a chance to sabotage buildings owned by the target faction.
- [ ] Tests pass and `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Coverage ≥85% for inheritance and sabotage logic.

## 7. Technical Guidance
- Lineage can be complex. `FamilyId` works for the minimal implementation, but if the game uses a fuller ancestry tree, tie inheritance into the actual child-spawning logic.
- Consider adding a `GrudgeIntensity` value that decays or grows over time.

## 8. Questions
- *Builder: How do grudges resolve? Is there a forgiveness event? (Let's start with eternal grudges, we can add peacemaker events later).*
