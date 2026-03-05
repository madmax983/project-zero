# 303 - The Chrono-Vault

## 1. Overview
The Chrono-Vault allows players to bury a time capsule for their future selves. It is an expensive, heavily armored vault that can be sealed with resources or tech inside. It requires a set real-time or in-game duration to open (e.g., "Locks for 50 in-game years"). When it finally opens, the contents have multiplied in value, yielded unique aged variants, or generated "Ancient" tech bonuses. This mechanic challenges the player to sacrifice current resources for a massive future payoff.

## 2. Dependencies
- `022` Resource Stockpiles
- `045` Structure Durability
- `062` Pop Lifecycle (for long-term time passage representation)
- `146` Command Center (for displaying the long-term timer)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_seal_chrono_vault_starts_timer() {
        // Arrange: Setup unsealed vault and event
        let mut app = App::new();
        let vault = app.world_mut().spawn(ChronoVault {
            state: VaultState::Open,
            contents: vec![ResourceItem { item_type: ResourceType::Metal, amount: 100 }],
            lock_duration: 50.0,
            timer: Timer::from_seconds(0.0, TimerMode::Once),
        }).id();

        app.add_event::<SealVaultEvent>();
        app.add_systems(Update, handle_seal_vault);

        // Act: Send seal event
        app.world_mut().send_event(SealVaultEvent { vault_entity: vault, duration: 50.0 });
        app.update();

        // Assert: Vault is sealed and timer started
        let vault_comp = app.world().get::<ChronoVault>(vault).unwrap();
        assert_eq!(vault_comp.state, VaultState::Sealed, "Vault should be sealed.");
        assert_eq!(vault_comp.timer.duration().as_secs_f32(), 50.0, "Timer duration should match.");
    }

    #[test]
    fn test_vault_opens_and_multiplies_contents() {
        // Arrange: Setup sealed vault near expiration
        let mut app = App::new();
        let mut timer = Timer::from_seconds(50.0, TimerMode::Once);
        timer.set_elapsed(std::time::Duration::from_secs_f32(49.9));

        let vault = app.world_mut().spawn(ChronoVault {
            state: VaultState::Sealed,
            contents: vec![ResourceItem { item_type: ResourceType::Metal, amount: 100 }],
            lock_duration: 50.0,
            timer,
        }).id();

        app.insert_resource(Time::new_with(bevy::utils::Instant::now()));
        // Note: Manual time stepping for testing is complex in bevy 0.13 without specific test helpers,
        // so we just mock the timer tick logic or advance time explicitly if possible.
        // For simplicity, assume `process_vault_timers` uses `Time::delta()`.
        app.add_systems(Update, process_vault_timers);

        // Act: Advance time past expiration
        let mut time_res = app.world_mut().resource_mut::<Time>();
        time_res.advance_by(std::time::Duration::from_secs_f32(0.2));
        app.update();

        // Assert: Vault opened and contents multiplied
        let vault_comp = app.world().get::<ChronoVault>(vault).unwrap();
        assert_eq!(vault_comp.state, VaultState::Opened, "Vault should transition to opened state.");
        assert!(vault_comp.contents[0].amount > 100, "Contents should have multiplied.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// --- Components and Resources ---
#[derive(Clone, PartialEq, Eq)]
pub enum ResourceType {
    Metal,
    Food,
    // Add other resources
}

#[derive(Clone)]
pub struct ResourceItem {
    pub item_type: ResourceType,
    pub amount: u32,
}

#[derive(PartialEq, Eq, Debug)]
pub enum VaultState {
    Open,
    Sealed,
    Opened, // Has finished its cycle
}

#[derive(Component)]
pub struct ChronoVault {
    pub state: VaultState,
    pub contents: Vec<ResourceItem>,
    pub lock_duration: f32, // Years or ticks
    pub timer: Timer,
}

#[derive(Event)]
pub struct SealVaultEvent {
    pub vault_entity: Entity,
    pub duration: f32,
}

// --- Systems ---
pub fn handle_seal_vault(
    mut events: EventReader<SealVaultEvent>,
    mut query: Query<&mut ChronoVault>,
) {
    for event in events.read() {
        if let Ok(mut vault) = query.get_mut(event.vault_entity) {
            if vault.state == VaultState::Open {
                vault.state = VaultState::Sealed;
                vault.lock_duration = event.duration;
                vault.timer = Timer::from_seconds(event.duration, TimerMode::Once);
            }
        }
    }
}

pub fn process_vault_timers(
    time: Res<Time>,
    mut query: Query<&mut ChronoVault>,
) {
    for mut vault in query.iter_mut() {
        if vault.state == VaultState::Sealed {
            if vault.timer.tick(time.delta()).just_finished() {
                vault.state = VaultState::Opened;
                // Basic multiplication factor based on duration (e.g., x2 for 50 units)
                let multiplier = 1.0 + (vault.lock_duration / 50.0);
                for item in vault.contents.iter_mut() {
                    item.amount = (item.amount as f32 * multiplier) as u32;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor Opportunity:** The multiplication logic should differentiate between "Aged" items (e.g., wine, data cores becoming artifacts) vs. simple volume increases. Implement a `ChronoYieldConfig`.
- **Refactor Opportunity:** The timer should use the simulation's `SimulationTime` resource (e.g., Ticks) rather than real-world `Time`, ensuring it respects fast-forward and pauses.
- **Design Improvement:** The vault should become heavily armored when `Sealed` to attract raiders but resist destruction. Add a `Durability` component modifier upon sealing.
- **Integration:** UI must display the vault's remaining time prominently when selected.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.
- [ ] Receiving a `SealVaultEvent` locks the vault and starts the timer.
- [ ] When the timer expires, the vault enters the `Opened` state and multiplies contents.

## 7. Technical Guidance
- **ECS Pattern:** The timer logic must be robust enough to handle saving/loading. Ensure the remaining duration is serialized correctly.
- **Seams:** Connect to the `Inventory` or `ResourceStockpile` system so colonists can load the vault before sealing and extract resources after it opens.
- **Balance:** The lock duration must be punitive (e.g., 10+ in-game years) to make the sacrifice meaningful and the payoff rewarding.

## 8. Questions
*Builder: add questions here if spec is unclear.*
