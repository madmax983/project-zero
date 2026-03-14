# 🗣️ Echo: Getting Started example is broken

## 1. 🔍 EXPERIENCE - The Walkthrough
- **Scenario:** "I am a new user trying to run the SCALE quick start examples."
- **Action:** Tried running a default `cargo build` and copy-pasted the quick start examples from `README.md`.

## 2. 🚧 STUMBLE - The Friction Points
- 🤦 **The Confusion:** "Tried to build the project without any specific feature flags. The compiler spat out a gigantic trait bound error saying `the method in_set exists for unit type (), but its trait bounds were not satisfied` in `observation.rs`."
- "I just want to run the code, why do I have to decipher Bevy ECS trait bound errors before I even write a single line?"

## 3. 📢 REPORT - The Complaint
- 🕵️ **The Reality:** "Turns out the default build (which doesn't include the `nova` feature) compiles to an empty tuple `()` in `schedule.add_systems` inside `src/layer1/systems/observation.rs`. Bevy 0.15 doesn't implement `IntoSystemConfigs` for an empty tuple."
- 💡 **The Fix:** "Move the `#[cfg(feature = "nova")]` macro outside of the `schedule.add_systems()` block entirely, so the compiler doesn't try to register an empty tuple when the feature is disabled."

## 4. 🧪 VERIFY - The "idiot proofing"
- "After the fix, `cargo build` and `cargo run --bin scale --features native` compile correctly out of the box. The narrative and headless examples in the README also compile and run beautifully without feature flags."
