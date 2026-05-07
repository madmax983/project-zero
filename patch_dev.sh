cat << 'INNER_EOF' > src/layer1/culture/nostalgia.rs
use bevy_ecs::prelude::*;
use crate::layer1::lifecycle::Age;
use crate::layer1::social::morale::Morale;

#[derive(Component, Debug)]
pub struct Nostalgia;

#[derive(Event)]
pub struct RumorSpreadEvent {
    pub source: Entity,
    pub target: Entity,
    pub rumor: Rumor,
}

pub enum Rumor {
    PastGlory,
    // other rumors
}

#[allow(clippy::type_complexity)]
pub fn nostalgia_trigger_system(
    mut commands: Commands,
    query: Query<(Entity, &Age, &Morale), (With<crate::layer1::pop::Pop>, Without<Nostalgia>)>
) {
    for (entity, age, morale) in query.iter() {
        if age.ticks_alive >= 60 * crate::layer1::balance::TICKS_PER_YEAR && morale.value <= 20.0 {
            commands.entity(entity).insert(Nostalgia);
        }
    }
}

pub fn nostalgia_spread_system(
    mut commands: Commands,
    mut events: EventReader<RumorSpreadEvent>,
    query: Query<&Nostalgia>,
) {
    let mut rng = rand::thread_rng();
    use rand::Rng;

    for event in events.read() {
        match event.rumor {
            Rumor::PastGlory => {
                if query.get(event.source).is_ok() {
                    // Refactor: Add a resistance mechanic so not every rumor spread guarantees an infection.
                    if rng.gen_bool(0.25) {
                        commands.entity(event.target).insert(Nostalgia);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::lifecycle::Age;
    use crate::layer1::social::morale::Morale;

    #[test]
    fn test_low_morale_triggers_nostalgia_in_old_pops() {
        let mut app = App::new();
        app.add_systems(Update, nostalgia_trigger_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Age { ticks_alive: 65 * crate::layer1::balance::TICKS_PER_YEAR, stage: crate::layer1::lifecycle::LifeStage::Elder },
            Morale { value: 10.0, modifiers: vec![] }, // Severe morale drop
        )).id();

        app.update();

        assert!(app.world().get::<Nostalgia>(pop_entity).is_some(), "Old pop with low morale should contract Nostalgia");
    }

    #[test]
    fn test_nostalgia_spreads_via_rumors() {
        let mut app = App::new();
        app.add_event::<RumorSpreadEvent>();
        app.add_systems(Update, nostalgia_spread_system);

        let infected = app.world_mut().spawn((Pop, Nostalgia)).id();
        let target = app.world_mut().spawn((Pop, Age { ticks_alive: 30 * crate::layer1::balance::TICKS_PER_YEAR, stage: crate::layer1::lifecycle::LifeStage::Adult })).id();

        app.world_mut().send_event(RumorSpreadEvent { source: infected, target, rumor: Rumor::PastGlory });

        // Force a large number of events to guarantee the 25% chance hits at least once
        for _ in 0..50 {
            app.world_mut().send_event(RumorSpreadEvent { source: infected, target, rumor: Rumor::PastGlory });
        }

        app.update();

        assert!(app.world().get::<Nostalgia>(target).is_some(), "Nostalgia should spread to target via rumors");
    }
}
INNER_EOF

echo -e "\npub mod nostalgia;\npub use nostalgia::*;" >> src/layer1/culture/mod.rs

sed -i 's/is_silent: bool,/is_silent: bool,\n    pub is_nostalgic: bool,/' src/layer1/mind/utility_eval_types.rs
sed -i 's/is_silent: item.is_silent.is_some(),/is_silent: item.is_silent.is_some(),\n            is_nostalgic: item.nostalgia.is_some(),/' src/layer1/mind/utility_eval_types.rs
sed -i 's/is_silent: false,/is_silent: false,\n            is_nostalgic: false,/' src/layer1/mind/utility_eval_types.rs
sed -i 's/pub is_silent: Option<&'\''static crate::layer1::stress::TraitSilent>,/pub is_silent: Option<\&'\''static crate::layer1::stress::TraitSilent>,\n    pub nostalgia: Option<\&'\''static crate::layer1::culture::nostalgia::Nostalgia>,/' src/layer1/mind/utility_eval_types.rs

sed -i 's/is_silent: false,/is_silent: false,\n            is_nostalgic: false,/' src/layer1/actions/mental_break.rs

cat << 'INNER_EOF' > src/layer1/actions/simple.rs
use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Generic evaluator for simple actions (work, repair, etc.)
#[must_use]
pub(crate) fn evaluate_simple_action(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    candidates: &[ScorableCandidate],
    base_utility: f32,
) -> Option<(f32, Entity)> {
    evaluate_candidates(pop_pos, weights, candidates, base_utility)
}
INNER_EOF

sed -i 's/pub(crate) fn evaluate_research(/pub(crate) fn evaluate_research(\n    is_nostalgic: bool,/g' src/layer1/actions/research.rs
sed -i 's/if resources.knowledge >= resources.max_knowledge {/if is_nostalgic || resources.knowledge >= resources.max_knowledge {/g' src/layer1/actions/research.rs

python3 -c '
import re
content = open("src/layer1/mind/utility_ai.rs").read()
content = content.replace("""        self.evaluator.evaluate_and_consider(
            evaluate_simple_action(pop_pos, &weights, &self.buffer.work_designations, 0.5),
            ActionType::Work,
            self.context,
            work_bonus,
        );""", """
        let filtered_designations: Vec<_> = if self.data.is_nostalgic {
            self.buffer.work_designations.clone()
        } else {
            self.buffer.work_designations.clone()
        };

        let work_utility = evaluate_simple_action(pop_pos, &weights, &filtered_designations, if self.data.is_nostalgic { 0.2 } else { 0.5 });
        self.evaluator.evaluate_and_consider(
            work_utility,
            ActionType::Work,
            self.context,
            work_bonus,
        );""")
content = content.replace("""        self.evaluator.evaluate_and_consider(
            evaluate_simple_action(pop_pos, &weights, &self.buffer.refining, 0.5),
            ActionType::Refine,""", """        let refining_utility = if self.data.is_nostalgic { None } else { evaluate_simple_action(pop_pos, &weights, &self.buffer.refining, 0.5) };
        self.evaluator.evaluate_and_consider(
            refining_utility,
            ActionType::Refine,""")
content = content.replace("""        if !is_feral {
            self.evaluator.evaluate_and_consider(
                evaluate_research(
                    pop_pos,
                    &weights,
                    self.context.resources,
                    &self.buffer.libraries,
                ),
                ActionType::Research,""", """        if !is_feral {
            let research_utility = if self.data.is_nostalgic { None } else { evaluate_research(
                    self.data.is_nostalgic,
                    pop_pos,
                    &weights,
                    self.context.resources,
                    &self.buffer.libraries,
                ) };
            self.evaluator.evaluate_and_consider(
                research_utility,
                ActionType::Research,""")
open("src/layer1/mind/utility_ai.rs", "w").write(content)
'

python3 -c '
import re
content = open("src/simulation.rs").read()
if "crate::layer1::culture::nostalgia::nostalgia_trigger_system," not in content:
    content = content.replace("crate::layer1::culture::ancestral_graves::bury_in_ancestral_grave_system,", "crate::layer1::culture::nostalgia::nostalgia_trigger_system,\n            crate::layer1::culture::nostalgia::nostalgia_spread_system,\n            crate::layer1::culture::ancestral_graves::bury_in_ancestral_grave_system,")
open("src/simulation.rs", "w").write(content)
'
rm src/layer1/culture/nostalgia_cult.rs
