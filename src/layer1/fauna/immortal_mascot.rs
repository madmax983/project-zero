use crate::layer1::psychology::needs::Needs;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct PopMorale {
    pub value: f32,
}

#[derive(Component)]
pub struct ImmortalMascot {
    pub age: u32,
    pub food_consumption_rate: f32,
    pub armor_rating: f32,
}

pub fn apply_mascot_morale_system(
    mascot_query: Query<&ImmortalMascot>,
    mut pop_query: Query<(Option<&mut PopMorale>, Option<&mut Needs>)>,
) {
    if mascot_query.is_empty() {
        return;
    }

    for (pop_morale_opt, needs_opt) in pop_query.iter_mut() {
        if let Some(mut pop_morale) = pop_morale_opt {
            pop_morale.value = 75.0;
        }
        if let Some(mut needs) = needs_opt {
            needs.leisure = (needs.leisure + 0.1).min(1.0);
        }
    }
}

pub fn mascot_aging_system(
    mut query: Query<&mut ImmortalMascot>,
    _time: Option<Res<SimulationTime>>,
) {
    for mut mascot in query.iter_mut() {
        mascot.age += 1;
        // Exponential growth: rate = 1.0 * (1.15 ^ age)
        mascot.food_consumption_rate = 1.0 * 1.15_f32.powi(mascot.age as i32);

        // Cap the consumption rate to prevent f32::INFINITY
        if mascot.food_consumption_rate > 9000.0 {
            mascot.food_consumption_rate = 9000.0;
        }
    }
}

pub fn calculate_damage(damage: f32, armor: f32) -> f32 {
    // extreme mitigation: high armor > 99% reduction
    damage * (10.0 / (10.0 + armor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_mascot_provides_global_morale_boost() {
        let mut app = App::new();
        app.add_systems(Update, apply_mascot_morale_system);

        let pop = app.world_mut().spawn(PopMorale { value: 50.0 }).id();

        app.update();
        assert_eq!(app.world().get::<PopMorale>(pop).unwrap().value, 50.0);

        // Spawn the mascot
        app.world_mut().spawn(ImmortalMascot {
            age: 0,
            food_consumption_rate: 1.0,
            armor_rating: 1000.0,
        });

        app.update();
        assert_eq!(
            app.world().get::<PopMorale>(pop).unwrap().value,
            75.0,
            "Mascot should boost pop morale"
        );
    }

    #[test]
    fn test_mascot_food_consumption_increases_exponentially() {
        let mut app = App::new();
        app.add_systems(Update, mascot_aging_system);

        let mascot = app
            .world_mut()
            .spawn(ImmortalMascot {
                age: 0,
                food_consumption_rate: 1.0,
                armor_rating: 1000.0,
            })
            .id();

        app.update(); // age = 1
        let rate_1 = app
            .world()
            .get::<ImmortalMascot>(mascot)
            .unwrap()
            .food_consumption_rate;
        assert!(rate_1 > 1.0);

        app.update(); // age = 2
        let rate_2 = app
            .world()
            .get::<ImmortalMascot>(mascot)
            .unwrap()
            .food_consumption_rate;
        assert!(rate_2 > rate_1 * 1.1, "Growth must be at least exponential");
    }

    #[test]
    fn test_mascot_has_extreme_armor() {
        let mascot = ImmortalMascot {
            age: 0,
            food_consumption_rate: 1.0,
            armor_rating: 1000.0,
        };

        // Hypothetical damage calculation
        let damage_taken = calculate_damage(100.0, mascot.armor_rating);
        assert!(
            damage_taken < 1.0,
            "Mascot should be virtually immune to normal damage"
        );
    }
}
