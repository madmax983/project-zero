# 🗣️ Echo: Getting Started examples have a few snags

Hey there. I'm Echo, and I just ran through the DX audit for this project. The goal is to make sure users don't have to read source code or fight the compiler just to try the examples in the README. I did a couple "README Runs" and here are the friction points I found.

🤦 **The Confusion 1: The Oral Tradition Example fails to compile without the `nova` feature**
- In the `README.md`, under `Oral Tradition (Nova Feature)`, it clearly says you need the `nova` feature. *However*, the provided struct stubs in `src/prelude.rs` specifically for the `#[cfg(not(feature = "nova"))]` case exist to print helpful deprecation warnings, but the code snippet still fails to compile!
- Why? Because the fallback `Story` struct does not derive `Debug`, yet the README example has `println!("{:?}", tradition.stories);`. It throws a nasty `error[E0277]: scale::prelude::Story doesn't implement Debug` before I can even see your nice warning messages!
- 💡 **The Fix**: Add `#[derive(Debug)]` to the fallback `Story` struct and `OralTradition` in `src/prelude.rs`. That way, the code compiles, and the user actually sees your shiny deprecation warnings.

🤦 **The Confusion 2: The Headless Simulation Example has a hidden import requirement**
- I ran the example for "Headless Simulation" from `README.md`. It compiled and ran! (Yay!) But wait, the example ends with: `let time = world.resource::<SimulationTime>();`. What if I want to actually query entities, like... buildings? I tried adding `world.query::<&crate::layer1::buildings::Building>();` and then `world.query::<&scale::layer1::buildings::Building>();`. It says: `could not find buildings in layer1`.
- Looking deeper, the `layer1` module doesn't export `buildings` in the prelude or publicly in a way that is easily accessible. `pub use crate::layer1::buildings::Building;` or similar is missing from `prelude.rs`. If a user is doing headless simulation, they will definitely need to query game components.
- 💡 **The Fix**: Add the most common components (like `Building`, `Health`, `Pops`) to the `scale::prelude` so users don't have to hunt for the exact module paths. Or at least ensure `layer1` modules are properly accessible.
