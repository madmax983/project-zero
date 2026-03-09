# 🗣️ Echo: Getting Started example is broken

## Description

🤦 **The Confusion:**
I tried to run the "Oral Tradition" feature from the `README.md` code snippet. I copy-pasted it exactly as shown. The compiler yelled at me about a missing `Resource` trait implementation for `OralTradition` and a bunch of errors from `bevy_ecs`. It just says `error[E0277]: the trait bound 'OralTradition: Resource' is not satisfied`. What gives?

🕵️ **The Reality:**
Turns out I actually need to add the `bevy_ecs` dependency to my own `Cargo.toml`. But more importantly, because the `scale` workspace uses a very specific version of `bevy_ecs` (v0.15), if I let `cargo` pick the default (v0.13), they have completely mismatched types under the hood. The `Resource` trait from 0.13 is NOT the `Resource` trait from 0.15, leading to horrible compiler mismatch errors. If I have to read the `scale` source code's `Cargo.toml` to figure out which version to use, the documentation failed!

💡 **The Fix:**
Add a comment directly to the example code's `Cargo.toml` configuration snippet in the `README.md` to explicitly state `bevy_ecs = "0.15"`.

---

## Description

🤦 **The Confusion:**
I copied the first example for "Procedural Generation (Narrative)" and ran it. It generated a sentence with a literal missing variable tag in it: `Year 2150. The Terran Dominion arise from Sol Prime. They will come to be called [ERROR: CIV_EPITHET].`. If I'm copying an example verbatim, I expect it to work without me having to debug missing context variables!

🕵️ **The Reality:**
The `CIVILIZATION_RISE` template expects a `CIV_EPITHET` variable, but the example code doesn't insert it into the `NarrativeContext` before generating the story.

💡 **The Fix:**
Add `context.insert("CIV_EPITHET", "The First Ones");` to the example in `README.md`.
