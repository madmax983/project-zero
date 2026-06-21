use crate::layer1::administration::edicts::ColonyPolicies;
use crate::layer1::administration::edicts::Policy;
use crate::layer1::entities::pop::Pop;
use crate::layer1::psychology::memory::{Memories, MemoryType};
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy::prelude::*;
use rand::seq::SliceRandom;

pub fn execute_lottery_system(
    mut commands: Commands,
    mut policies: ResMut<ColonyPolicies>,
    pop_query: Query<Entity, With<Pop>>,
    mut memory_query: Query<&mut Memories>,
    mut traits_query: Query<&mut Traits>,
    time: Res<Time>,
) {
    if policies.active_policies.contains(&Policy::TheLottery) {
        let mut all_pops: Vec<Entity> = pop_query.iter().collect();

        if all_pops.is_empty() {
            policies
                .active_policies
                .retain(|p| *p != Policy::TheLottery);
            return;
        }

        let sacrifice_count = std::cmp::min(1, all_pops.len());

        let mut rng = rand::thread_rng();
        all_pops.shuffle(&mut rng);

        for victim in all_pops.iter().take(sacrifice_count) {
            commands.entity(*victim).despawn_recursive();
        }

        for survivor in all_pops.iter().skip(sacrifice_count) {
            if let Ok(mut memories) = memory_query.get_mut(*survivor) {
                memories.add(MemoryType::StarvationTrauma, time.elapsed_secs_f64() as u64);
            }

            if let Ok(mut traits) = traits_query.get_mut(*survivor) {
                traits.add(Trait::Traumatized);
            }
        }

        policies
            .active_policies
            .retain(|p| *p != Policy::TheLottery);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::administration::edicts::ColonyPolicies;
    use crate::layer1::administration::edicts::Policy;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::psychology::memory::Memories;
    use crate::layer1::psychology::traits::Traits;
    use std::collections::HashSet;

    #[test]
    fn test_lottery_edict_sacrifices_pops() {
        let mut app = App::new();
        app.add_plugins(bevy_time::TimePlugin);
        let mut set = HashSet::new();
        set.insert(Policy::TheLottery);
        app.insert_resource(ColonyPolicies {
            active_policies: set,
            ..Default::default()
        });
        app.add_systems(Update, execute_lottery_system);

        let _pop1 = app.world_mut().spawn(Pop).id();
        let _pop2 = app.world_mut().spawn(Pop).id();

        app.update();

        let mut pops_alive = 0;
        let mut query = app.world_mut().query::<&Pop>();
        for _ in query.iter(app.world()) {
            pops_alive += 1;
        }
        assert_eq!(pops_alive, 1, "The Lottery should sacrifice 1 pop.");

        let policies = app.world().resource::<ColonyPolicies>();
        assert!(
            !policies.active_policies.contains(&Policy::TheLottery),
            "The Lottery edict should be removed after execution."
        );
    }

    #[test]
    fn test_surviving_pops_receive_survivors_guilt_trauma() {
        let mut app = App::new();
        app.add_plugins(bevy_time::TimePlugin);
        let mut set = HashSet::new();
        set.insert(Policy::TheLottery);
        app.insert_resource(ColonyPolicies {
            active_policies: set,
            ..Default::default()
        });
        app.add_systems(Update, execute_lottery_system);

        let _pop1 = app
            .world_mut()
            .spawn((Pop, Memories::default(), Traits::default()))
            .id();
        let _pop2 = app
            .world_mut()
            .spawn((Pop, Memories::default(), Traits::default()))
            .id();

        app.update();

        let mut found_trauma = false;
        let mut query = app.world_mut().query::<(&Memories, &Traits)>();
        for (_memories, traits) in query.iter(app.world()) {
            if traits.has(Trait::Traumatized) {
                found_trauma = true;
            }
        }

        assert!(
            found_trauma,
            "Surviving Pops should receive the Survivor's Guilt trauma."
        );
    }
}
