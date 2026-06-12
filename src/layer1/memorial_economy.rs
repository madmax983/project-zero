use crate::layer1::entities::pop::PopDied;
use crate::layer1::social::morale::{MoodModifier, Morale};
use crate::layer1::social::social_stratification::Prestige;
use bevy_ecs::prelude::*;

pub mod ancestral_server {
    use bevy_ecs::prelude::*;

    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::entities::pop::PopDied;

    #[derive(Component)]
    pub struct AncestralServer {
        pub stored_engrams: u32,
    }

    pub fn handle_engram_upload_system(
        mut events: EventReader<PopDied>,
        mut servers: Query<&mut AncestralServer>,
    ) {
        let mut server_opt = servers.iter_mut().next();

        if let Some(ref mut server) = server_opt {
            for _event in events.read() {
                server.stored_engrams += 1;
            }
        } else {
            // If no server, we still need to consume the events so they don't leak
            for _ in events.read() {}
        }
    }

    pub fn scale_server_power_demand_system(
        mut servers: Query<(&AncestralServer, &mut PowerConsumer)>,
    ) {
        for (server, mut consumer) in servers.iter_mut() {
            // Base demand is 10, each engram adds 0.1, scaling quadratically for "exponential" feel
            let engrams_f32 = server.stored_engrams as f32;
            consumer.demand = 10.0 + (engrams_f32 * 0.1) + (engrams_f32 * engrams_f32 * 0.001);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::layer1::energy::PowerConsumer;
        use crate::layer1::entities::pop::{Pop, PopDied};
        use bevy_app::prelude::*;

        // RED Phase Test Setup
        fn setup_app() -> App {
            let mut app = App::new();
            app.add_event::<PopDied>();
            app.add_systems(
                Update,
                (
                    handle_engram_upload_system,
                    scale_server_power_demand_system,
                ),
            );
            app
        }

        #[test]
        fn test_pop_death_uploads_engram() {
            let mut app = setup_app();

            let server = app
                .world_mut()
                .spawn(AncestralServer { stored_engrams: 0 })
                .id();
            let pop = app.world_mut().spawn(Pop).id();

            app.world_mut().send_event(PopDied {
                entity: pop,
                name: "Test Pop".to_string(),
                tick: 1,
                reason: "Old Age".to_string(),
            });
            app.update();

            let server_data = app.world().get::<AncestralServer>(server).unwrap();
            assert_eq!(
                server_data.stored_engrams, 1,
                "Pop death should increase stored engrams on the server"
            );
        }

        #[test]
        fn test_engram_count_increases_power_demand() {
            let mut app = setup_app();

            let server = app
                .world_mut()
                .spawn((
                    AncestralServer {
                        stored_engrams: 100,
                    },
                    PowerConsumer {
                        demand: 10.0,
                        active: true,
                    },
                ))
                .id();

            app.update();

            let consumer = app.world().get::<PowerConsumer>(server).unwrap();
            assert!(
                consumer.demand > 10.0,
                "High engram count should scale the power demand exponentially"
            );
        }
    }
}

#[derive(Component)]
pub struct Relic;

#[derive(Component)]
pub struct MemorialStructure {
    pub dedicated_to: Entity,
}

#[derive(Component)]
pub struct DescendantOf(pub Entity);

pub fn process_pop_deaths_for_relics(
    mut commands: Commands,
    mut events: EventReader<PopDied>,
    query: Query<&Prestige>,
) {
    for event in events.read() {
        if let Ok(prestige) = query.get(event.entity) {
            if prestige.value >= 10 {
                commands.spawn(Relic);
            }
        }
    }
}

pub fn apply_memorial_morale_boost(
    memorials: Query<&MemorialStructure>,
    mut descendants: Query<(&mut Morale, &DescendantOf)>,
) {
    for memorial in memorials.iter() {
        for (mut morale, descendant_of) in descendants.iter_mut() {
            if descendant_of.0 == memorial.dedicated_to {
                morale.add_modifier(MoodModifier {
                    label: "Ancestral Memorial".to_string(),
                    value: 0.1,  // Equivalent to morale boost
                    duration: 1, // transient duration, maintained by structure presence
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;
    use bevy_app::prelude::*;

    #[test]
    fn test_high_prestige_death_generates_relic() {
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.init_resource::<Events<PopDied>>();
        app.add_systems(Update, process_pop_deaths_for_relics);

        let pop = app.world_mut().spawn((Pop, Prestige { value: 10 })).id();

        app.world_mut()
            .resource_mut::<Events<PopDied>>()
            .send(PopDied {
                entity: pop,
                name: "Hero".to_string(),
                tick: 0,
                reason: "Old Age".to_string(),
            });

        app.update();

        // Assert a Relic was generated
        let relics = app.world_mut().query::<&Relic>().iter(app.world()).count();
        assert_eq!(relics, 1);
    }

    #[test]
    fn test_memorial_structure_boosts_descendant_morale() {
        let mut app = App::new();
        app.add_systems(Update, apply_memorial_morale_boost);

        let ancestor_id = Entity::from_raw(1); // Fake ID for testing
        let _memorial = app
            .world_mut()
            .spawn((MemorialStructure {
                dedicated_to: ancestor_id,
            },))
            .id();

        let descendant = app
            .world_mut()
            .spawn((Pop, Morale::default(), DescendantOf(ancestor_id)))
            .id();

        app.update();

        // Assert morale was boosted
        let morale = app.world_mut().get::<Morale>(descendant).unwrap();
        assert!(morale
            .modifiers
            .iter()
            .any(|m| m.label == "Ancestral Memorial"));
    }
}
