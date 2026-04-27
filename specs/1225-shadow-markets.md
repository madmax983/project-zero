# 1225: Shadow Markets

## 1. Overview
**Layer:** 1

**Fantasy:** Deals done in the dark.

**Mechanic:** "Black Market" traders only spawn on Unlit tiles. They sell forbidden tech/goods. Lighting up the map improves safety but kills the market.

**Emergence:** You intentionally leave the "slums" dark so you can buy illegal "Stim-Packs" to keep your miners working.

**Tension:** Safety (Light) vs. Access (Darkness).

## 2. Dependencies
- Lighting Grid system
- Economy / Trading system

## 3. RED Phase: Tests First
```rust
#[test]
fn test_shadow_market_spawns_in_darkness() {
    let mut app = App::new();
    app.add_systems(Update, spawn_shadow_markets);

    // Create a Lighting grid
    let mut lighting = LightingGrid::new(10, 10);
    lighting.set_light(5, 5, 0.0); // Completely dark
    app.world_mut().insert_resource(lighting);

    // Mock a random spawn trigger
    app.world_mut().insert_resource(Events::<TraderSpawnTrigger>::default());
    app.world_mut().send_event(TraderSpawnTrigger { trader_type: TraderType::BlackMarket });

    app.update();

    // A Black Market trader should exist at the dark spot
    let mut trader_query = app.world_mut().query::<(&Trader, &GridPosition)>();
    let mut found = false;
    for (trader, pos) in trader_query.iter(app.world()) {
        if trader.trader_type == TraderType::BlackMarket {
            assert_eq!(pos.x, 5);
            assert_eq!(pos.y, 5);
            found = true;
        }
    }
    assert!(found);
}

#[test]
fn test_shadow_market_flees_light() {
    let mut app = App::new();
    app.add_systems(Update, evaluate_shadow_market_safety);

    // Spawn a trader
    let trader = app.world_mut().spawn((
        Trader { trader_type: TraderType::BlackMarket },
        GridPosition { x: 5, y: 5 },
    )).id();

    // Create a Lighting grid and suddenly illuminate the trader
    let mut lighting = LightingGrid::new(10, 10);
    lighting.set_light(5, 5, 100.0); // Bright light!
    app.world_mut().insert_resource(lighting);

    app.update();

    // The trader should flee (be despawned or marked for exit)
    assert!(app.world().get::<Trader>(trader).is_none());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn spawn_shadow_markets(
    mut commands: Commands,
    mut events: EventReader<TraderSpawnTrigger>,
    lighting: Res<LightingGrid>,
) {
    for ev in events.read() {
        if ev.trader_type == TraderType::BlackMarket {
            // Find a dark tile (simplified: just grab the first one)
            for y in 0..lighting.height {
                for x in 0..lighting.width {
                    if lighting.get_light(x as i32, y as i32) == 0.0 {
                        commands.spawn((
                            Trader { trader_type: TraderType::BlackMarket },
                            GridPosition { x: x as i32, y: y as i32 },
                        ));
                        return; // Spawned one, done
                    }
                }
            }
        }
    }
}

fn evaluate_shadow_market_safety(
    mut commands: Commands,
    query: Query<(Entity, &GridPosition, &Trader)>,
    lighting: Res<LightingGrid>,
) {
    for (entity, pos, trader) in query.iter() {
        if trader.trader_type == TraderType::BlackMarket {
            if lighting.get_light(pos.x, pos.y) > 10.0 {
                // Spooked by the light!
                commands.entity(entity).despawn();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Hook the trader up to the actual UI inventory screen so you can buy items.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Integrate into the existing `LightingGrid` and `Trader` components.

## 8. Questions
*Builder: add questions here if spec is unclear.*
