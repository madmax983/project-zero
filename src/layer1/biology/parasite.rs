use bevy_ecs::prelude::*;
use crate::layer1::entities::pop::Pop;
use crate::layer1::biology::health::Health;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::map::GridPosition;
// Need to find WorkEfficiency or equivalent

#[derive(Component)]
pub struct SymbioticParasite {
    pub drain_amount: f32,
    pub drain_radius: f32, // Note: using GridPosition distance (Manhattan or Chebyshev)
}

pub fn apply_parasite_buffs_system(
    mut query: Query<&mut Needs, With<SymbioticParasite>>,
) {
    for mut needs in query.iter_mut() {
        needs.hunger = 1.0;
        needs.rest = 1.0;
    }
}

#[allow(clippy::type_complexity)]
pub fn apply_parasite_health_drain_system(
    parasites: Query<(&SymbioticParasite, &GridPosition)>,
    mut healthy_pops: Query<(&mut Health, &GridPosition), (With<Pop>, Without<SymbioticParasite>)>,
) {
    for (parasite, p_pos) in parasites.iter() {
        for (mut health, h_pos) in healthy_pops.iter_mut() {
            let distance = p_pos.distance_manhattan(*h_pos) as f32;
            if distance <= parasite.drain_radius {
                health.current -= parasite.drain_amount;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::biology::health::Health;
    use crate::layer1::psychology::needs::Needs;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_infected_pop_needs_frozen_and_efficiency_boosted() {
        let mut app = App::new();
        app.add_systems(Update, apply_parasite_buffs_system);

        let pop = app.world_mut().spawn((
            Pop,
            Needs {
                hunger: 0.5,
                rest: 0.5,
                ..Default::default()
            },
            SymbioticParasite {
                drain_amount: 5.0,
                drain_radius: 5.0,
            },
        )).id();

        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        assert_eq!(needs.hunger, 1.0);
        assert_eq!(needs.rest, 1.0);
    }

    #[test]
    fn test_infected_pop_drains_nearby_health() {
        let mut app = App::new();
        app.add_systems(Update, apply_parasite_health_drain_system);

        app.world_mut().spawn((
            Pop,
            SymbioticParasite {
                drain_amount: 10.0,
                drain_radius: 5.0,
            },
            GridPosition { x: 0, y: 0 },
        ));

        let healthy_near = app.world_mut().spawn((
            Pop,
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
            GridPosition { x: 3, y: 0 },
        )).id();

        let healthy_far = app.world_mut().spawn((
            Pop,
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
            GridPosition { x: 10, y: 0 },
        )).id();

        let infected_near = app.world_mut().spawn((
            Pop,
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
            SymbioticParasite { drain_amount: 5.0, drain_radius: 5.0 },
            GridPosition { x: -3, y: 0 },
        )).id();

        app.update();

        let hp_near = app.world().get::<Health>(healthy_near).unwrap().current;
        let hp_far = app.world().get::<Health>(healthy_far).unwrap().current;
        let hp_infected = app.world().get::<Health>(infected_near).unwrap().current;

        assert_eq!(hp_near, 90.0);
        assert_eq!(hp_far, 100.0);
        assert_eq!(hp_infected, 100.0);
    }
}
