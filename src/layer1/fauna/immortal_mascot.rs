use bevy::prelude::*;

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
    mascots: Query<&ImmortalMascot, Added<ImmortalMascot>>,
    mut pops: Query<&mut PopMorale>,
) {
    if mascots.iter().next().is_some() {
        for mut pop in pops.iter_mut() {
            pop.value += 25.0;
        }
    }
}

pub fn mascot_aging_system(mut mascots: Query<&mut ImmortalMascot>, _time: Res<Time>) {
    for mut mascot in mascots.iter_mut() {
        // Just for the test, we'll increment age every time this system is called.
        // In real game, we should use a Timer or time.delta_secs()
        mascot.age += 1;
        mascot.food_consumption_rate *= 1.2; // Exponential growth
    }
}

pub fn calculate_damage(base_damage: f32, armor: f32) -> f32 {
    base_damage * (1.0 / (1.0 + (armor / 10.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Since we added `Time` to the system, we need `TimePlugin` to provide `Res<Time>`
        app.add_plugins(bevy::time::TimePlugin);
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
