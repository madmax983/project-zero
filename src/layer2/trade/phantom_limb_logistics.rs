use bevy::prelude::*;
use crate::layer1::economy::resources::{ColonyResources, ResourceType};
use crate::layer1::social::factions::FactionId;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(pub u32);

#[derive(Component)]
pub struct PhantomSupplyChain {
    pub target_node: NodeId,
    pub resource: ResourceType,
    pub amount: f32,
    pub ticks_until_drop: u32,
}

#[derive(Component)]
pub struct PhantomDrop {
    pub node: NodeId,
    pub resource: ResourceType,
    pub amount: f32,
    pub faction_owner: FactionId,
}

#[derive(Event)]
pub struct InterceptDropEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct AuditRiskEvent {
    pub target_faction: FactionId,
    pub severity: f32,
}

pub fn phantom_limb_logistics_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut PhantomSupplyChain)>,
) {
    for (entity, mut chain) in query.iter_mut() {
        if chain.ticks_until_drop > 0 {
            chain.ticks_until_drop -= 1;
        } else {
            commands.spawn(PhantomDrop {
                node: chain.target_node,
                resource: chain.resource,
                amount: chain.amount,
                faction_owner: FactionId::Unaligned, // Mock faction
            });
            commands.entity(entity).despawn();
        }
    }
}

pub fn intercept_phantom_drop_system(
    mut commands: Commands,
    mut events: EventReader<InterceptDropEvent>,
    mut resources: ResMut<ColonyResources>,
    mut audit_events: EventWriter<AuditRiskEvent>,
    drop_query: Query<&PhantomDrop>,
) {
    for event in events.read() {
        if let Ok(drop) = drop_query.get(event.entity) {
            resources.add_resource(&drop.resource, drop.amount);
            audit_events.send(AuditRiskEvent {
                target_faction: drop.faction_owner,
                severity: 0.1,
            });
            commands.entity(event.entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::resources::{ColonyResources, ResourceType};
    use crate::layer1::social::factions::FactionId;

    #[test]
    fn test_phantom_drop_spawns_at_destroyed_colony() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, phantom_limb_logistics_system);

        // Setup a trade route targeting a non-existent colony node
        app.world_mut().spawn(PhantomSupplyChain {
            target_node: NodeId(1),
            resource: ResourceType::Food,
            amount: 50.0,
            ticks_until_drop: 1,
        });

        // Tick 1
        app.update();
        // Tick 2 (should drop)
        app.update();

        // Check if a PhantomDrop entity was created at the node
        let mut drop_found = false;
        for drop in app.world_mut().query::<&PhantomDrop>().iter(app.world()) {
            if drop.node == NodeId(1) { drop_found = true; }
        }
        assert!(drop_found, "Phantom Drop should spawn at the destroyed colony's location");
    }

    #[test]
    fn test_intercept_phantom_drop() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<InterceptDropEvent>();
        app.add_event::<AuditRiskEvent>();
        app.add_systems(Update, intercept_phantom_drop_system);

        // Setup local colony resources
        app.insert_resource(ColonyResources::default());

        // Setup a drop
        let drop_id = app.world_mut().spawn(PhantomDrop {
            node: NodeId(1),
            resource: ResourceType::Food,
            amount: 50.0,
            faction_owner: FactionId::MinersGuild,
        }).id();

        // Simulate player action to intercept
        app.world_mut().send_event(InterceptDropEvent { entity: drop_id });

        // Process interception
        app.update();

        // Check resources gained
        let current_resources = app.world().get_resource::<ColonyResources>().unwrap();
        assert_eq!(current_resources.food, 50.0, "Intercepting should grant the resources");

        // Check drop despawned
        assert!(app.world().get::<PhantomDrop>(drop_id).is_none(), "Drop should be removed after interception");
    }
}
