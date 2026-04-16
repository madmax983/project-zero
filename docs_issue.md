# 🗣️ Echo: Getting Started example is broken

* 🤦 **The Confusion:** "Tried to run the Oral Tradition example from the README. I copy-pasted it exactly into my new `main.rs`, ran `cargo run --features nova`, and it immediately yelled at me: `warning: unexpected cfg condition value: nova`. And then it printed 'Please enable the nova feature to run this example' anyway, even though I did!"
* 🕵️ **The Reality:** "Turns out the example code puts `#[cfg(feature = "nova")]` inside the `main.rs` file. This means I have to add a `nova` feature to *my own app's* `Cargo.toml` just to compile the example for your library, which makes no sense."
* 💡 **The Fix:** "Remove the `#[cfg(feature = "nova")]` and `#[cfg(not(feature = "nova"))]` conditional blocks from the README's example code entirely. Just show the actual usage."
