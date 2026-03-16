import re

with open('src/layer1/gravitational_debt.rs', 'r') as f:
    content = f.read()

content = content.replace(
'''        app.add_plugins(bevy_time::TimePlugin);
        app.update();
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs(1));''',
'''        app.insert_resource(crate::shared::time::SimulationTime::default());'''
)

content = content.replace(
'''    // Accumulated debt should likely just tick per simulation tick.
    // In SCALE, SimulationTime ticks are the standard unit.
    let delta = 1.0; // 1 tick = 1 unit of time''',
'''    let delta = 1.0;'''
)

content = content.replace(
'''time: Res<SimulationTime>,''',
'''_time: Res<SimulationTime>,'''
)

with open('src/layer1/gravitational_debt.rs', 'w') as f:
    f.write(content)
