1.  **Create Integration Bridge System**
    - Create a bridge system `nostalgia_tavern_bridge_system` in `src/layer1/core/integration.rs` to emit `RumorSpreadEvent`.
    - Command: `cat << 'INNER_EOF' >> src/layer1/core/integration.rs

use crate::layer1::culture::nostalgia::{Nostalgia, Rumor, RumorSpreadEvent};
use crate::layer1::social::Tavern;
use rand::Rng;

pub fn nostalgia_tavern_bridge_system(
    tavern_query: Query<&Tavern>,
    nostalgia_query: Query<(), With<Nostalgia>>,
    mut spread_events: EventWriter<RumorSpreadEvent>,
) {
    let mut rng = rand::thread_rng();
    for tavern in tavern_query.iter() {
        if tavern.visitors.len() >= 2 {
            for &speaker in &tavern.visitors {
                if nostalgia_query.get(speaker).is_ok() {
                    for &listener in &tavern.visitors {
                        if speaker != listener && nostalgia_query.get(listener).is_err() {
                            if rng.gen_bool(0.1) { // 10% chance to proselytize
                                spread_events.send(RumorSpreadEvent {
                                    source: speaker,
                                    target: listener,
                                    rumor: Rumor::PastGlory,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}
INNER_EOF`
    - Verify with `tail -n 25 src/layer1/core/integration.rs`.

2.  **Register `RumorSpreadEvent` and Systems**
    - I'll edit `src/setup.rs` to register `Events<RumorSpreadEvent>`.
    - Command: `cat << 'INNER_EOF' > fix_setup.py
import re
with open("src/setup.rs", "r") as f:
    content = f.read()
target = "world.init_resource::<Events<PopBorn>>();"
replacement = target + "\n    world.init_resource::<Events<crate::layer1::culture::nostalgia::RumorSpreadEvent>>();"
content = content.replace(target, replacement)
with open("src/setup.rs", "w") as f:
    f.write(content)
INNER_EOF
python3 fix_setup.py
`
    - I'll edit `src/layer1/systems/observation.rs` to register the three systems.
    - Command: `cat << 'INNER_EOF' > fix_observation.py
import re
with open("src/layer1/systems/observation.rs", "r") as f:
    content = f.read()
target = "crate::layer1::core::integration::society_suspicion_bridge_system,"
replacement = target + """
            crate::layer1::culture::nostalgia::nostalgia_trigger_system,
            crate::layer1::core::integration::nostalgia_tavern_bridge_system,
            crate::layer1::culture::nostalgia::nostalgia_spread_system,
"""
content = content.replace(target, replacement)
with open("src/layer1/systems/observation.rs", "w") as f:
    f.write(content)
INNER_EOF
python3 fix_observation.py
`
    - Verify via `grep "RumorSpreadEvent" src/setup.rs` and `grep "nostalgia_trigger_system" src/layer1/systems/observation.rs`.

3.  **Write Tests**
    - Write an integration test to verify the `nostalgia_tavern_bridge_system`.
    - To properly assert the events sent by the bridge system without relying on `reader.read(events)`, I will use a test system that listens to the events and increments a counter in a test `Resource`.
    - Command: `cat << 'INNER_EOF' > tests/integration/nostalgia_plague_bridge.rs
use bevy::prelude::*;
use scale::layer1::pop::Pop;
use scale::layer1::social::Tavern;
use scale::layer1::culture::nostalgia::{Nostalgia, RumorSpreadEvent};
use scale::layer1::core::integration::nostalgia_tavern_bridge_system;

#[derive(Resource, Default)]
struct EventCounter {
    count: usize,
}

fn count_rumor_events(mut events: EventReader<RumorSpreadEvent>, mut counter: ResMut<EventCounter>) {
    for _ in events.read() {
        counter.count += 1;
    }
}

#[test]
fn test_nostalgia_tavern_bridge_emits_rumor() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, (nostalgia_tavern_bridge_system, count_rumor_events).chain());
    app.add_event::<RumorSpreadEvent>();
    app.init_resource::<EventCounter>();

    let mut tavern = Tavern::default();

    let nostalgic_pop = app.world_mut().spawn((Pop, Nostalgia)).id();
    let normal_pop = app.world_mut().spawn(Pop).id();

    tavern.visitors.push(nostalgic_pop);
    tavern.visitors.push(normal_pop);

    app.world_mut().spawn(tavern);

    // Run multiple times to trigger the 10% chance
    for _ in 0..100 {
        app.update();
    }

    let counter = app.world().resource::<EventCounter>();
    assert!(counter.count > 0, "Should emit RumorSpreadEvent when socializing with nostalgic pop");
}
INNER_EOF`
    - Command: `echo "pub mod nostalgia_plague_bridge;" >> tests/integration/mod.rs`
    - Verify `ls -l tests/integration/nostalgia_plague_bridge.rs` and `tail -n 1 tests/integration/mod.rs`.

4.  **Run Tests**
    - Command: `cargo test` to ensure all tests pass and changes didn't break anything.

5.  **Complete Pre-commit steps**
    - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
