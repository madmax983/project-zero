#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::resources::ColonyResources;
    use scale::layer1::pop::Pop;
    use scale::layer1::stress::StressTracker;
    use scale::layer2::trade::biomass_tariff::{TradeDeal, process_biomass_tariff_system};

    #[test]
    fn test_biomass_tariff_seam_crops() {
        let mut app = App::new();

        let mut stash = ColonyResources::default();
        stash.food = 500.0;
        stash.wood = 0.0;
        app.insert_resource(stash);
        app.init_resource::<Events<TradeDeal>>();

        app.world_mut().send_event(TradeDeal {
            offered_crops: 500.0,
            offered_pops: 0,
            requested_resin: 100.0,
        });

        app.add_systems(Update, process_biomass_tariff_system);
        app.update();

        let final_stash = app.world().resource::<ColonyResources>();
        assert_eq!(final_stash.food, 0.0, "Crops consumed for tariff");
        assert_eq!(final_stash.wood, 100.0, "Bio-resin received");
    }

    #[test]
    fn test_biomass_tariff_seam_pops() {
        let mut app = App::new();

        app.world_mut().spawn((Pop, StressTracker { accumulated_stress: 10.0 }));

        let mut stash = ColonyResources::default();
        stash.wood = 0.0;
        app.insert_resource(stash);
        app.init_resource::<Events<TradeDeal>>();

        app.world_mut().send_event(TradeDeal {
            offered_crops: 0.0,
            offered_pops: 1,
            requested_resin: 1000.0,
        });

        app.add_systems(Update, process_biomass_tariff_system);
        app.update();

        let final_stash = app.world().resource::<ColonyResources>();
        assert_eq!(final_stash.wood, 1000.0, "Bio-resin received");

        // The single pop should have been despawned.
        let mut pop_query = app.world_mut().query::<&Pop>();
        assert_eq!(pop_query.iter(app.world()).count(), 0, "Pop was traded away");
    }
}
