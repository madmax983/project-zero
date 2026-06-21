use crate::layer1::resources::ResourceItem;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum VaultState {
    Open,
    Sealed,
    Opened, // Has finished its cycle
}

#[derive(Component, Clone)]
pub struct ChronoVault {
    pub state: VaultState,
    pub contents: Vec<ResourceItem>,
    pub lock_duration_ticks: u64,
    pub sealed_tick: u64,
}

#[derive(Event)]
pub struct SealVaultEvent {
    pub vault_entity: Entity,
    pub lock_duration_ticks: u64,
}

pub fn handle_seal_vault(
    mut events: EventReader<SealVaultEvent>,
    time: Res<SimulationTime>,
    mut query: Query<&mut ChronoVault>,
) {
    for event in events.read() {
        if let Ok(mut vault) = query.get_mut(event.vault_entity) {
            if vault.state == VaultState::Open {
                vault.state = VaultState::Sealed;
                vault.lock_duration_ticks = event.lock_duration_ticks;
                vault.sealed_tick = time.tick;
            }
        }
    }
}

pub fn process_vault_timers(time: Res<SimulationTime>, mut query: Query<&mut ChronoVault>) {
    let current_tick = time.tick;
    for mut vault in query.iter_mut() {
        if vault.state == VaultState::Sealed
            && current_tick >= vault.sealed_tick + vault.lock_duration_ticks
        {
            vault.state = VaultState::Opened;
            // Basic multiplication factor based on duration (e.g., +2% per 1000 ticks)
            // Need to do this manually because f32 / u64 logic is clearer
            #[allow(clippy::cast_precision_loss)]
            let duration_f32 = vault.lock_duration_ticks as f32;
            let multiplier = 1.0 + (duration_f32 / 50000.0);

            for item in vault.contents.iter_mut() {
                #[allow(
                    clippy::cast_precision_loss,
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss
                )]
                let new_amount = (item.amount * multiplier) as u32;
                item.amount = new_amount as f32;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::resources::ResourceType;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 100,
            ..Default::default()
        });
        world.init_resource::<Events<SealVaultEvent>>();
        world
    }

    #[test]
    fn test_seal_chrono_vault_starts_timer() {
        let mut world = setup_world();
        let vault = world
            .spawn(ChronoVault {
                state: VaultState::Open,
                contents: vec![ResourceItem {
                    resource_type: ResourceType::Metal,
                    amount: 100.0,
                }],
                lock_duration_ticks: 0,
                sealed_tick: 0,
            })
            .id();

        world
            .resource_mut::<Events<SealVaultEvent>>()
            .send(SealVaultEvent {
                vault_entity: vault,
                lock_duration_ticks: 1000,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_seal_vault);
        schedule.run(&mut world);

        let vault_comp = world.get::<ChronoVault>(vault).unwrap();
        assert_eq!(
            vault_comp.state,
            VaultState::Sealed,
            "Vault should be sealed."
        );
        assert_eq!(
            vault_comp.lock_duration_ticks, 1000,
            "Lock duration should match."
        );
        assert_eq!(
            vault_comp.sealed_tick, 100,
            "Sealed tick should match current SimulationTime."
        );
    }

    #[test]
    fn test_vault_opens_and_multiplies_contents() {
        let mut world = setup_world();
        let vault = world
            .spawn(ChronoVault {
                state: VaultState::Sealed,
                contents: vec![ResourceItem {
                    resource_type: ResourceType::Metal,
                    amount: 100.0,
                }],
                lock_duration_ticks: 1000,
                sealed_tick: 100,
            })
            .id();

        // Advance time past expiration
        world.resource_mut::<SimulationTime>().tick = 1100;

        let mut schedule = Schedule::default();
        schedule.add_systems(process_vault_timers);
        schedule.run(&mut world);

        let vault_comp = world.get::<ChronoVault>(vault).unwrap();
        assert_eq!(
            vault_comp.state,
            VaultState::Opened,
            "Vault should transition to opened state."
        );
        assert!(
            vault_comp.contents[0].amount > 100.0,
            "Contents should have multiplied."
        );
    }

    #[test]
    fn test_vault_does_not_open_early() {
        let mut world = setup_world();
        let vault = world
            .spawn(ChronoVault {
                state: VaultState::Sealed,
                contents: vec![ResourceItem {
                    resource_type: ResourceType::Metal,
                    amount: 100.0,
                }],
                lock_duration_ticks: 1000,
                sealed_tick: 100,
            })
            .id();

        // Advance time, but not past expiration
        world.resource_mut::<SimulationTime>().tick = 1099;

        let mut schedule = Schedule::default();
        schedule.add_systems(process_vault_timers);
        schedule.run(&mut world);

        let vault_comp = world.get::<ChronoVault>(vault).unwrap();
        assert_eq!(
            vault_comp.state,
            VaultState::Sealed,
            "Vault should still be sealed."
        );
        assert_eq!(
            vault_comp.contents[0].amount, 100.0,
            "Contents should not multiply yet."
        );
    }
}
