use bevy_ecs::prelude::*;
use rand::Rng;
use crate::layer1::morale::{Morale, MoodModifier};
use crate::layer1::health::Health;

#[derive(Component, Debug, Clone)]
pub struct ZeroGArena;

#[derive(Component, Debug, Clone)]
pub struct PlayZeroGSportsAction {
    pub target_arena: Entity,
    pub opponent: Option<Entity>,
}

pub fn zero_g_sports_system(
    mut commands: Commands,
    mut query: Query<(Entity, &PlayZeroGSportsAction, &mut Morale, &mut Health)>,
) {
    let mut resolved_matches = Vec::new();

    // First pass: resolve matches and determine winners/losers
    for (entity, action, _, _) in query.iter() {
        if let Some(opponent) = action.opponent {
            // Ensure we only process each pair once
            if entity < opponent && !resolved_matches.contains(&(entity, opponent)) {
                let mut rng = rand::thread_rng();
                let winner = if rng.gen_bool(0.5) { entity } else { opponent };
                let loser = if winner == entity { opponent } else { entity };

                resolved_matches.push((winner, loser));
            }
        }
    }

    // Second pass: apply effects
    for (winner, loser) in resolved_matches {
        if let Ok((_, _, mut morale, _)) = query.get_mut(winner) {
            morale.add_modifier(MoodModifier {
                label: "Zero-G Sports Champion".to_string(),
                value: 0.3, // 30% boost (Morale max is 1.0)
                duration: 100,
            });
            commands.entity(winner).remove::<PlayZeroGSportsAction>();
        }
        if let Ok((_, _, _, mut health)) = query.get_mut(loser) {
            health.take_damage(20.0); // Injury
            commands.entity(loser).remove::<PlayZeroGSportsAction>();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use super::{ZeroGArena, PlayZeroGSportsAction, zero_g_sports_system};
    use crate::layer1::pop::Pop;
    use crate::layer1::morale::Morale;
    use crate::layer1::health::Health;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_zero_g_sports_applies_buff_and_debuff() {
        let mut world = setup_world();

        let arena = world.spawn(ZeroGArena).id();

        let pop1 = world.spawn((
            Pop,
            Morale::default(),
            Health::default(),
            PlayZeroGSportsAction { target_arena: arena, opponent: None },
        )).id();

        let pop2 = world.spawn((
            Pop,
            Morale::default(),
            Health::default(),
            PlayZeroGSportsAction { target_arena: arena, opponent: Some(pop1) },
        )).id();

        // Update opponent link
        world.get_mut::<PlayZeroGSportsAction>(pop1).unwrap().opponent = Some(pop2);

        let mut schedule = Schedule::default();
        schedule.add_systems(zero_g_sports_system);
        schedule.run(&mut world);

        // One pop should have increased morale modifiers, the other decreased health.
        let m1 = &world.get::<Morale>(pop1).unwrap().modifiers;
        let h1 = world.get::<Health>(pop1).unwrap().current;

        let m2 = &world.get::<Morale>(pop2).unwrap().modifiers;
        let h2 = world.get::<Health>(pop2).unwrap().current;

        let p1_won = !m1.is_empty() && h1 == 100.0;
        let p2_won = !m2.is_empty() && h2 == 100.0;

        assert!(p1_won || p2_won);

        if p1_won {
            assert!(h2 < 100.0);
            assert_eq!(m1[0].value, 0.3);
        } else {
            assert!(h1 < 100.0);
            assert_eq!(m2[0].value, 0.3);
        }
    }
}
