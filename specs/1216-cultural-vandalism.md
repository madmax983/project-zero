# 1216: Cultural Vandalism

## 1. Overview
**Layer:** 1

**Fantasy:** The streets speak back to the palace.

**Mechanic:** High Unrest causes Pops to deface "Official" structures (Statues, Banners, Propaganda Screens). Defaced items invert their buffs (e.g., "Loyalty" banner becomes "Rebellion" symbol).

**Emergence:** You build a massive statue of the Governor to boost morale. Rebels spray-paint it overnight. Now it's a rallying point for the mutiny.

**Tension:** Projection of Power (Statues) vs. Vulnerability to Subversion.

## 2. Dependencies
- Unrest system
- Aesthetics/Beauty system
- Action evaluation pipeline

## 3. RED Phase: Tests First
```rust
#[test]
fn test_defaced_structure_inverts_buff() {
    let mut app = App::new();
    app.add_systems(Update, evaluate_aesthetic_buffs);

    // Spawn a defaced statue
    let statue = app.world_mut().spawn((
        Building { type_: BuildingType::Statue },
        AestheticSource { base_morale_buff: 5.0 },
        Defaced { is_defaced: true },
        GridPosition { x: 5, y: 5 },
    )).id();

    // Pop nearby
    let pop = app.world_mut().spawn((
        Pop,
        Morale { value: 50.0 },
        GridPosition { x: 5, y: 5 },
    )).id();

    app.update();

    // The pop's morale should drop due to the defaced statue
    let morale = app.world().get::<Morale>(pop).unwrap();
    assert!(morale.value < 50.0);
}

#[test]
fn test_high_unrest_triggers_vandalism() {
    let mut app = App::new();
    app.add_systems(Update, process_vandalism_events);

    // Setup high unrest
    app.world_mut().insert_resource(Unrest { level: 90.0 });

    // Spawn a clean statue
    let statue = app.world_mut().spawn((
        Building { type_: BuildingType::Statue },
        AestheticSource { base_morale_buff: 5.0 },
        GridPosition { x: 5, y: 5 },
    )).id();

    // A pop to commit the act
    let rebel = app.world_mut().spawn((
        Pop,
        MentalState::Broken(MentalBreakType::Vandalize),
        GridPosition { x: 5, y: 5 },
    )).id();

    app.world_mut().insert_resource(Events::<VandalizeEvent>::default());
    app.world_mut().send_event(VandalizeEvent { target: statue, perpetrator: rebel });

    app.update();

    // Statue is now defaced
    assert!(app.world().get::<Defaced>(statue).is_some());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn evaluate_aesthetic_buffs(
    mut pop_query: Query<(&GridPosition, &mut Morale), With<Pop>>,
    source_query: Query<(&GridPosition, &AestheticSource, Option<&Defaced>)>,
) {
    for (pop_pos, mut morale) in pop_query.iter_mut() {
        for (source_pos, source, defaced) in source_query.iter() {
            if source_pos == pop_pos { // simplified distance check
                if defaced.is_some() {
                    morale.value -= source.base_morale_buff;
                } else {
                    morale.value += source.base_morale_buff;
                }
            }
        }
    }
}

fn process_vandalism_events(
    mut events: EventReader<VandalizeEvent>,
    mut commands: Commands,
) {
    for ev in events.read() {
        commands.entity(ev.target).insert(Defaced { is_defaced: true });
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Hook the utility AI so pops select "Clean Graffiti" as a chore to remove the `Defaced` component.
- The `Defaced` component should trigger a visual indicator on the client side.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Verify integration with `src/layer1/social/unrest.rs` and `MentalBreakType::Vandalize`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
