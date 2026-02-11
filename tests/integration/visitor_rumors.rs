#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::map::GridPosition;
    use scale::layer1::pop::Pop;
    use scale::layer1::rumor::{exchange_rumors_system, Knowledge, Rumor, RumorTopic};
    use scale::layer1::social::{AffinityChange, Tavern};
    use scale::layer1::visitor::{spawn_visitor_system, Visitor, VisitorSource};
    use scale::shared::time::SimulationTime;

    #[test]
    fn test_visitor_learns_rumors_in_tavern() {
        let mut world = World::new();
        world.init_resource::<Events<AffinityChange>>();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(VisitorSource {
            spawn_points: vec![GridPosition { x: 0, y: 0 }],
            next_spawn_tick: 0,
        });

        // 1. Spawn a Tavern
        let tavern_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Tavern,
                },
                GridPosition { x: 5, y: 5 },
                Tavern::default(),
            ))
            .id();

        // 2. Spawn a Pop with a Rumor
        let rumor = Rumor {
            topic: RumorTopic::EventNews("Colony Founded".to_string()),
            source: Entity::PLACEHOLDER,
            timestamp: 1,
            strength: 1.0,
        };

        let pop_entity = world
            .spawn((
                Pop,
                Knowledge {
                    known_rumors: vec![rumor.clone()],
                },
            ))
            .id();

        // 3. Spawn a Visitor using the system (Simulate arrival)
        // Run spawn system once
        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_visitor_system);
        schedule.run(&mut world);

        // Find the spawned visitor
        let visitor_entity = world
            .query_filtered::<Entity, With<Visitor>>()
            .single(&world);

        // 4. Put both in Tavern (Simulate handle_socialize)
        let mut tavern = world.get_mut::<Tavern>(tavern_entity).unwrap();
        tavern.visitors.push(pop_entity);
        tavern.visitors.push(visitor_entity);

        // 5. Run exchange_rumors_system
        let mut schedule = Schedule::default();
        schedule.add_systems(exchange_rumors_system);
        schedule.run(&mut world);

        // 6. Assert Visitor knows the rumor
        // This will panic if Visitor lacks Knowledge component
        let knowledge = world.get::<Knowledge>(visitor_entity);
        assert!(knowledge.is_some(), "Visitor should have Knowledge component");
        let knowledge = knowledge.unwrap();
        assert!(
            knowledge.knows(&rumor.topic),
            "Visitor should have learned the rumor"
        );
    }

    #[test]
    fn test_visitor_shares_rumors_in_tavern() {
        let mut world = World::new();
        world.init_resource::<Events<AffinityChange>>();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(VisitorSource {
            spawn_points: vec![GridPosition { x: 0, y: 0 }],
            next_spawn_tick: 0,
        });

        // 1. Spawn a Tavern
        let tavern_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Tavern,
                },
                GridPosition { x: 5, y: 5 },
                Tavern::default(),
            ))
            .id();

        // 2. Spawn a Pop (Knows nothing)
        let pop_entity = world.spawn((Pop, Knowledge::default())).id();

        // 3. Spawn a Visitor
        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_visitor_system);
        schedule.run(&mut world);

        let visitor_entity = world
            .query_filtered::<Entity, With<Visitor>>()
            .single(&world);

        // Inject rumor into Visitor
        let rumor = Rumor {
            topic: RumorTopic::EventNews("News from Outside".to_string()),
            source: Entity::PLACEHOLDER,
            timestamp: 1,
            strength: 1.0,
        };

        // We check if Knowledge exists, if not we add it (to simulate the fix working or proving logic)
        // If we want to strictly test the fix, we shouldn't manually add Knowledge if it's supposed to be there.
        // But for this test, we want to prove that IF they have knowledge, the system works.
        // The first test proves they HAVE knowledge.
        // So I'll manually add it here to ensure this test passes logic-wise even if spawn is broken,
        // BUT wait, if I want to verify the fix, I should rely on spawn.
        // However, I can't inject the rumor if Knowledge component is missing.
        // So I'll just Insert the Knowledge component with the rumor.
        // This effectively overwrites/adds it.
        world.entity_mut(visitor_entity).insert(Knowledge {
            known_rumors: vec![rumor.clone()],
        });

        // 4. Put both in Tavern
        let mut tavern = world.get_mut::<Tavern>(tavern_entity).unwrap();
        tavern.visitors.push(pop_entity);
        tavern.visitors.push(visitor_entity);

        // 5. Run exchange
        let mut schedule = Schedule::default();
        schedule.add_systems(exchange_rumors_system);
        schedule.run(&mut world);

        // 6. Assert Pop knows the rumor
        let knowledge = world.get::<Knowledge>(pop_entity).unwrap();
        assert!(
            knowledge.knows(&rumor.topic),
            "Pop should have learned the rumor from Visitor"
        );
    }
}
