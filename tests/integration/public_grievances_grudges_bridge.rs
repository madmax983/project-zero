#[cfg(test)]
mod integration_tests {
    use bevy_app::prelude::*;
    use scale::layer1::social::grievances::PostGrievanceEvent;
    use scale::layer1::social::inherited_grudges::GrudgeList;
    use scale::layer1::core::integration::public_grievance_grudge_bridge;

    #[test]
    fn test_grievance_creates_grudge() {
        let mut app = App::new();
        app.add_event::<PostGrievanceEvent>();
        app.add_systems(Update, public_grievance_grudge_bridge);

        let poster = app.world_mut().spawn_empty().id();
        let target = app.world_mut().spawn_empty().id();
        let board = app.world_mut().spawn_empty().id();

        app.world_mut().send_event(PostGrievanceEvent {
            poster,
            target,
            board,
            impact: -10.0,
        });

        app.update();

        let grudges = app.world().get::<GrudgeList>(poster).unwrap();
        assert_eq!(grudges.0.len(), 1);
        assert_eq!(grudges.0[0].target_entity, target);
        assert_eq!(grudges.0[0].intensity, 10.0);
    }
}
