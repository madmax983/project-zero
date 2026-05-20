#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_market_virus_creation() {
        let mut app = App::new();
        // Setup initial state
        app.add_systems(Update, engineer_market_virus_system);

        let biolab = app.world_mut().spawn(BioLab { efficiency: 1.0 }).id();

        // Act: trigger engineering of a virus targeting "Foul Root"
        app.world_mut().spawn(EngineerVirusCommand {
            source_biolab: biolab,
            target_good: GoodType::FoulRoot,
            potency: 5.0,
        });

        app.update();

        // Assert: virus is created in inventory/storage
        let mut query = app.world_mut().query::<&MarketVirus>();
        let virus_count = query.iter(app.world()).count();
        assert_eq!(virus_count, 1, "A market virus should be created");

        let virus = query.single(app.world());
        assert_eq!(virus.target_good, GoodType::FoulRoot);
        assert_eq!(virus.potency, 5.0);
    }

    #[test]
    fn test_inoculate_export_shipment() {
        let mut app = App::new();
        app.add_systems(Update, inoculate_shipment_system);

        let virus = app
            .world_mut()
            .spawn(MarketVirus {
                target_good: GoodType::FoulRoot,
                potency: 5.0,
            })
            .id();

        let shipment = app
            .world_mut()
            .spawn(TradeShipment {
                goods: vec![GoodType::LuxuryTextiles],
                destination: FactionId(1),
                infected_with: None,
            })
            .id();

        app.world_mut().spawn(InoculateCommand { virus, shipment });

        app.update();

        // Assert: shipment is now infected
        let updated_shipment = app.world().get::<TradeShipment>(shipment).unwrap();
        assert!(updated_shipment.infected_with.is_some());
        assert_eq!(
            updated_shipment.infected_with.as_ref().unwrap().target_good,
            GoodType::FoulRoot
        );
        // Virus entity should be consumed/moved
        assert!(app.world().get::<MarketVirus>(virus).is_none());
    }

    #[test]
    fn test_market_demand_manipulation() {
        let mut app = App::new();
        app.add_event::<ShipmentArrivalEvent>();
        app.add_systems(Update, process_infected_imports_system);

        app.world_mut().insert_resource(GalacticMarket {
            demands: vec![(GoodType::FoulRoot, 10.0)].into_iter().collect(),
        });

        let _faction = app.world_mut().spawn(Faction { id: FactionId(1) }).id();

        // Simulate a shipment arriving
        app.world_mut().send_event(ShipmentArrivalEvent {
            shipment: TradeShipment {
                goods: vec![GoodType::LuxuryTextiles],
                destination: FactionId(1),
                infected_with: Some(MarketVirus {
                    target_good: GoodType::FoulRoot,
                    potency: 15.0,
                }),
            },
        });

        app.update();

        // Assert: Market demand for target good has increased
        let market = app.world().resource::<GalacticMarket>();
        let new_demand = market.demands.get(&GoodType::FoulRoot).unwrap();
        assert!(
            *new_demand > 10.0,
            "Market demand should increase due to the virus"
        );
        assert_eq!(*new_demand, 25.0, "Demand increases by virus potency");
    }
}
