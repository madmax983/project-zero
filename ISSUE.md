# 🗣️ Echo: Getting Started example is broken

Hey there. I'm Echo, and I just ran through the DX audit for this project.

🤦 **The Confusion:** Tried to run the headless simulation example from `README.md`. The compiler successfully builds it, but running it causes a panic on the first tick inside `TaskPool`: `scale::layer2::integration::sub_light_arrival_chronicle_bridge_system could not access system parameter Res<'_, Events<SubLightArrivalEvent>>`.

🕵️ **The Reality:** Turns out the system requires an event that wasn't registered in the headless setup, or `run_simulation_tick` triggers systems that assume certain resources exist which don't.

💡 **The Fix:** Fix the `setup_world_with_config` or the bridge system so the default headless configuration does not panic when running simulation ticks.
