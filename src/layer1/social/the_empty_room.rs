use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct SanctuaryZone {
    pub active: bool,
}

#[derive(Component)]
pub struct Clutter;

#[derive(Component)]
pub struct Stress {
    pub value: f32,
}

#[derive(Component)]
pub struct InZone {
    pub zone_entity: Entity,
}

pub fn evaluate_sanctuary_emptiness(
    mut q_zones: Query<(Entity, &mut SanctuaryZone)>,
    q_clutter: Query<&InZone, With<Clutter>>,
) {
    for (zone_entity, mut sanctuary) in q_zones.iter_mut() {
        let has_clutter = q_clutter
            .iter()
            .any(|in_zone| in_zone.zone_entity == zone_entity);
        sanctuary.active = !has_clutter;
    }
}

pub fn apply_sanctuary_stress_relief(
    q_zones: Query<&SanctuaryZone>,
    mut q_pops: Query<(&mut Stress, &InZone)>,
) {
    for (mut stress, in_zone) in q_pops.iter_mut() {
        if let Ok(sanctuary) = q_zones.get(in_zone.zone_entity) {
            if sanctuary.active {
                stress.value = (stress.value - 1.0).max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_sanctuary_active_when_empty() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_sanctuary_emptiness);

        let zone = app.world_mut().spawn(SanctuaryZone { active: false }).id();

        app.update();

        let sanctuary = app.world().get::<SanctuaryZone>(zone).unwrap();
        assert!(
            sanctuary.active,
            "Sanctuary should be active when no clutter is present"
        );
    }

    #[test]
    fn test_sanctuary_deactivated_by_clutter() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_sanctuary_emptiness);

        let zone = app.world_mut().spawn(SanctuaryZone { active: true }).id();
        app.world_mut()
            .spawn((Clutter, InZone { zone_entity: zone }));

        app.update();

        let sanctuary = app.world().get::<SanctuaryZone>(zone).unwrap();
        assert!(
            !sanctuary.active,
            "Sanctuary should deactivate if clutter is inside"
        );
    }

    #[test]
    fn test_stress_relief_in_active_sanctuary() {
        let mut app = App::new();
        app.add_systems(Update, apply_sanctuary_stress_relief);

        let zone = app.world_mut().spawn(SanctuaryZone { active: true }).id();
        let pop = app
            .world_mut()
            .spawn((Stress { value: 50.0 }, InZone { zone_entity: zone }))
            .id();

        app.update();

        let stress = app.world().get::<Stress>(pop).unwrap();
        assert!(
            stress.value < 50.0,
            "Pop should lose stress in an active sanctuary"
        );
    }

    #[test]
    fn test_no_stress_relief_in_inactive_sanctuary() {
        let mut app = App::new();
        app.add_systems(Update, apply_sanctuary_stress_relief);

        let zone = app.world_mut().spawn(SanctuaryZone { active: false }).id();
        let pop = app
            .world_mut()
            .spawn((Stress { value: 50.0 }, InZone { zone_entity: zone }))
            .id();

        app.update();

        let stress = app.world().get::<Stress>(pop).unwrap();
        assert_eq!(
            stress.value, 50.0,
            "Pop should not lose stress in an inactive sanctuary"
        );
    }
}
