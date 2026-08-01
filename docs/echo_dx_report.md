# 🗣️ Echo: Getting Started example is broken

## 🤦 The Confusion
I copied the **Oral Tradition (Nova Feature)** code snippet from `README.md` into my `main.rs` file. When I ran `cargo run --features nova`, the Rust compiler exploded:
`error: expected one of \`!\` or \`[\`, found \`#\``

## 🕵️ The Reality
Turns out the example code starts with `# #![cfg(feature = "nova")]`. The `# ` prefix is `rustdoc` syntax to hide lines in documentation tests. But as a new user, I literally copy-pasted the whole block. The Rust compiler doesn't understand `# #![cfg...]` in a regular `main.rs` file, so it completely breaks.

## 💡 The Fix
Change the `# #![cfg(feature = "nova")]` line in the README to a standard comment like `// #![cfg(feature = "nova")]` or just remove it entirely so it compiles cleanly when copy-pasted. "Simple" is better than "Powerful."
