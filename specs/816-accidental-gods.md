# 816: Accidental Gods

## 1. Overview
To the Stone Age locals, your mining ship is a chariot of fire.
Discovering a Primitive world allows you to establish an "Observation Post". If you periodically drop supplies (Food/Tech), you gain "Faith" (a Diplomatic/Unity currency). If you stop, they build effigies and burn them, causing Diplomatic penalties.

You forget to drop the "Manna" (nutrient paste) because of a pirate raid. The primitives launch a crude rocket at your station in retaliation. It actually hits.
Tension: Resource cost of "Miracles" vs. Influence gain.

## 2. Dependencies
- `010-chronicle-system.md` for historical recording of the primitive encounters.
- `701-primitive-civilizations.md` for existing primitives system.
- `039-trade-system.md` for resource dropping mechanics.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_accidental_gods_faith_gain() {
    // Arrange: Setup world with a primitive civilization
    let mut app = setup_test_app();
    let prim_entity = spawn_primitive_civ(&mut app.world);
    let post_entity = spawn_observation_post(&mut app.world, prim_entity);

    // Act: Drop supplies (Manna) to the primitives
    let initial_faith = get_faith_currency(&app.world);
    drop_supplies_to_primitives(&mut app.world, post_entity, ItemType::Food);
    app.update();

    // Assert: Verify Faith currency increases
    let new_faith = get_faith_currency(&app.world);
    assert!(new_faith > initial_faith, "Dropping supplies should generate Faith");
}

#[test]
fn test_accidental_gods_faith_loss_on_neglect() {
    // Arrange: Setup world where primitives received supplies previously
    let mut app = setup_test_app();
    let prim_entity = spawn_primitive_civ(&mut app.world);
    let post_entity = spawn_observation_post(&mut app.world, prim_entity);
    drop_supplies_to_primitives(&mut app.world, post_entity, ItemType::Food);
    app.update();

    // Act: Neglect the primitives for an extended duration
    let initial_diplomacy = get_diplomatic_standing(&app.world, prim_entity);
    advance_simulation_time(&mut app.world, Duration::days(30));
    app.update();

    // Assert: Verify diplomatic penalty is applied due to missing 'miracles'
    let new_diplomacy = get_diplomatic_standing(&app.world, prim_entity);
    assert!(new_diplomacy < initial_diplomacy, "Neglecting primitive followers should cause diplomatic penalties");
}

#[test]
fn test_accidental_gods_primitive_retaliation() {
    // Arrange: Setup neglected primitives
    let mut app = setup_test_app();
    let prim_entity = spawn_primitive_civ(&mut app.world);
    let post_entity = spawn_observation_post(&mut app.world, prim_entity);

    // Act: Advance simulation time to trigger extreme neglect retaliation
    advance_simulation_time(&mut app.world, Duration::days(60));
    trigger_primitive_retaliation(&mut app.world, prim_entity);
    app.update();

    // Assert: Verify the observation post takes damage or a chronicle event is logged
    let post_health = get_building_health(&app.world, post_entity);
    assert!(post_health < 100.0, "Primitives should attack the post when angry");
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Core components and systems for Accidental Gods
#[derive(Component)]
pub struct PrimitiveFollowers {
    pub last_miracle_time: f64,
    pub anger_level: f32,
}

#[derive(Event)]
pub struct PrimitiveRetaliationEvent {
    pub target: Entity,
}

pub fn primitive_faith_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut PrimitiveFollowers, &ObservationPost)>,
    mut faith: ResMut<FaithCurrency>,
    mut events: EventWriter<PrimitiveRetaliationEvent>,
) {
    let current_time = time.elapsed_seconds_f64();
    for (entity, mut followers, post) in query.iter_mut() {
        let time_since_miracle = current_time - followers.last_miracle_time;

        if time_since_miracle > 30.0 {
            // Neglect causes anger to rise
            followers.anger_level += time.delta_seconds() * 0.1;

            // Retaliation trigger
            if followers.anger_level > 10.0 {
                events.send(PrimitiveRetaliationEvent { target: post.entity });
                followers.anger_level = 0.0; // Reset after attack
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Diplomacy Link:** Tie the Faith currency into the broader Layer 3 diplomatic system (if existing) or establish it as a generic Unity resource that buffs domestic output.
- **Varying Miracles:** Different supply types (Food vs Tech vs Meds) should yield different amounts of Faith, and Tech drops should rapidly advance the primitives' tech level, making their retaliations more dangerous (e.g. they invent crude rockets).
- **Chronicle Details:** Generate unique chronicle entries when the primitives burn effigies of the player's colony or launch their first successful attack.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Dropping supplies grants Faith, neglecting them causes anger and eventual damage to observation posts.

## 7. Technical Guidance
- **Timers and Decay:** Use Bevy's time resources to manage the decay of the "miracle timer".
- **Event Integration:** Hook into the cargo/trade system to detect when a drop pod actually lands in the primitive tile to reset the timer.

## 8. Questions
*Builder: add questions here if spec is unclear.*
