use crate::layer1::economy::resources::ResourceType;
use crate::layer1::Structure;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub struct ResourceItem {
    pub item_type: ResourceType,
    pub amount: u32,
}

#[derive(PartialEq, Eq, Debug)]
pub enum VaultState {
    Open,
    Sealed,
    Opened,
}

#[derive(Component)]
pub struct ChronoVault {
    pub state: VaultState,
    pub contents: Vec<ResourceItem>,
    pub lock_duration: u64, // Ticks
    pub target_tick: u64,
}

#[derive(Event)]
pub struct SealVaultEvent {
    pub vault_entity: Entity,
    pub duration: u64,
}

pub fn handle_seal_vault(
    mut events: EventReader<SealVaultEvent>,
    mut query: Query<(&mut ChronoVault, Option<&mut Structure>)>,
    time: Res<SimulationTime>,
) {
    for event in events.read() {
        if let Ok((mut vault, mut structure)) = query.get_mut(event.vault_entity) {
            if vault.state == VaultState::Open {
                vault.state = VaultState::Sealed;
                vault.lock_duration = event.duration;
                vault.target_tick = time.tick + event.duration;

                // Heavily armor the vault when sealed
                if let Some(ref mut str) = structure {
                    str.max_hp *= 10.0;
                    str.current_hp = str.max_hp;
                }
            }
        }
    }
}

pub fn process_vault_timers(time: Res<SimulationTime>, mut query: Query<&mut ChronoVault>) {
    for mut vault in query.iter_mut() {
        if vault.state == VaultState::Sealed {
            if time.tick >= vault.target_tick {
                vault.state = VaultState::Opened;
                // Basic multiplication factor based on duration
                let multiplier = 1.0 + (vault.lock_duration as f32 / 50.0);
                for item in vault.contents.iter_mut() {
                    item.amount = (item.amount as f32 * multiplier) as u32;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;

    #[test]
    fn test_seal_chrono_vault_starts_timer_and_armors() {
        let mut app = App::new();
        let vault = app
            .world_mut()
            .spawn((
                ChronoVault {
                    state: VaultState::Open,
                    contents: vec![ResourceItem {
                        item_type: ResourceType::Metal,
                        amount: 100,
                    }],
                    lock_duration: 0,
                    target_tick: 0,
                },
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        app.insert_resource(SimulationTime {
            tick: 10,
            ..Default::default()
        });
        app.add_event::<SealVaultEvent>();
        app.add_systems(Update, handle_seal_vault);

        app.world_mut().send_event(SealVaultEvent {
            vault_entity: vault,
            duration: 50,
        });
        app.update();

        let vault_comp = app.world().get::<ChronoVault>(vault).unwrap();
        assert_eq!(
            vault_comp.state,
            VaultState::Sealed,
            "Vault should be sealed."
        );
        assert_eq!(
            vault_comp.target_tick, 60,
            "Target tick should be current + duration."
        );

        let structure = app.world().get::<Structure>(vault).unwrap();
        assert_eq!(
            structure.max_hp, 1000.0,
            "Vault should be armored upon sealing."
        );
    }

    #[test]
    fn test_vault_opens_and_multiplies_contents() {
        let mut app = App::new();

        let vault = app
            .world_mut()
            .spawn(ChronoVault {
                state: VaultState::Sealed,
                contents: vec![ResourceItem {
                    item_type: ResourceType::Metal,
                    amount: 100,
                }],
                lock_duration: 50,
                target_tick: 100,
            })
            .id();

        app.insert_resource(SimulationTime {
            tick: 99,
            ..Default::default()
        });
        app.add_systems(Update, process_vault_timers);

        app.update();
        let vault_comp = app.world().get::<ChronoVault>(vault).unwrap();
        assert_eq!(
            vault_comp.state,
            VaultState::Sealed,
            "Vault should still be sealed."
        );

        app.world_mut().resource_mut::<SimulationTime>().tick = 100;
        app.update();

        let vault_comp = app.world().get::<ChronoVault>(vault).unwrap();
        assert_eq!(
            vault_comp.state,
            VaultState::Opened,
            "Vault should transition to opened state."
        );
        assert!(
            vault_comp.contents[0].amount > 100,
            "Contents should have multiplied."
        );
        assert_eq!(
            vault_comp.contents[0].amount, 200,
            "Contents should have doubled for 50 duration."
        );
    }
}
