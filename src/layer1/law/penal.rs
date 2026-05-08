use crate::layer1::law::justice::Inmate;
use crate::layer1::map::GridPosition;
use crate::layer1::zone::{ZoneGrid, ZoneType};
use bevy_ecs::prelude::*;

/// Component indicating an Inmate is currently performing forced labor.
///
/// This component is added by [`evaluate_penal_work_system`] when an Inmate is in a [`ZoneType::Penal`],
/// and removed by [`cleanup_penal_work_system`] when they leave.
#[derive(Component, Clone, Copy, Debug)]
pub struct PenalLabor {
    /// Work efficiency multiplier (default > 1.0 because fear motivates).
    pub efficiency_bonus: f32,
}

impl Default for PenalLabor {
    fn default() -> Self {
        Self {
            efficiency_bonus: 0.2, // +20% speed
        }
    }
}

/// Tracks the risk of an inmate rebelling.
///
/// Accumulates over time while working.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct RevoltRisk {
    /// Current risk level.
    pub current: f32,
    /// Threshold at which a jailbreak occurs.
    pub threshold: f32,
}

/// Checks if Inmates are in a Penal Zone and assigns `PenalLabor` status.
#[allow(clippy::type_complexity)]
pub fn evaluate_penal_work_system(
    mut commands: Commands,
    zone_grid: Res<ZoneGrid>,
    query: Query<(Entity, &GridPosition), (With<Inmate>, Without<PenalLabor>)>,
) {
    for (entity, pos) in &query {
        if zone_grid.get(pos.x, pos.y) == ZoneType::Penal {
            commands.entity(entity).insert((
                PenalLabor::default(),
                RevoltRisk {
                    current: 0.0,
                    threshold: 100.0,
                },
            ));
        }
    }
}

/// Removes `PenalLabor` if Inmate leaves zone.
pub fn cleanup_penal_work_system(
    mut commands: Commands,
    zone_grid: Res<ZoneGrid>,
    query: Query<(Entity, &GridPosition), With<PenalLabor>>,
) {
    for (entity, pos) in &query {
        if zone_grid.get(pos.x, pos.y) != ZoneType::Penal {
            commands
                .entity(entity)
                .remove::<PenalLabor>()
                .remove::<RevoltRisk>();
        }
    }
}

/// Increases revolt risk for working inmates.
pub fn update_revolt_risk_system(mut query: Query<&mut RevoltRisk, With<PenalLabor>>) {
    for mut risk in &mut query {
        risk.current += 0.1;
    }
}

/// Triggers jailbreak if risk exceeds threshold.
pub fn check_jailbreak_system(
    mut commands: Commands,
    query: Query<(Entity, &RevoltRisk), With<Inmate>>,
) {
    for (entity, risk) in &query {
        if risk.current >= risk.threshold {
            commands
                .entity(entity)
                .remove::<Inmate>()
                .remove::<PenalLabor>()
                .remove::<RevoltRisk>();

            // TODO: Add Wanted status or aggression
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::law::justice::Inmate;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::utility_types::PopAction;
    use crate::layer1::zone::{ZoneGrid, ZoneType};

    fn setup_world() -> World {
        let mut world = World::new();
        let mut zone_grid = ZoneGrid::new(10, 10);
        // Define a Penal Zone at (5,5)
        zone_grid.set(5, 5, ZoneType::Penal);
        world.insert_resource(zone_grid);
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world
    }

    #[test]
    fn test_penal_labor_component_defaults() {
        let labor = PenalLabor::default();
        assert!((labor.efficiency_bonus - 0.2).abs() < f32::EPSILON); // 20% faster
    }

    #[test]
    fn test_inmate_evaluates_work_in_penal_zone() {
        let mut world = setup_world();

        // Inmate at (5,5) which is a Penal Zone
        let inmate = world
            .spawn((
                Pop,
                Inmate {
                    sentence_ticks: 1000,
                },
                GridPosition { x: 5, y: 5 },
                PopAction::default(),
            ))
            .id();

        // Run evaluation system
        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            evaluate_penal_work_system,
        );

        // Should receive PenalLabor component
        assert!(world.get::<PenalLabor>(inmate).is_some());
        // Should receive RevoltRisk
        assert!(world.get::<RevoltRisk>(inmate).is_some());
    }

    #[test]
    fn test_inmate_outside_penal_zone_is_idle() {
        let mut world = setup_world();

        // Inmate at (0,0) - NOT a Penal Zone
        let inmate = world
            .spawn((
                Pop,
                Inmate {
                    sentence_ticks: 1000,
                },
                GridPosition { x: 0, y: 0 },
                PopAction::default(),
            ))
            .id();

        // Ensure system doesn't wrongly add it
        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            evaluate_penal_work_system,
        );
        assert!(world.get::<PenalLabor>(inmate).is_none());

        // Test cleanup
        world
            .entity_mut(inmate)
            .insert((PenalLabor::default(), RevoltRisk::default()));

        // Still at (0,0) which is not Penal
        let _ =
            bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, cleanup_penal_work_system);

        assert!(world.get::<PenalLabor>(inmate).is_none());
    }

    #[test]
    fn test_working_accumulates_revolt_risk() {
        let mut world = setup_world();

        let inmate = world
            .spawn((
                Pop,
                Inmate {
                    sentence_ticks: 1000,
                },
                PenalLabor::default(), // Actively working
                RevoltRisk {
                    current: 0.0,
                    threshold: 100.0,
                },
            ))
            .id();

        // Run system tick
        let _ =
            bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, update_revolt_risk_system);

        let risk = world.get::<RevoltRisk>(inmate).unwrap();
        assert!(risk.current > 0.0);
    }

    #[test]
    fn test_jailbreak_trigger() {
        let mut world = setup_world();

        let inmate = world
            .spawn((
                Pop,
                Inmate {
                    sentence_ticks: 1000,
                },
                RevoltRisk {
                    current: 101.0,
                    threshold: 100.0,
                }, // Over threshold
            ))
            .id();

        let _ =
            bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, check_jailbreak_system);

        // Should lose Inmate status (escaped)
        assert!(world.get::<Inmate>(inmate).is_none());
        assert!(world.get::<PenalLabor>(inmate).is_none());
        assert!(world.get::<RevoltRisk>(inmate).is_none());
    }
}

use crate::layer1::administration::edicts::{ColonyPolicies, Policy};
use crate::layer1::economy::items::ItemType;
use crate::layer1::health::Dead;
use crate::layer1::psychology::traits::Trait;
use crate::layer1::social::morale::Morale;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ColonyInventory {
    pub vital_organs: usize,
}

impl ColonyInventory {
    pub fn add(&mut self, _item: ItemType, amount: usize) {
        self.vital_organs += amount;
    }

    pub fn get_amount(&self, _item: &ItemType) -> usize {
        self.vital_organs
    }
}

#[derive(Event, Default)]
pub struct OrganHarvestedEvent;

pub fn process_dead_pops_for_organs_system(
    mut commands: Commands,
    policies: Option<Res<ColonyPolicies>>,
    inventory: Option<ResMut<ColonyInventory>>,
    dead_pops: Query<Entity, With<Dead>>,
    mut harvest_events: EventWriter<OrganHarvestedEvent>,
) {
    if let Some(policies) = policies {
        if policies.is_active(Policy::MandatoryOrganHarvesting) {
            if let Some(mut inv) = inventory {
                for entity in dead_pops.iter() {
                    inv.add(ItemType::VitalOrgans, 1);
                    commands.entity(entity).try_despawn_recursive();
                    harvest_events.send(OrganHarvestedEvent);
                }
            }
        }
    }
}

pub fn apply_harvesting_horror_system(
    mut events: EventReader<OrganHarvestedEvent>,
    mut pops: Query<(
        &mut Morale,
        Option<&crate::layer1::psychology::traits::Traits>,
    )>,
) {
    let mut harvested = false;
    for _ in events.read() {
        harvested = true;
    }
    if !harvested {
        return;
    }

    for (mut morale, traits_opt) in pops.iter_mut() {
        let is_psycho = traits_opt.is_some_and(|t| t.0.contains(&Trait::Psychopath));
        if !is_psycho {
            morale.value -= 20.0;
            morale.value = morale.value.max(0.0);
        }
    }
}

#[cfg(test)]
mod organ_trade_tests {
    use super::*;

    use crate::layer1::entities::pop::Pop;
    use crate::layer1::psychology::traits::Traits;

    #[test]
    fn test_organ_harvesting_edict_produces_organs() {
        let mut world = World::new();

        let mut policies = ColonyPolicies::default();
        policies
            .active_policies
            .insert(Policy::MandatoryOrganHarvesting);
        world.insert_resource(policies);

        let inventory = ColonyInventory::default();
        world.insert_resource(inventory);
        world.insert_resource(Events::<OrganHarvestedEvent>::default());

        let dead_pop = world.spawn((Pop, Dead)).id();

        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            process_dead_pops_for_organs_system,
        );

        let current_inventory = world.resource::<ColonyInventory>();
        assert_eq!(current_inventory.get_amount(&ItemType::VitalOrgans), 1);
        assert!(world.get_entity(dead_pop).is_err());
    }

    #[test]
    fn test_organ_harvesting_causes_horror() {
        let mut world = World::new();
        world.insert_resource(Events::<OrganHarvestedEvent>::default());

        let normal_pop = world
            .spawn((
                Pop,
                Morale {
                    value: 100.0,
                    ..Default::default()
                },
            ))
            .id();

        let mut traits = Traits::default();
        traits.0.insert(Trait::Psychopath);
        let psycho_pop = world
            .spawn((
                Pop,
                traits,
                Morale {
                    value: 100.0,
                    ..Default::default()
                },
            ))
            .id();

        world
            .resource_mut::<Events<OrganHarvestedEvent>>()
            .send(OrganHarvestedEvent);

        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            apply_harvesting_horror_system,
        );

        let normal_morale = world.get::<Morale>(normal_pop).unwrap();
        let psycho_morale = world.get::<Morale>(psycho_pop).unwrap();

        assert!(normal_morale.value < 100.0);
        assert_eq!(psycho_morale.value, 100.0);
    }
}
