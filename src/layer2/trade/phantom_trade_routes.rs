use bevy_ecs::prelude::*;
use crate::layer1::economy::resources::{ColonyResources, ResourceType};
use crate::layer1::social::morale::Morale;
#[cfg(test)]
use crate::layer1::entities::pop::Pop;

#[derive(Component)]
pub struct PhantomTradeRoute {
    pub required_resource: ResourceType,
    pub amount: f32,
}

#[derive(Event)]
pub struct FixPhantomRouteEvent {
    pub entity: Entity,
}

pub fn process_phantom_trade_routes_system(
    mut resources: Option<ResMut<ColonyResources>>,
    routes: Query<&PhantomTradeRoute>,
    mut morale_query: Query<&mut Morale>,
) {
    if let Some(ref mut res) = resources {
        let mut food_to_consume = 0.0;
        for route in routes.iter() {
            if route.required_resource == ResourceType::Food {
                food_to_consume += route.amount;
            }
        }

        if food_to_consume > 0.0 && res.try_consume(ResourceType::Food, food_to_consume) {
            for mut morale in morale_query.iter_mut() {
                morale.modifiers.push(crate::layer1::social::morale::MoodModifier {
                    value: 0.05,
                    duration: 100,
                    label: "Purpose: Fulfilling Quota".to_string(),
                });
            }
        }
    }
}

pub fn fix_phantom_route_reaction_system(
    mut events: EventReader<FixPhantomRouteEvent>,
    mut morale_query: Query<&mut Morale>,
    mut commands: Commands,
) {
    let mut events_processed = false;

    for event in events.read() {
        events_processed = true;
        if let Some(mut cmds) = commands.get_entity(event.entity) {
            cmds.despawn();
        }
    }

    if events_processed {
        for mut morale in morale_query.iter_mut() {
            morale.modifiers.push(crate::layer1::social::morale::MoodModifier {
                value: -0.25,
                duration: 500,
                label: "Despair: Pointless Labor".to_string(),
            });
        }
    }
}

pub struct PhantomTradeRoutePlugin;

impl bevy_app::Plugin for PhantomTradeRoutePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_event::<FixPhantomRouteEvent>()
           .add_systems(bevy_app::Update, (process_phantom_trade_routes_system, fix_phantom_route_reaction_system));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phantom_trade_route_consumes_resources_and_boosts_morale() {
        let mut app = bevy_app::App::new();
        app.add_plugins((bevy::MinimalPlugins, PhantomTradeRoutePlugin));

        // Setup resources
        let mut resources = ColonyResources::default();
        resources.food = 100.0;
        app.insert_resource(resources);

        // Setup phantom trade route (destination destroyed)
        app.world_mut().spawn(PhantomTradeRoute { required_resource: ResourceType::Food, amount: 10.0 });

        // Setup pop with Morale
        let pop_id = app.world_mut().spawn((Pop, Morale::default())).id();

        // Run the system
        app.update();

        // Check resources consumed
        let current_resources = app.world().get_resource::<ColonyResources>().unwrap();
        assert_eq!(current_resources.food, 90.0, "Phantom route should consume resources");

        // Check morale boosted
        let morale = app.world().get::<Morale>(pop_id).unwrap();
        let has_boost = morale.modifiers.iter().any(|m| m.value > 0.0 && m.label == "Purpose: Fulfilling Quota");
        assert!(has_boost, "Pop should gain morale from fulfilling the phantom quota");
    }

    #[test]
    fn test_fixing_phantom_trade_route_shatters_morale() {
        let mut app = bevy_app::App::new();
        app.add_plugins((bevy::MinimalPlugins, PhantomTradeRoutePlugin));

        // Setup phantom route
        let route_id = app.world_mut().spawn(PhantomTradeRoute { required_resource: ResourceType::Food, amount: 10.0 }).id();

        // Setup pop
        let pop_id = app.world_mut().spawn((Pop, Morale::default())).id();

        // Simulate player action to fix/delete the route
        app.world_mut().send_event(FixPhantomRouteEvent { entity: route_id });

        // Run the reaction system
        app.update();

        // Check route removed
        assert!(app.world().get::<PhantomTradeRoute>(route_id).is_none(), "Route should be deleted");

        // Check morale shattered
        let morale = app.world().get::<Morale>(pop_id).unwrap();
        let has_penalty = morale.modifiers.iter().any(|m| m.value < 0.0 && m.label == "Despair: Pointless Labor");
        assert!(has_penalty, "Fixing the route should cause a severe morale penalty");
    }
}
