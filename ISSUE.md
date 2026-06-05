# 🗣️ Echo: Getting Started examples have a few snags

Hey there. I'm Echo, and I just ran through the DX audit for this project. The goal is to make sure users don't have to read source code or fight the compiler just to try the examples in the README. I did a couple "README Runs" and here are the friction points I found.

🤦 **The Confusion 1: The Oral Tradition Example fails to compile without the `nova` feature**
- In the `README.md`, under `Oral Tradition (Nova Feature)`, it clearly says you need the `nova` feature. So I tried running it *without* the `nova` feature just to see what the error looks like.
- It throws a nasty `error[E0277]: scale::prelude::Story doesn't implement Debug` because of the `println!("{:?}", tradition.stories);` line. This completely overrides whatever helpful deprecation messages you might be trying to show!
- 💡 **The Fix**: Please make `Story` implement `Debug` even when the `nova` feature is missing, so the code compiles and users can actually see the real warning/error messages.

🤦 **The Confusion 2: The Headless Simulation Example has a hidden import requirement**
- I ran the example for "Headless Simulation" from `README.md`. It compiled and ran! (Yay!) But wait, the example ends with: `let time = world.resource::<SimulationTime>();`. What if I want to actually query entities, like... buildings? I tried adding `world.query::<&Building>();`. The compiler yelled at me: `cannot find type Building in this scope`.
- I shouldn't have to guess the exact import paths for basic game components just to write a headless script.
- 💡 **The Fix**: Add the most common components (like `Building`, `Health`, `Pop`) to the `scale::prelude` so users don't have to hunt for them.

🤦 **The Confusion 3: Scary warning on the base Narrative feature**
- Under `Procedural Generation (Narrative)`, there's a giant warning: `> # 🚨 ⚠️ REQUIRES FEATURE NOVA ⚠️ 🚨`. But the text literally right below it says: "This is the base narrative system... It is available in the default build." So I ran it without the `nova` feature and it worked fine.
- 💡 **The Fix**: Remove the scary `REQUIRES FEATURE NOVA` warning and "STOP" text from the `Procedural Generation (Narrative)` section. It's totally misleading and belongs in the Oral Tradition section!
