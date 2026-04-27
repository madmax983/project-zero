# 1217: Sartorial Rebellion

## 1. Overview
**Layer:** 1

**Fantasy:** The uniform is the first casualty of war.

**Mechanic:** Factions develop "Visual Signifiers" (e.g., rolled sleeves, backwards caps, specific colors). Pops modify their appearance to signal allegiance. Banning these mods increases Unrest but restores "Order".

**Emergence:** You notice the engineers are all wearing red sashes. It's not a fashion statement; it's a strike warning.

**Tension:** Discipline (Uniforms) vs. Expression (Morale/Intel).

## 2. Dependencies
- Faction system
- Pop Equipment/Clothing system
- Policy/Edict system

## 3. RED Phase: Tests First
```rust
#[test]
fn test_sartorial_rebellion_applies_uniform_mod() {
    let mut app = App::new();
    app.add_systems(Update, process_sartorial_rebellion);

    let mut factions = Factions::default();
    factions.get_mut(FactionId::MinersGuild).unwrap().state = FactionState::Unhappy;
    app.world_mut().insert_resource(factions);

    let pop = app.world_mut().spawn((
        Pop,
        FactionMember { faction_id: Some(FactionId::MinersGuild) },
        Equipment { uniform_modifier: None },
    )).id();

    app.update();

    // The unhappy faction member should have modified their uniform
    let equipment = app.world().get::<Equipment>(pop).unwrap();
    assert!(equipment.uniform_modifier.is_some());
}

#[test]
fn test_banning_mods_increases_unrest() {
    let mut app = App::new();
    app.add_systems(Update, enforce_uniform_policy);

    app.world_mut().insert_resource(Unrest { level: 0.0 });
    let mut policies = ColonyPolicies::default();
    policies.toggle(Policy::StrictUniforms); // Enable strict uniforms
    app.world_mut().insert_resource(policies);

    // Spawn a pop with a rebellious uniform mod
    let pop = app.world_mut().spawn((
        Pop,
        Equipment { uniform_modifier: Some("RedSash".to_string()) },
    )).id();

    app.update();

    // The mod is removed, but unrest increases
    let equipment = app.world().get::<Equipment>(pop).unwrap();
    assert!(equipment.uniform_modifier.is_none());

    let unrest = app.world().resource::<Unrest>();
    assert!(unrest.level > 0.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_sartorial_rebellion(
    mut query: Query<(&FactionMember, &mut Equipment)>,
    factions: Res<Factions>,
) {
    for (member, mut equipment) in query.iter_mut() {
        if let Some(fid) = member.faction_id {
            if let Some(faction_data) = factions.get(fid) {
                if faction_data.state == FactionState::Unhappy && equipment.uniform_modifier.is_none() {
                    equipment.uniform_modifier = Some("RebelSash".to_string());
                }
            }
        }
    }
}

fn enforce_uniform_policy(
    mut query: Query<&mut Equipment>,
    policies: Res<ColonyPolicies>,
    mut unrest: ResMut<Unrest>,
) {
    if policies.is_active(Policy::StrictUniforms) {
        for mut equipment in query.iter_mut() {
            if equipment.uniform_modifier.is_some() {
                equipment.uniform_modifier = None;
                unrest.level += 5.0; // Penalty for stripping expression
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Hook the uniform modifications into the rendering system to actually display the sashes or backwards caps.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Use the existing `Equipment` or `Clothing` components.
- Ensure integration with `FactionState`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
