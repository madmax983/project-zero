use crate::layer1::architecture::{Building, BuildingType};
use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::health::Health;
use bevy_ecs::prelude::*;

#[derive(Clone, Debug)]
pub struct SponsorshipDeal {
    pub corporation_id: String,
    pub upfront_credits: f32,
    pub required_billboards: u32,
    pub drm_tech_unlocked: Vec<String>,
}

#[derive(Event)]
pub struct AcceptSponsorshipEvent {
    pub deal: SponsorshipDeal,
}

#[derive(Resource)]
pub struct ActiveSponsorship {
    pub deal: SponsorshipDeal,
    pub billboards_built: u32,
    pub breach_timer: i32,
}

#[derive(Component)]
pub struct DrmLocked;

#[derive(Event)]
pub struct RepairBuildingEvent {
    pub target: Entity,
    pub repair_amount: f32,
}

pub fn process_sponsorship_acceptance(
    mut events: EventReader<AcceptSponsorshipEvent>,
    mut commands: Commands,
    mut resources: ResMut<ColonyResources>,
) {
    for event in events.read() {
        resources.credits += event.deal.upfront_credits;
        commands.insert_resource(ActiveSponsorship {
            deal: event.deal.clone(),
            billboards_built: 0,
            breach_timer: 1000,
        });
    }
}

pub fn enforce_sponsorship_requirements(
    sponsorship: Option<ResMut<ActiveSponsorship>>,
    mut resources: ResMut<ColonyResources>,
    billboard_query: Query<&Building>,
) {
    let mut billboard_count = 0;
    for building in billboard_query.iter() {
        if building.building_type == BuildingType::Billboard {
            billboard_count += 1;
        }
    }

    if let Some(mut s) = sponsorship {
        s.billboards_built = billboard_count;
        if s.billboards_built < s.deal.required_billboards {
            s.breach_timer -= 1;
            if s.breach_timer <= 0 {
                resources.credits -= 500.0;
                s.breach_timer = 1000;
            }
        }
    }
}

pub fn handle_repair_requests(
    mut events: EventReader<RepairBuildingEvent>,
    mut buildings: Query<(Option<&DrmLocked>, &mut Health)>,
) {
    for event in events.read() {
        if let Ok((drm_locked, mut health)) = buildings.get_mut(event.target) {
            if drm_locked.is_none() {
                health.current += event.repair_amount;
                if health.current > health.max {
                    health.current = health.max;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accepting_sponsorship_grants_resources() {
        let mut app = bevy_app::App::new();
        app.insert_resource(ColonyResources::default());
        app.world_mut().resource_mut::<ColonyResources>().credits = 0.0;
        app.add_event::<AcceptSponsorshipEvent>();
        app.add_systems(bevy_app::Update, process_sponsorship_acceptance);

        let deal = SponsorshipDeal {
            corporation_id: "lightspeed_cola".to_string(),
            upfront_credits: 5000.0,
            required_billboards: 3,
            drm_tech_unlocked: vec!["nano_med_bay".to_string()],
        };

        app.world_mut().send_event(AcceptSponsorshipEvent { deal });
        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.credits, 5000.0);
    }

    #[test]
    fn test_billboard_requirement_enforced() {
        let mut app = bevy_app::App::new();
        app.insert_resource(ColonyResources::default());
        let deal = SponsorshipDeal {
            corporation_id: "megacorp".to_string(),
            upfront_credits: 1000.0,
            required_billboards: 2,
            drm_tech_unlocked: vec![],
        };
        app.world_mut().insert_resource(ActiveSponsorship {
            deal,
            billboards_built: 0,
            breach_timer: 1, // Ticks until breach if not built
        });

        app.add_systems(bevy_app::Update, enforce_sponsorship_requirements);

        // Advance time without building billboards
        app.update();

        // A breach event or penalty should be triggered
        let penalty = app.world().resource::<ColonyResources>();
        assert!(penalty.credits < 1000.0); // Penalty applied
    }

    #[test]
    fn test_drm_tech_cannot_be_repaired() {
        let mut app = bevy_app::App::new();
        app.add_event::<RepairBuildingEvent>();
        app.add_systems(bevy_app::Update, handle_repair_requests);

        let building_id = app
            .world_mut()
            .spawn((
                Health {
                    current: 10.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                Building {
                    building_type: BuildingType::Hospital,
                },
                DrmLocked,
            ))
            .id();

        // Attempt a repair action
        app.world_mut().send_event(RepairBuildingEvent {
            target: building_id,
            repair_amount: 50.0,
        });

        app.update();

        // Health should not increase because it is DRM locked
        let health = app.world().get::<Health>(building_id).unwrap();
        assert_eq!(health.current, 10.0);
    }
}
