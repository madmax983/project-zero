# 🗣️ Echo: Getting Started narrative example requires anyhow

## 🤦 The Confusion
I copy-pasted the **Procedural Generation (Narrative)** code block from `README.md` directly into a fresh `main.rs`, hit `cargo run`, and the compiler blew up. It said:
`use of undeclared crate or module 'anyhow'`

## 🕵️ The Reality
Turns out the example code sets the `main` function to return `anyhow::Result<()>` and uses the `?` operator. Even though there are commented-out lines above the code saying `// anyhow = "1.0"`, I just copied the Rust block and expected it to run. Why should I have to install an error handling crate just to print out a generated story in the basic "Getting Started" example?

## 💡 The Fix
Either:
1. Change the `main` function in the README to `fn main() { ... }` and use `.unwrap()` or `.expect("Failed to generate story")` instead of `?` so it compiles with zero extra dependencies.
2. Add a huge, unmissable banner saying **MUST RUN `cargo add anyhow`** before running the example.
