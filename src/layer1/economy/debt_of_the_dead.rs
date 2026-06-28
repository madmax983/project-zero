use crate::layer1::economy::{ColonyPrices, Wallet};
use crate::layer1::entities::pop::PopDied;
use crate::layer1::social::morale::{MoodModifier, Morale};
use crate::layer1::social::Relationships;
use bevy_ecs::prelude::*;

#[derive(Resource, Default, Debug)]
pub struct SocializedDebt {
    pub total_debt: f32,
}

#[derive(Event)]
pub struct DebtInheritedEvent {
    pub pop_name: String,
    pub amount: f32,
}

#[derive(Event)]
pub struct DebtSocializedEvent {
    pub pop_name: String,
    pub amount: f32,
}

#[allow(clippy::type_complexity)]
pub fn process_debt_of_the_dead_system(
    mut events: EventReader<PopDied>,
    relationships: Query<&Relationships>,
    mut wallets_and_morale: ParamSet<(Query<&Wallet>, Query<(&mut Wallet, &mut Morale)>)>,
    mut socialized_debt: ResMut<SocializedDebt>,
    mut inherited_events: EventWriter<DebtInheritedEvent>,
    mut socialized_events: EventWriter<DebtSocializedEvent>,
) {
    for event in events.read() {
        let mut debt_amount = 0.0;
        if let Ok(wallet) = wallets_and_morale.p0().get(event.entity) {
            if wallet.credits < 0.0 {
                debt_amount = wallet.credits.abs();
            }
        }

        if debt_amount > 0.0 {
            let mut closest_relative: Option<Entity> = None;
            let mut max_affinity = f32::MIN;

            if let Ok(rels) = relationships.get(event.entity) {
                for (&rel_entity, &affinity) in &rels.affinities {
                    if affinity > max_affinity {
                        max_affinity = affinity;
                        closest_relative = Some(rel_entity);
                    }
                }
            }

            if let Some(relative_entity) = closest_relative {
                if let Ok((mut rel_wallet, mut rel_morale)) =
                    wallets_and_morale.p1().get_mut(relative_entity)
                {
                    rel_wallet.credits -= debt_amount;
                    rel_morale.add_modifier(MoodModifier {
                        label: "Inherited Burden".to_string(),
                        value: -20.0,
                        duration: 100, // Arbitrary duration
                    });
                    inherited_events.send(DebtInheritedEvent {
                        pop_name: event.name.clone(),
                        amount: debt_amount,
                    });
                } else {
                    // Relative exists in affinities but is not valid (e.g. dead), socialize it
                    socialized_debt.total_debt += debt_amount;
                    socialized_events.send(DebtSocializedEvent {
                        pop_name: event.name.clone(),
                        amount: debt_amount,
                    });
                }
            } else {
                socialized_debt.total_debt += debt_amount;
                socialized_events.send(DebtSocializedEvent {
                    pop_name: event.name.clone(),
                    amount: debt_amount,
                });
            }
        }
    }
}

pub fn apply_socialized_debt_system(
    socialized_debt: Res<SocializedDebt>,
    mut prices: ResMut<ColonyPrices>,
) {
    if socialized_debt.is_changed() {
        let debt_factor = socialized_debt.total_debt / 1000.0;
        prices.food_price = 1.0 + debt_factor;
        prices.luxury_price = 5.0 + (debt_factor * 2.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(ColonyPrices::default());
        world.insert_resource(SocializedDebt::default());
        world.init_resource::<Events<PopDied>>();
        world.init_resource::<Events<DebtInheritedEvent>>();
        world.init_resource::<Events<DebtSocializedEvent>>();
        world
    }

    #[test]
    fn test_dead_pop_debt_transferred_to_relative() {
        let mut world = setup_world();

        let relative = world
            .spawn((Pop, Wallet { credits: 10.0 }, Morale::default()))
            .id();

        let dead_pop = world
            .spawn((
                Pop,
                Wallet { credits: -50.0 },
                Relationships {
                    affinities: vec![(relative, 50.0)].into_iter().collect(),
                },
            ))
            .id();

        world.send_event(PopDied {
            entity: dead_pop,
            name: "Debtor".to_string(),
            tick: 1,
            reason: "Old age".to_string(),
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_debt_of_the_dead_system);
        schedule.run(&mut world);

        let relative_wallet = world.get::<Wallet>(relative).unwrap();
        assert_eq!(relative_wallet.credits, -40.0); // 10 - 50 = -40

        let relative_morale = world.get::<Morale>(relative).unwrap();
        assert!(
            relative_morale
                .modifiers
                .iter()
                .any(|m| m.label == "Inherited Burden"),
            "Relative should receive Inherited Burden morale penalty"
        );
    }

    #[test]
    fn test_dead_pop_debt_socialized_if_no_relatives() {
        let mut world = setup_world();

        let dead_pop = world
            .spawn((
                Pop,
                Wallet { credits: -100.0 },
                Relationships {
                    affinities: std::collections::HashMap::new(),
                },
            ))
            .id();

        world.send_event(PopDied {
            entity: dead_pop,
            name: "Loner Debtor".to_string(),
            tick: 1,
            reason: "Starvation".to_string(),
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_debt_of_the_dead_system);
        schedule.run(&mut world);

        let socialized_debt = world.resource::<SocializedDebt>();
        assert_eq!(socialized_debt.total_debt, 100.0);
    }

    #[test]
    fn test_socialized_debt_increases_prices() {
        let mut world = setup_world();
        world.resource_mut::<SocializedDebt>().total_debt = 1000.0;

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_socialized_debt_system);
        schedule.run(&mut world);

        let prices = world.resource::<ColonyPrices>();
        assert!(prices.food_price > 1.0, "Food price should be increased");
        assert!(
            prices.luxury_price > 5.0,
            "Luxury price should be increased"
        );
    }
}
