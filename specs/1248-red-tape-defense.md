# Specification: 1248 Red Tape Defense

## 1. Overview
Weaponizing bureaucracy against an empire. When a hostile Layer 3 faction demands tribute or an inspection, the player can use "Bureaucracy" (Admin resource) to stall them. By sending forms, citing obscure regulations, and delaying landing permissions, the player can force hostile fleets to orbit harmlessly for an extended period. This buys time to build a proper defense grid.

## 2. Dependencies
- Layer 3 Faction demands/invasion logic
- Admin/Bureaucracy resource tracking
- Fleet orbit and landing states

## 3. RED Phase: Tests First
```rust
#[test]
fn test_invoke_red_tape_delays_invasion() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(RedTapePlugin);

    // Arrange: A hostile fleet preparing to invade (timer at 1.0)
    let fleet = app.world_mut().spawn(HostileFleet { invasion_timer: 1.0 }).id();
    app.world_mut().insert_resource(AdminResource { amount: 100 });

    // Act: Invoke red tape
    let mut system_state: SystemState<(Commands, Query<Entity, With<HostileFleet>>, ResMut<AdminResource>)> = SystemState::new(app.world_mut());
    let (mut commands, q_fleets, mut admin) = system_state.get_mut(app.world_mut());

    invoke_red_tape(q_fleets.single(), &mut commands, &mut admin);
    system_state.apply(app.world_mut());

    // Assert: Admin resource decreased, invasion timer increased
    let admin = app.world().resource::<AdminResource>();
    assert_eq!(admin.amount, 50); // Cost 50
    let fleet_data = app.world().get::<HostileFleet>(fleet).unwrap();
    assert!(fleet_data.invasion_timer > 10.0);
}

#[test]
fn test_invoke_red_tape_fails_without_admin() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(RedTapePlugin);

    // Arrange: Insufficient Admin
    let fleet = app.world_mut().spawn(HostileFleet { invasion_timer: 1.0 }).id();
    app.world_mut().insert_resource(AdminResource { amount: 10 });

    // Act
    let mut system_state: SystemState<(Commands, Query<Entity, With<HostileFleet>>, ResMut<AdminResource>)> = SystemState::new(app.world_mut());
    let (mut commands, q_fleets, mut admin) = system_state.get_mut(app.world_mut());

    invoke_red_tape(q_fleets.single(), &mut commands, &mut admin);
    system_state.apply(app.world_mut());

    // Assert: Admin unchanged, timer unchanged
    assert_eq!(app.world().resource::<AdminResource>().amount, 10);
    assert_eq!(app.world().get::<HostileFleet>(fleet).unwrap().invasion_timer, 1.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct HostileFleet {
    pub invasion_timer: f32,
}

#[derive(Resource)]
pub struct AdminResource {
    pub amount: u32,
}

pub struct RedTapePlugin;

impl Plugin for RedTapePlugin {
    fn build(&self, _app: &mut App) {}
}

const RED_TAPE_COST: u32 = 50;
const DELAY_AMOUNT: f32 = 20.0;

pub fn invoke_red_tape(
    fleet_entity: Entity,
    commands: &mut Commands,
    admin: &mut ResMut<AdminResource>,
) {
    if admin.amount >= RED_TAPE_COST {
        admin.amount -= RED_TAPE_COST;
        commands.entity(fleet_entity).insert(DelayedByBureaucracy(DELAY_AMOUNT));
    }
}

#[derive(Component)]
pub struct DelayedByBureaucracy(pub f32);

// Standard system to process the delay
fn process_bureaucracy_delays(
    mut q_fleets: Query<(&mut HostileFleet, &mut DelayedByBureaucracy)>,
) {
    for (mut fleet, mut delay) in q_fleets.iter_mut() {
        fleet.invasion_timer += delay.0;
        delay.0 = 0.0; // In reality, this might increment the timer frame-by-frame or apply a modifier
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **State management:** Instead of directly adding to an `invasion_timer`, insert a `BureaucraticHold` component that prevents the timer from ticking down until it expires.
- **Scaling Costs:** Red tape should cost more each consecutive time it's used on the same fleet.
- **UI Feedback:** Ensure the delay action integrates with a visible player-facing event notification so the player knows they successfully stalled the invasion.

## 6. Acceptance Criteria (Testable!)
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85%
- [ ] Invoking Red Tape costs Admin resources.
- [ ] A successfully invoked Red Tape delays the hostile fleet's invasion progress.

## 7. Technical Guidance
- Integrate this as a player action within the diplomacy or fleet interaction UI.
- Use an action intent system rather than direct commands if the architecture expects it.

## 8. Questions
*Builder: add questions here if spec is unclear.*
