use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::social::morale::{MoodModifier, Morale};
use bevy::prelude::*;

#[derive(Component)]
pub struct EmpathicInfection;

#[allow(clippy::type_complexity)]
pub fn process_empathic_resonance(
    mut infected_query: Query<(&Transform, &mut Morale), (With<Pop>, With<EmpathicInfection>)>,
    others_query: Query<(&Transform, &Morale, &Needs), (With<Pop>, Without<EmpathicInfection>)>,
) {
    for (infected_transform, mut infected_morale) in infected_query.iter_mut() {
        let mut resonance_penalty = 0.0;

        for (other_transform, other_morale, other_needs) in others_query.iter() {
            let distance = infected_transform
                .translation
                .distance(other_transform.translation);
            // In our data model: hunger < 0.2 means starving (high hunger in spec terms), rest < 0.2 is exhausted
            if distance < 5.0
                && (other_morale.value < 0.5 || other_needs.hunger < 0.2 || other_needs.rest < 0.2)
            {
                resonance_penalty += 0.2;
            }
        }

        if resonance_penalty > 0.0 {
            infected_morale.add_modifier(MoodModifier {
                label: "Empathic Resonance".to_string(),
                value: -resonance_penalty,
                duration: 1, // temporary modifier per tick
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empathic_plague_spreads_negative_mood() {
        let mut world = World::new();

        let mut infected_morale = Morale::default();
        infected_morale.value = 1.0;

        // Setup infected pop
        let infected_pop = world
            .spawn((
                Pop,
                Transform::from_xyz(0.0, 0.0, 0.0),
                EmpathicInfection,
                infected_morale,
            ))
            .id();

        let mut miserable_morale = Morale::default();
        miserable_morale.value = 0.2; // miserable
        let mut miserable_needs = Needs::default();
        miserable_needs.hunger = 0.1; // starving

        // Setup nearby miserable pop
        world.spawn((
            Pop,
            Transform::from_xyz(1.0, 0.0, 0.0),
            miserable_needs,
            miserable_morale,
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(process_empathic_resonance);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(infected_pop).unwrap();

        // The infected pop should have a negative modifier added, reducing its value below 1.0
        assert!(
            morale.modifiers.iter().any(|m| m.value < 0.0),
            "Infected pop should have a negative mood modifier added"
        );
    }

    #[test]
    fn test_empathic_plague_no_effect_from_happy_pop() {
        let mut world = World::new();

        let mut infected_morale = Morale::default();
        infected_morale.value = 1.0;

        // Setup infected pop
        let infected_pop = world
            .spawn((
                Pop,
                Transform::from_xyz(0.0, 0.0, 0.0),
                EmpathicInfection,
                infected_morale,
            ))
            .id();

        let mut happy_morale = Morale::default();
        happy_morale.value = 0.8; // happy
        let mut happy_needs = Needs::default();
        happy_needs.hunger = 0.9; // full

        // Setup nearby happy pop
        world.spawn((
            Pop,
            Transform::from_xyz(1.0, 0.0, 0.0),
            happy_needs,
            happy_morale,
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(process_empathic_resonance);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(infected_pop).unwrap();

        // The infected pop should NOT have a negative modifier added
        assert!(
            !morale.modifiers.iter().any(|m| m.value < 0.0),
            "Infected pop should NOT have a negative mood modifier added"
        );
    }
}
