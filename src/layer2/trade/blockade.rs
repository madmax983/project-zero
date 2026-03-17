use bevy::prelude::*;

#[derive(Resource)]
pub struct ColonyDebt {
    pub amount: f32,
    pub threshold: f32,
}

impl Default for ColonyDebt {
    fn default() -> Self {
        Self {
            amount: 0.0,
            threshold: 50_000.0,
        }
    }
}

#[derive(Component)]
pub struct CollectionSphereBlockade {
    pub active: bool,
}

#[derive(Event)]
pub struct TradeShipArrivalEvent {
    pub cargo_value: f32,
    pub faction: String,
}

pub fn debt_blockade_system(
    mut commands: Commands,
    debt: Res<ColonyDebt>,
    query: Query<Entity, With<CollectionSphereBlockade>>,
) {
    let is_critical = debt.amount >= debt.threshold;
    let has_blockade = !query.is_empty();

    if is_critical && !has_blockade {
        commands.spawn(CollectionSphereBlockade { active: true });
    } else if !is_critical && has_blockade {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn blockade_interception_system(
    mut debt: ResMut<ColonyDebt>,
    blockade_query: Query<&CollectionSphereBlockade>,
    mut trade_events: EventReader<TradeShipArrivalEvent>,
) {
    let has_blockade = !blockade_query.is_empty();

    for event in trade_events.read() {
        if !has_blockade {
            continue; // No blockade, ships pass normally (but we still drain the event queue)
        }

        // Intercept ship cargo to pay debt
        debt.amount -= event.cargo_value;
        if debt.amount < 0.0 {
            debt.amount = 0.0;
        }
        // In a real system, we'd also prevent the cargo from reaching the colony here
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ColonyDebt>();
        app.add_event::<TradeShipArrivalEvent>();
        app.add_systems(Update, (debt_blockade_system, blockade_interception_system));
        app
    }

    #[test]
    fn test_blockade_spawns_when_debt_critical() {
        let mut app = setup_test_app();

        app.world_mut().resource_mut::<ColonyDebt>().amount = 100_000.0;
        app.world_mut().resource_mut::<ColonyDebt>().threshold = 50_000.0;

        app.update();

        let mut query = app.world_mut().query::<&CollectionSphereBlockade>();
        let blockades = query.iter(app.world()).count();
        assert_eq!(blockades, 1, "A Collection Sphere should spawn when debt exceeds threshold");
    }

    #[test]
    fn test_blockade_intercepts_trade_ships() {
        let mut app = setup_test_app();

        // Setup blockade
        app.world_mut().spawn(CollectionSphereBlockade { active: true });

        // Setup a pending trade ship arrival
        app.world_mut().send_event(TradeShipArrivalEvent {
            cargo_value: 5000.0,
            faction: "Megacorp".to_string(),
        });

        // Debt before interception
        app.world_mut().resource_mut::<ColonyDebt>().amount = 50_000.0;

        app.update();

        // The event should have been intercepted, reducing debt instead of delivering cargo
        let debt = app.world().resource::<ColonyDebt>().amount;
        assert_eq!(debt, 45_000.0, "The trade ship's cargo should be siphoned to pay debt");
    }

    #[test]
    fn test_blockade_lifts_when_debt_paid() {
        let mut app = setup_test_app();

        // Setup blockade and zero debt
        let blockade_entity = app.world_mut().spawn(CollectionSphereBlockade { active: true }).id();
        app.world_mut().resource_mut::<ColonyDebt>().amount = 0.0;
        app.world_mut().resource_mut::<ColonyDebt>().threshold = 50_000.0;

        app.update();

        // The blockade should despawn or deactivate
        assert!(app.world().get::<CollectionSphereBlockade>(blockade_entity).is_none(), "Blockade should despawn when debt is cleared");
    }
}
