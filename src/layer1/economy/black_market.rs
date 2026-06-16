use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
/// Colony-wide statistics tracking economy and law.
pub struct ColonyStats {
    /// Measure of how many luxury needs are currently unmet.
    pub unmet_luxury: u32,
    /// Total corruption level in the colony.
    pub corruption: f32,
}

#[derive(Component)]
/// Marker component for a pop acting as a smuggler.
pub struct Smuggler;

/// Spawns smugglers if unmet luxury needs are high. (Legacy Implementation)
pub fn black_market_spawn_system(mut commands: Commands, stats: Res<ColonyStats>) {
    if stats.unmet_luxury > 50 {
        commands.spawn(Smuggler);
    }
}

/// Increases corruption when smugglers are present.
pub fn smuggler_trade_system(mut stats: ResMut<ColonyStats>, query: Query<&Smuggler>) {
    if !query.is_empty() {
        stats.corruption += 1.0;
    }
}

#[derive(Event)]
pub struct SmugglerArrivalEvent {
    pub intensity: u32,
}

#[derive(Component)]
pub struct DropNode {
    pub stored_metal: f32,
}

#[derive(Component)]
pub struct Desperate;

#[derive(Component)]
pub struct ContrabandUser;

#[derive(Event)]
pub struct ShutdownDropNodeEvent {
    pub node_entity: Entity,
}

/// Handles the arrival of a smuggler, establishing a DropNode (RED/GREEN Phase Spec 1043)
pub fn handle_smuggler_arrival(
    mut commands: Commands,
    mut events: EventReader<SmugglerArrivalEvent>,
) {
    for _ in events.read() {
        commands.spawn(DropNode { stored_metal: 0.0 });
    }
}

/// Allows desperate pops to exchange colony resources (metal) for contraband morale boosts (RED/GREEN Phase Spec 1043)
pub fn pop_smuggling_system(
    mut colony_resources: ResMut<crate::layer1::economy::resources::ColonyResources>,
    mut drop_nodes: Query<&mut DropNode>,
    mut desperate_pops: Query<
        (Entity, &mut crate::layer1::social::morale::Morale),
        With<Desperate>,
    >,
    mut commands: Commands,
) {
    if let Ok(mut node) = drop_nodes.get_single_mut() {
        for (pop_entity, mut morale) in desperate_pops.iter_mut() {
            if colony_resources.metal >= 5.0 && node.stored_metal < 500.0 {
                colony_resources.metal -= 5.0;
                node.stored_metal += 5.0;

                // Morale boost from contraband
                morale.add_modifier(crate::layer1::social::morale::MoodModifier {
                    label: "Contraband High".to_string(),
                    value: 0.2,
                    duration: 100,
                });

                commands.entity(pop_entity).insert(ContrabandUser);
            }
        }
    }
}

pub fn shutdown_drop_node_system(
    mut commands: Commands,
    mut events: EventReader<ShutdownDropNodeEvent>,
    nodes: Query<&DropNode>,
    mut colony_resources: ResMut<crate::layer1::economy::resources::ColonyResources>,
    mut contraband_users: Query<&mut crate::layer1::social::morale::Morale, With<ContrabandUser>>,
) {
    for event in events.read() {
        if let Ok(node) = nodes.get(event.node_entity) {
            // Refund stolen resources up to max capacity
            colony_resources.metal =
                (colony_resources.metal + node.stored_metal).min(colony_resources.max_metal);
            commands.entity(event.node_entity).despawn();

            for mut morale in contraband_users.iter_mut() {
                morale.add_modifier(crate::layer1::social::morale::MoodModifier {
                    label: "Contraband Withdrawal".to_string(),
                    value: -0.5,
                    duration: 500,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_unmet_needs_spawn_smugglers() {
        // Arrange
        let mut world = World::new();
        // Setup colony with massive unmet luxury needs
        world.insert_resource(ColonyStats {
            unmet_luxury: 100,
            ..Default::default()
        });

        // Act
        world.run_system_once(black_market_spawn_system).unwrap();

        // Assert
        let smugglers = world.query::<&Smuggler>().iter(&world).count();
        assert_eq!(smugglers, 1, "A smuggler should spawn due to unmet needs");
    }

    #[test]
    fn test_smuggler_increases_corruption() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(ColonyStats {
            corruption: 0.0,
            ..Default::default()
        });
        let _smuggler = world.spawn(Smuggler).id();

        // Act
        world.run_system_once(smuggler_trade_system).unwrap(); // Mock trade

        // Assert
        let stats = world.get_resource::<ColonyStats>().unwrap();
        assert!(
            stats.corruption > 0.0,
            "Corruption should increase after smuggler trade"
        );
    }

    use crate::layer1::economy::resources::ColonyResources;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::social::morale::Morale;
    use bevy::prelude::{App, Update};

    #[test]
    fn test_smugglers_establish_drop_node() {
        // Arrange
        let mut app = App::new();
        app.add_event::<SmugglerArrivalEvent>();
        app.add_systems(Update, handle_smuggler_arrival);

        // Act
        app.world_mut()
            .send_event(SmugglerArrivalEvent { intensity: 1 });
        app.update();

        // Assert
        let mut query = app.world_mut().query::<&DropNode>();
        assert_eq!(
            query.iter(app.world()).count(),
            1,
            "A Drop Node should be established upon smuggler arrival"
        );
    }

    #[test]
    fn test_desperate_pop_exchanges_resources_for_contraband() {
        // Arrange
        let mut app = App::new();
        let res = ColonyResources {
            metal: 50.0,
            ..Default::default()
        };
        app.insert_resource(res);

        app.add_systems(Update, pop_smuggling_system);

        let drop_node_entity = app.world_mut().spawn(DropNode { stored_metal: 0.0 }).id();
        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                Morale::default(),
                Desperate, // Tag indicating they are likely to smuggle
            ))
            .id();

        // Act
        app.update();

        // Assert
        let colony_resources = app.world().resource::<ColonyResources>();
        assert!(
            colony_resources.metal < 50.0,
            "Colony alloys should have been stolen"
        );

        let drop_node = app.world().get::<DropNode>(drop_node_entity).unwrap();
        assert!(
            drop_node.stored_metal > 0.0,
            "Drop Node should have accumulated stolen alloys"
        );

        let pop_morale = app.world().get::<Morale>(pop_entity).unwrap();
        assert!(
            pop_morale
                .modifiers
                .iter()
                .any(|m| m.label == "Contraband High"),
            "Pop should have received a morale boost from contraband"
        );
    }

    #[test]
    fn test_shutting_down_drop_node_returns_resources_and_crashes_morale() {
        // Arrange
        let mut app = App::new();
        let res = ColonyResources {
            metal: 50.0,
            max_metal: 1000.0,
            ..Default::default()
        };
        app.insert_resource(res);
        app.add_event::<ShutdownDropNodeEvent>();
        app.add_systems(Update, shutdown_drop_node_system);

        let drop_node_entity = app
            .world_mut()
            .spawn(DropNode {
                stored_metal: 100.0,
            })
            .id();
        let pop_entity = app
            .world_mut()
            .spawn((Pop, Morale::default(), ContrabandUser))
            .id();

        // Act
        app.world_mut().send_event(ShutdownDropNodeEvent {
            node_entity: drop_node_entity,
        });
        app.update();

        // Assert
        assert!(
            app.world().get::<DropNode>(drop_node_entity).is_none(),
            "Drop Node should be destroyed"
        );

        let colony_resources = app.world().resource::<ColonyResources>();
        assert_eq!(
            colony_resources.metal, 150.0,
            "Stored alloys should be returned to the colony"
        );

        let pop_morale = app.world().get::<Morale>(pop_entity).unwrap();
        assert!(
            pop_morale
                .modifiers
                .iter()
                .any(|m| m.label == "Contraband Withdrawal"),
            "Pop morale should crash when their contraband supply is cut off"
        );
    }
}
