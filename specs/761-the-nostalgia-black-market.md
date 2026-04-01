# 761 - The Nostalgia Black Market

## 1. Overview
Pops on deeply traumatized colonies develop a hidden `Nostalgia` need. Smugglers can introduce "Golden Age" artifacts (Layer 3) to the colony (Layer 2). These artifacts provide a massive temporary morale boost but create long-term dependency and enrich rival empires. The player must choose between allowing the drain on their economy for artificial docility or cracking down and facing severe immediate depression and riots.

## 2. Dependencies
- `src/layer1/needs.rs` (For adding the `Nostalgia` need or a generic `Dependency` component)
- `src/layer1/economy/trade.rs` (For smuggling mechanics)
- `src/layer1/social/morale.rs` (For depression/riot mechanics)
- `src/layer1/components.rs` (For `GoldenAgeArtifact` and `Traumatized` markers)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_traumatized_pops_develop_nostalgia_need() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, process_trauma_nostalgia);
    let pop = app.world_mut().spawn((Pop, Traumatized)).id();

    // Act
    app.update();

    // Assert
    assert!(app.world().entity(pop).get::<Needs>().unwrap().nostalgia > 0.0);
}

#[test]
fn test_golden_age_artifact_boosts_morale() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, consume_golden_age_artifacts);
    let pop = app.world_mut().spawn((
        Pop,
        Needs { nostalgia: 50.0, morale: 20.0, ..Default::default() },
        Inventory(vec![Item::GoldenAgeArtifact])
    )).id();

    // Act
    app.update();

    // Assert
    let needs = app.world().entity(pop).get::<Needs>().unwrap();
    assert!(needs.morale > 20.0);
    assert_eq!(needs.nostalgia, 0.0);
}

#[test]
fn test_artifact_withdrawal_causes_depression() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, process_nostalgia_withdrawal);
    let pop = app.world_mut().spawn((
        Pop,
        Needs { nostalgia: 100.0, morale: 50.0, ..Default::default() },
        NostalgiaDependent
    )).id();

    // Act
    app.update();

    // Assert
    let needs = app.world().entity(pop).get::<Needs>().unwrap();
    assert!(needs.morale < 50.0);
    assert!(app.world().entity(pop).get::<Trait>().map_or(false, |t| matches!(t, Trait::Depressed)));
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// In src/layer1/needs.rs
#[derive(Component, Default)]
pub struct Needs {
    // ... existing needs
    pub nostalgia: f32, // 0 to 100
}

#[derive(Component)]
pub struct NostalgiaDependent;

// In src/layer1/economy/items.rs
pub enum Item {
    // ... existing items
    GoldenAgeArtifact,
}

// In src/layer1/social/trauma.rs
pub fn process_trauma_nostalgia(mut query: Query<&mut Needs, With<Traumatized>>) {
    for mut needs in query.iter_mut() {
        needs.nostalgia = (needs.nostalgia + 5.0).min(100.0);
    }
}

pub fn consume_golden_age_artifacts(mut commands: Commands, mut query: Query<(Entity, &mut Needs, &mut Inventory)>) {
    for (entity, mut needs, mut inventory) in query.iter_mut() {
        if let Some(pos) = inventory.0.iter().position(|i| matches!(i, Item::GoldenAgeArtifact)) {
            inventory.0.remove(pos);
            needs.nostalgia = 0.0;
            needs.morale = (needs.morale + 30.0).min(100.0);
            commands.entity(entity).insert(NostalgiaDependent);
        }
    }
}

pub fn process_nostalgia_withdrawal(mut commands: Commands, mut query: Query<(Entity, &mut Needs), With<NostalgiaDependent>>) {
    for (entity, mut needs) in query.iter_mut() {
        if needs.nostalgia >= 100.0 {
            needs.morale = (needs.morale - 10.0).max(0.0);
            commands.entity(entity).insert(Trait::Depressed);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The smuggling event (Layer 2 -> Layer 1) needs a bridge system to periodically spawn `GoldenAgeArtifact` in the colony's shadow market inventory.
- **Economy Drain**: Implement the economic drain by having `GoldenAgeArtifact` purchases deduct `ColonyFunds` or `TradeGoods` and transfer them to a rival empire tracker in Layer 3.
- **UI**: Add a `Nostalgia` meter to the Pop inspector UI if `Traumatized` is present.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops with `Traumatized` component slowly increase `Needs.nostalgia`.
- [ ] Consuming `GoldenAgeArtifact` resets `nostalgia`, boosts `morale`, and applies `NostalgiaDependent`.
- [ ] Reaching 100 `nostalgia` while `NostalgiaDependent` applies `Trait::Depressed` and lowers morale.

## 7. Technical Guidance
- Ensure `Needs.nostalgia` doesn't affect baseline morale calculations until `NostalgiaDependent` is applied to prevent penalizing early-game colonies that haven't encountered artifacts.
- The `Traumatized` component should probably be added by major disaster events (e.g., bombardment, starvation). This is outside the scope of this MVP but keep it in mind.

## 8. Questions
*Builder: add questions here if spec is unclear.*
