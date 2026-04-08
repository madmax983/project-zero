Title: 🗣️ Echo: Getting Started example is broken

🤦 **The Confusion:**
Tried to run `cargo test --doc` and it immediately exploded on `src/layer1/physics/pressure.rs` ("this function takes 3 arguments but 1 argument was supplied"). Then I tried the "README Run" for the NarrativeGenerator. I literally copy-pasted the example into a fresh `main.rs` and it failed because `NarrativeGenerator` isn't found and I don't have `anyhow`. I also tried the Oral Tradition example, and it also failed to compile because I didn't know I had to add the `nova` feature, it just says `use scale::prelude::*;`!
Finally, the "Error Check" missing `YEAR` is pretty good ("Missing required context variable or fragment: 'YEAR'"), but the broken examples stopped me from getting there.

🕵️ **The Reality:**
The doctest in `pressure.rs` is out of date. The README examples don't explicitly tell the user to add the dependencies into `Cargo.toml` in the code block itself, and the Oral Tradition example doesn't scream at me to enable the `nova` feature *before* the code block so I don't miss it when copy-pasting.

💡 **The Fix:**
- Add a huge banner in README saying 'REQUIRES FEATURE NOVA' for the Oral Tradition section.
- Put the required `Cargo.toml` dependencies (`scale = "0.1"`, `anyhow = "1.0"`) directly above or inside the example code blocks so idiots like me don't forget them.
- Fix the `pressure.rs` doctest so `cargo test --doc` works out-of-the-box.
