1. **Claim work**: Use `echo` to append `- [ ] \`INT-877\` Integration: Debt of the dead -> Chronicle — claimed 2026-10-31` to `design/IN_PROGRESS.md`. Then commit using `git commit -am "claim: INT-877 debt-of-the-dead-to-chronicle integration"`.
2. **Create integration test**: Create `tests/integration/debt_of_the_dead_bridge.rs` by executing the following bash block:
```bash
cat << 'EOF' > tests/integration/debt_of_the_dead_bridge.rs
use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::economy::debt_of_the_dead::{
    DebtInheritedEvent, DebtSocializedEvent, SocializedDebt,
};
use scale::layer1::economy::Wallet;
use scale::layer1::entities::pop::{Pop, PopDied};
use scale::layer1::social::morale::Morale;
use scale::layer1::social::Relationships;

#[test]
fn test_debt_socialized_chronicle() {
    let mut app = bevy_app::App::new();
    app.add_event::<PopDied>();
    app.add_event::<DebtInheritedEvent>();
    app.add_event::<DebtSocializedEvent>();
    app.add_event::<AddChronicleEvent>();
    app.insert_resource(SocializedDebt::default());

    app.add_systems(
        bevy_app::Update,
        (
            scale::layer1::economy::debt_of_the_dead::process_debt_of_the_dead_system,
            scale::layer1::core::integration::debt_of_the_dead_chronicle_bridge,
        )
            .chain(),
    );

    let dead_pop = app
        .world_mut()
        .spawn((
            Pop,
            Wallet { credits: -100.0 },
            Relationships {
                affinities: std::collections::HashMap::new(),
            },
        ))
        .id();

    app.world_mut().send_event(PopDied {
        entity: dead_pop,
        name: "Loner Debtor".to_string(),
        tick: 1,
        reason: "Starvation".to_string(),
    });

    app.update();

    let chronicle_events = app
        .world()
        .resource::<Events<AddChronicleEvent>>()
        .get_reader()
        .read(app.world().resource::<Events<AddChronicleEvent>>())
        .collect::<Vec<_>>();

    assert_eq!(chronicle_events.len(), 1);
    assert!(chronicle_events[0]
        .text
        .contains("Loner Debtor died, leaving 100 credits of debt"));
}

#[test]
fn test_debt_inherited_chronicle() {
    let mut app = bevy_app::App::new();
    app.add_event::<PopDied>();
    app.add_event::<DebtInheritedEvent>();
    app.add_event::<DebtSocializedEvent>();
    app.add_event::<AddChronicleEvent>();
    app.insert_resource(SocializedDebt::default());

    app.add_systems(
        bevy_app::Update,
        (
            scale::layer1::economy::debt_of_the_dead::process_debt_of_the_dead_system,
            scale::layer1::core::integration::debt_of_the_dead_chronicle_bridge,
        )
            .chain(),
    );

    let relative = app
        .world_mut()
        .spawn((Pop, Wallet { credits: 10.0 }, Morale::default()))
        .id();

    let dead_pop = app
        .world_mut()
        .spawn((
            Pop,
            Wallet { credits: -50.0 },
            Relationships {
                affinities: vec![(relative, 50.0)].into_iter().collect(),
            },
        ))
        .id();

    app.world_mut().send_event(PopDied {
        entity: dead_pop,
        name: "Debtor".to_string(),
        tick: 1,
        reason: "Old age".to_string(),
    });

    app.update();

    let chronicle_events = app
        .world()
        .resource::<Events<AddChronicleEvent>>()
        .get_reader()
        .read(app.world().resource::<Events<AddChronicleEvent>>())
        .collect::<Vec<_>>();

    assert_eq!(chronicle_events.len(), 1);
    assert!(chronicle_events[0]
        .text
        .contains("Debtor died. 50 credits of debt inherited"));
}
EOF
```
Then verify it using `cat tests/integration/debt_of_the_dead_bridge.rs`.
3. **Register test**: Add `pub mod debt_of_the_dead_bridge;` to `tests/integration/mod.rs` by running `echo "pub mod debt_of_the_dead_bridge;" >> tests/integration/mod.rs`. Then run `cat tests/integration/mod.rs` to verify.
4. **Modify debt system**: I will use a python script to modify `src/layer1/economy/debt_of_the_dead.rs` to add `DebtInheritedEvent` and `DebtSocializedEvent` and emit them. The script will be:
```bash
cat << 'EOF' > update_debt.py
with open("src/layer1/economy/debt_of_the_dead.rs", "r") as f:
    code = f.read()

search1 = """pub struct SocializedDebt {
    pub total_debt: f32,
}

#[allow(clippy::type_complexity)]
pub fn process_debt_of_the_dead_system(
    mut events: EventReader<PopDied>,
    relationships: Query<&Relationships>,
    mut wallets_and_morale: ParamSet<(Query<&Wallet>, Query<(&mut Wallet, &mut Morale)>)>,
    mut socialized_debt: ResMut<SocializedDebt>,
) {
    for event in events.read() {
        let mut debt_amount = 0.0;"""

replace1 = """pub struct SocializedDebt {
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
        let mut debt_amount = 0.0;"""

search2 = """                    rel_morale.add_modifier(MoodModifier {
                        label: "Inherited Burden".to_string(),
                        value: -20.0,
                        duration: 100, // Arbitrary duration
                    });
                } else {
                    // Relative exists in affinities but is not valid (e.g. dead), socialize it
                    socialized_debt.total_debt += debt_amount;
                }
            } else {
                socialized_debt.total_debt += debt_amount;
            }"""

replace2 = """                    rel_morale.add_modifier(MoodModifier {
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
            }"""

code = code.replace(search1, replace1)
code = code.replace(search2, replace2)

with open("src/layer1/economy/debt_of_the_dead.rs", "w") as f:
    f.write(code)
EOF
python update_debt.py
```
Use `cat` and `git diff` to verify the changes.
5. **Write glue code**: Modify `src/layer1/core/integration.rs` using a python script to add `debt_of_the_dead_chronicle_bridge`:
```bash
cat << 'EOF' > update_integration.py
with open("src/layer1/core/integration.rs", "r") as f:
    code = f.read()

search = """/// INT-642: Bridges the construction of a Simulacrum to AddChronicleEvent (Chronicle)."""

replace = """/// INT-877: Bridges the debt of the dead system to AddChronicleEvent (Chronicle).
pub fn debt_of_the_dead_chronicle_bridge(
    mut inherited_events: bevy_ecs::prelude::EventReader<
        crate::layer1::economy::debt_of_the_dead::DebtInheritedEvent,
    >,
    mut socialized_events: bevy_ecs::prelude::EventReader<
        crate::layer1::economy::debt_of_the_dead::DebtSocializedEvent,
    >,
    mut chronicle_events: bevy_ecs::prelude::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
) {
    for event in inherited_events.read() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            importance: crate::layer1::core::chronicle::EventImportance::Minor,
            text: format!(
                "{} died. {} credits of debt inherited by next of kin.",
                event.pop_name, event.amount
            ),
        });
    }

    for event in socialized_events.read() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            importance: crate::layer1::core::chronicle::EventImportance::Standard,
            text: format!(
                "{} died, leaving {} credits of debt to be socialized by the colony.",
                event.pop_name, event.amount
            ),
        });
    }
}

/// INT-642: Bridges the construction of a Simulacrum to AddChronicleEvent (Chronicle)."""

code = code.replace(search, replace)

with open("src/layer1/core/integration.rs", "w") as f:
    f.write(code)
EOF
python update_integration.py
```
Use `cat` and `git diff` to verify.
6. **Register events and systems**: Modify `src/simulation.rs`, `src/layer1/systems/observation.rs`, and `src/layer1/systems/cleanup.rs` using a python script. We checked the source of `src/simulation.rs`, `src/layer1/systems/observation.rs`, and `src/layer1/systems/cleanup.rs` to make sure these string replacements match actual anchors. `src/setup.rs` doesn't need to be modified as we can just add the new events using python string replacement in `src/simulation.rs`.
```bash
cat << 'EOF' > update_registers.py
with open("src/simulation.rs", "r") as f:
    code = f.read()
code = code.replace("        world.init_resource::<Events<crate::layer1::economy::resources::ResourceMinedEvent>>();", "        world.init_resource::<Events<crate::layer1::economy::resources::ResourceMinedEvent>>();\n        world.init_resource::<Events<crate::layer1::economy::debt_of_the_dead::DebtInheritedEvent>>();\n        world.init_resource::<Events<crate::layer1::economy::debt_of_the_dead::DebtSocializedEvent>>();")
with open("src/simulation.rs", "w") as f:
    f.write(code)

with open("src/layer1/systems/observation.rs", "r") as f:
    code = f.read()
code = code.replace("            crate::layer1::core::integration::impact_warning_chronicle_bridge,", "            crate::layer1::core::integration::impact_warning_chronicle_bridge,\n            crate::layer1::core::integration::debt_of_the_dead_chronicle_bridge,")
with open("src/layer1/systems/observation.rs", "w") as f:
    f.write(code)

with open("src/layer1/systems/cleanup.rs", "r") as f:
    code = f.read()
code = code.replace("            update_event_buffer::<crate::layer1::environment::impact::ImpactStrikeEvent>,", "            update_event_buffer::<crate::layer1::environment::impact::ImpactStrikeEvent>,\n            update_event_buffer::<crate::layer1::economy::debt_of_the_dead::DebtInheritedEvent>,\n            update_event_buffer::<crate::layer1::economy::debt_of_the_dead::DebtSocializedEvent>,")
with open("src/layer1/systems/cleanup.rs", "w") as f:
    f.write(code)
EOF
python update_registers.py
```
Verify changes with `git diff`.
7. **Run tests**: Execute `cargo test` and `cargo clippy -- -D warnings` to verify all tests pass and there are no warnings.
8. **Update documentation**: Use python to update `design/SEAM_MAP.md` and `design/COMPLETED.md` with:
```bash
cat << 'EOF' > update_docs.py
import re

with open('design/IN_PROGRESS.md', 'r') as f:
    in_progress = f.read()
in_progress = re.sub(r'- \[ \] `INT-877` Integration: Debt of the dead -> Chronicle — claimed \d{4}-\d{2}-\d{2}\n', '', in_progress)
with open('design/IN_PROGRESS.md', 'w') as f:
    f.write(in_progress)

with open('design/COMPLETED.md', 'r') as f:
    completed = f.read()
completed += "- [x] `INT-877` Integration: Debt of the Dead -> Chronicle — completed 2026-10-31\n"
with open('design/COMPLETED.md', 'w') as f:
    f.write(completed)

with open('design/SEAM_MAP.md', 'r') as f:
    seam_map = f.read()
seam_map += """
### INT-877: Debt of the Dead -> Chronicle
- **Date:** 2026-10-31
- **Systems connected:** `process_debt_of_the_dead_system` -> `debt_of_the_dead_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** Added `DebtInheritedEvent` and `DebtSocializedEvent` to `src/layer1/economy/debt_of_the_dead.rs`. Added `debt_of_the_dead_chronicle_bridge` in `src/layer1/core/integration.rs` to generate Chronicle records when debt is inherited or socialized.
- **Schedule:** Registered the new events in `src/simulation.rs`, the bridge system in `src/layer1/systems/observation.rs`, and the event buffer updates in `src/layer1/systems/cleanup.rs`.
- **Tests:** `tests/integration/debt_of_the_dead_bridge.rs` (2 tests)
"""
with open('design/SEAM_MAP.md', 'w') as f:
    f.write(seam_map)
EOF
python update_docs.py
```
Verify changes with `git diff`.
9. **Final Test Run**: Run `cargo test` to verify all tests pass after documentation updates.
10. **Pre-commit step**: Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
11. **Commit code**: Commit the code using `git commit -am "$(cat <<'EOF'
feat(integration): connect debt of the dead to chronicle

INT-877: Bridges the debt of the dead system with the chronicle system.

Glue added:
- DebtInheritedEvent and DebtSocializedEvent are emitted by process_debt_of_the_dead_system.
- debt_of_the_dead_chronicle_bridge in src/layer1/core/integration.rs listens to these events and generates Chronicle records.

Integration tests: 2 passing
All unit tests still passing.

Co-Authored-By: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>
EOF
)"`.
