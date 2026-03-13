#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::social::pen_pals::{RemoteBond, update_pen_pals_system};
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::factions::{FactionId, FactionMember};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::shared::log::MessageLog;

    #[test]
    fn test_remote_bond_formation() {
        let mut world = World::new();
        let pop = world.spawn(Pop).id();

        // Simulate working at Comms
        world.spawn(RemoteBond {
            local_pop: pop,
            foreign_faction: FactionId::MinersGuild, // Mock foreign faction
            affinity: 10.0,
        });

        // Verify bond exists (query)
        let count = world.query::<&RemoteBond>().iter(&world).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_intel_gain_from_bond() {
        let mut world = World::new();
        let mut res = ColonyResources::zeroed();
        res.max_knowledge = 100.0;
        world.insert_resource(res);

        let pop = world.spawn(Pop).id();

        let library = world.spawn(Building {
            building_type: BuildingType::Library,
        }).id();

        world.entity_mut(pop).insert(AssignedTo {
            entity: library,
            assignment_type: AssignmentType::LibraryWorker,
        });

        world.spawn(RemoteBond {
            local_pop: pop,
            foreign_faction: FactionId::MinersGuild,
            affinity: 50.0, // High affinity = more intel
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_pen_pals_system);

        let mut knowledge_gained = false;
        // With 10% espionage, 90% chance to gain. 1000 iterations is plenty.
        for _ in 0..1000 {
            let before = world.resource::<ColonyResources>().knowledge;
            schedule.run(&mut world);
            let after = world.resource::<ColonyResources>().knowledge;
            if after > before {
                knowledge_gained = true;
                break;
            }
        }

        assert!(knowledge_gained, "Should have gained knowledge from pen pal");
    }

    #[test]
    fn test_ethics_shift() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::zeroed());
        let pop = world.spawn((
            Pop,
            FactionMember { faction_id: Some(FactionId::MinersGuild) },
        )).id();

        let library = world.spawn(Building {
            building_type: BuildingType::Library,
        }).id();

        world.entity_mut(pop).insert(AssignedTo {
            entity: library,
            assignment_type: AssignmentType::LibraryWorker,
        });

        world.spawn(RemoteBond {
            local_pop: pop,
            foreign_faction: FactionId::FarmersGuild, // Different ethic
            affinity: 100.0, // Max affinity forces shift
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_pen_pals_system);
        schedule.run(&mut world);

        let member = world.get::<FactionMember>(pop).unwrap();
        assert_eq!(member.faction_id, Some(FactionId::FarmersGuild));
    }

    #[test]
    fn test_espionage() {
        let mut world = World::new();
        let mut res = ColonyResources::zeroed();
        res.knowledge = 100.0;
        res.max_knowledge = 100.0;
        world.insert_resource(res);
        world.insert_resource(MessageLog::default());

        let pop = world.spawn(Pop).id();

        let library = world.spawn(Building {
            building_type: BuildingType::Library,
        }).id();

        world.entity_mut(pop).insert(AssignedTo {
            entity: library,
            assignment_type: AssignmentType::LibraryWorker,
        });

        world.spawn(RemoteBond {
            local_pop: pop,
            foreign_faction: FactionId::MinersGuild,
            affinity: 50.0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_pen_pals_system);

        let mut esp_found = false;
        // Increase knowledge massively so we don't hit 0 quickly and false negative
        for _ in 0..1000 {
            world.resource_mut::<ColonyResources>().knowledge = 100.0;
            let before = world.resource::<ColonyResources>().knowledge;
            schedule.run(&mut world);
            let after = world.resource::<ColonyResources>().knowledge;
            if after < before {
                esp_found = true;
                break;
            }
        }

        assert!(esp_found, "Should have triggered espionage and lost intel");
    }

    #[test]
    fn test_firewall_comms_policy() {
        use bevy_ecs::prelude::*;
        use crate::layer1::edicts::{ColonyPolicies, Policy};
        use crate::layer1::pop::Pop;
        use crate::layer1::resources::ColonyResources;
        use crate::layer1::factions::{FactionId, FactionMember};
        use crate::layer1::building::{Building, BuildingType};
        use crate::layer1::actions::{AssignedTo, AssignmentType};
        use crate::layer1::social::pen_pals::{RemoteBond, update_pen_pals_system};

        let mut world = World::new();
        let mut policies = ColonyPolicies::default();
        policies.toggle(Policy::FirewallComms);
        world.insert_resource(policies);

        let mut res = ColonyResources::zeroed();
        res.knowledge = 100.0;
        world.insert_resource(res);

        let pop = world.spawn((
            Pop,
            FactionMember { faction_id: Some(FactionId::MinersGuild) },
        )).id();

        let library = world.spawn(Building {
            building_type: BuildingType::Library,
        }).id();

        world.entity_mut(pop).insert(AssignedTo {
            entity: library,
            assignment_type: AssignmentType::LibraryWorker,
        });

        world.spawn(RemoteBond {
            local_pop: pop,
            foreign_faction: FactionId::FarmersGuild,
            affinity: 100.0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_pen_pals_system);
        schedule.run(&mut world);

        let final_res = world.resource::<ColonyResources>();
        assert_eq!(final_res.knowledge, 100.0);

        let member = world.get::<FactionMember>(pop).unwrap();
        assert_eq!(member.faction_id, Some(FactionId::MinersGuild));
    }
}
