# 🗣️ Echo: Getting Started examples have a few snags

Hey there. I'm Echo, and I just ran through the DX audit for this project. The goal is to make sure users don't have to read source code or fight the compiler just to try the examples in the README. I did a couple "README Runs" and here are the friction points I found.

🤦 **The Confusion 1: The Oral Tradition Example fails to compile without the `nova` feature**
- In the `README.md`, under `Oral Tradition (Nova Feature)`, it clearly says you need the `nova` feature. So I tried running it *without* the `nova` feature just to see what happens.
- It throws a nasty `error[E0277]: scale::prelude::Story doesn't implement Debug` before I can even see any helpful warning messages!
- 💡 **The Fix**: Please make sure the code still compiles without the feature so I can actually see the deprecation warnings, or make the error message clearer.

🤦 **The Confusion 2: The Headless Simulation Example has a hidden import requirement**
- I ran the example for "Headless Simulation" from `README.md`. It compiled and ran! (Yay!) But wait, the example ends with: `let time = world.resource::<SimulationTime>();`. What if I want to actually query entities, like... buildings? I tried adding `world.query::<&crate::layer1::buildings::Building>();` and then `world.query::<&scale::layer1::buildings::Building>();`. It says: `could not find buildings in layer1`.
- 💡 **The Fix**: Add the most common components (like `Building`, `Health`, `Pops`) to the `scale::prelude` so users don't have to hunt for the exact module paths. Or update the README to show exactly how to import and query a building!

🤦 **The Confusion 3: Scary warning on the base Narrative feature**
- Under `Procedural Generation (Narrative)`, there's a giant warning: `> # 🚨 ⚠️ REQUIRES FEATURE NOVA ⚠️ 🚨`. But the text literally right below it says: "This is the base narrative system... It is available in the default build." So I ran it without the `nova` feature and it worked fine.
- 💡 **The Fix**: Remove the scary `REQUIRES FEATURE NOVA` warning from the `Procedural Generation (Narrative)` section. It's confusing and incorrect!
