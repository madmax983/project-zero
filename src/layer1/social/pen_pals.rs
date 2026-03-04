use crate::layer1::edicts::{ColonyPolicies, Policy};
use crate::layer1::factions::{FactionId, FactionMember};
use crate::layer1::pop::Job;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_types::AssignmentType;
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct RemoteBond {
    pub local_pop: Entity,
    pub foreign_faction: FactionId,
    pub affinity: f32,
    pub is_spy: bool,
}

#[derive(Event)]
pub struct PenPalEvent {
    pub pop: Entity,
    pub message: String,
}

pub fn update_pen_pals_system(
    bonds: Query<&RemoteBond>,
    mut resources: ResMut<ColonyResources>,
    mut pops: Query<(&mut FactionMember, Option<&Job>)>,
    policies: Option<Res<ColonyPolicies>>,
) {
    let firewall_active = policies.map_or(false, |p| p.is_active(Policy::FirewallComms));

    for bond in bonds.iter() {
        if let Ok((mut member, job)) = pops.get_mut(bond.local_pop) {
            // Must be working at Library or Observatory (representing Comms/Research)
            let is_at_comms = job.map_or(false, |w| {
                w.job_type == AssignmentType::LibraryWorker
                    || w.job_type == AssignmentType::ObservatoryWorker
            });

            if !is_at_comms {
                continue;
            }

            if firewall_active {
                // Edict active: no Intel, no Ethics shift
                continue;
            }

            if bond.is_spy {
                // Espionage risk
                let mut rng = rand::thread_rng();
                if rng.gen_bool(0.1) {
                    // 10% chance per tick to lose knowledge
                    resources.knowledge = (resources.knowledge - 5.0).max(0.0);
                }
            } else {
                // Gain Intel (Knowledge)
                if bond.affinity > 0.0 {
                    resources.knowledge += 0.1 * (bond.affinity / 100.0);
                }
            }

            // Ethics Shift check
            if bond.affinity > 80.0 {
                member.faction_id = Some(bond.foreign_faction);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::edicts::{ColonyPolicies, Policy};
    use crate::layer1::factions::{FactionId, FactionMember};
    use crate::layer1::pop::{Job, Pop};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::social::pen_pals::{update_pen_pals_system, RemoteBond};
    use crate::layer1::utility_types::AssignmentType;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_remote_bond_formation() {
        let mut world = World::new();
        let pop = world.spawn(Pop).id();

        world.spawn(RemoteBond {
            local_pop: pop,
            foreign_faction: FactionId::FarmersGuild,
            affinity: 10.0,
            is_spy: false,
        });

        let count = world.query::<&RemoteBond>().iter(&world).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_intel_gain_from_bond() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(ColonyPolicies::default());

        let pop = world
            .spawn((
                Pop,
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: AssignmentType::LibraryWorker,
                },
                FactionMember { faction_id: None },
            ))
            .id();
        world.spawn(RemoteBond {
            local_pop: pop,
            foreign_faction: FactionId::FarmersGuild,
            affinity: 50.0,
            is_spy: false,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_pen_pals_system);
        schedule.run(&mut world);

        let res = world.resource::<ColonyResources>();
        assert!(res.knowledge > 0.0);
    }

    #[test]
    fn test_intel_gain_requires_comms_console() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(ColonyPolicies::default());

        let pop = world.spawn((Pop, FactionMember { faction_id: None })).id(); // No Job
        world.spawn(RemoteBond {
            local_pop: pop,
            foreign_faction: FactionId::FarmersGuild,
            affinity: 50.0,
            is_spy: false,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_pen_pals_system);
        schedule.run(&mut world);

        let res = world.resource::<ColonyResources>();
        assert_eq!(
            res.knowledge, 0.0,
            "Should not gain knowledge if not working at comms"
        );
    }

    #[test]
    fn test_ethics_shift() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(ColonyPolicies::default());
        let pop = world
            .spawn((
                Pop,
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: AssignmentType::LibraryWorker,
                },
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
            ))
            .id();

        world.spawn(RemoteBond {
            local_pop: pop,
            foreign_faction: FactionId::FarmersGuild,
            affinity: 100.0,
            is_spy: false,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_pen_pals_system);
        schedule.run(&mut world);

        let member = world.get::<FactionMember>(pop).unwrap();
        assert_eq!(member.faction_id, Some(FactionId::FarmersGuild));
    }

    #[test]
    fn test_firewall_comms_policy() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        let mut policies = ColonyPolicies::default();
        policies.active_policies.insert(Policy::FirewallComms);
        world.insert_resource(policies);

        let _pop = world
            .spawn((
                Pop,
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: AssignmentType::LibraryWorker,
                },
                FactionMember { faction_id: None },
            ))
            .id();
        let pop = world
            .spawn((
                Pop,
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: AssignmentType::LibraryWorker,
                },
                FactionMember { faction_id: None },
            ))
            .id();
        world.spawn(RemoteBond {
            local_pop: pop,
            foreign_faction: FactionId::FarmersGuild,
            affinity: 50.0,
            is_spy: false,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_pen_pals_system);
        schedule.run(&mut world);

        let res = world.resource::<ColonyResources>();
        assert_eq!(
            res.knowledge, 0.0,
            "Knowledge gain should be blocked by Firewall"
        );
    }

    #[test]
    fn test_espionage_loss() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.knowledge = 100.0;
        world.insert_resource(resources);
        world.insert_resource(ColonyPolicies::default());

        let pop = world
            .spawn((
                Pop,
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: AssignmentType::LibraryWorker,
                },
                FactionMember { faction_id: None },
            ))
            .id();
        world.spawn(RemoteBond {
            local_pop: pop,
            foreign_faction: FactionId::FarmersGuild,
            affinity: 50.0,
            is_spy: true, // Spy!
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_pen_pals_system);
        // Run several times to hit the chance
        for _ in 0..100 {
            schedule.run(&mut world);
        }

        let res = world.resource::<ColonyResources>();
        assert!(
            res.knowledge < 100.0,
            "Knowledge should be lost due to espionage"
        );
    }
}
