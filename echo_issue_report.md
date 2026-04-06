Title: 🗣️ Echo: Getting Started example is broken and Doctests Fail

🤦 **The Confusion:**
Tried to run the `cargo test --doc` as part of exploring the repo, but it completely failed on `src/layer1/physics/pressure.rs` (error: "this function takes 3 arguments but 1 argument was supplied"). Also, the README's `Oral Tradition (Nova Feature)` example gives confusing compiler errors ("cannot find struct... Story in this scope") if you just copy-paste it into a new project without realizing you have to manually configure your `Cargo.toml` with `features = ["nova"]`.

🕵️ **The Reality:**
The doctest in `pressure.rs` is out of date and breaks the out-of-the-box test experience. The `README.md` *does* mention enabling the `nova` feature, but it doesn't show the `Cargo.toml` line clearly right before the code block, meaning impatient users will just copy the Rust code, compile it, see it fail, and give up.

💡 **The Fix:**
1. Fix the doctest in `src/layer1/physics/pressure.rs` so it passes and doesn't scare off new users.
2. Add a clear `// In Cargo.toml: scale = { version = "...", features = ["nova"] }` comment *inside* or immediately before the Rust code block in the README so copy-pasters see it.

---

Title: 🗣️ Echo: Getting Started example is broken and Doctests Fail

🤦 **The Confusion:**
Tried to run the `cargo test --doc` as part of exploring the repo, but it completely failed on `src/layer1/physics/pressure.rs` (error: "this function takes 3 arguments but 1 argument was supplied"). Also, the README's `Oral Tradition (Nova Feature)` example gives confusing compiler errors ("cannot find struct... Story in this scope") if you just copy-paste it into a new project without realizing you have to manually configure your `Cargo.toml` with `features = ["nova"]`.

🕵️ **The Reality:**
The doctest in `pressure.rs` is out of date and breaks the out-of-the-box test experience. The `README.md` *does* mention enabling the `nova` feature, but it doesn't show the `Cargo.toml` line clearly right before the code block, meaning impatient users will just copy the Rust code, compile it, see it fail, and give up.

💡 **The Fix:**
1. Fix the doctest in `src/layer1/physics/pressure.rs` so it passes and doesn't scare off new users.
2. Add a clear `// In Cargo.toml: scale = { version = "...", features = ["nova"] }` comment *inside* or immediately before the Rust code block in the README so copy-pasters see it.

---
Title: 🗣️ Echo: Getting Started example is broken and Doctests Fail

🤦 **The Confusion:**
Tried to run the `cargo test --doc` as part of exploring the repo, but it completely failed on `src/layer1/physics/pressure.rs` (error: "this function takes 3 arguments but 1 argument was supplied"). Also, the README's `Oral Tradition (Nova Feature)` example gives confusing compiler errors ("cannot find struct... Story in this scope") if you just copy-paste it into a new project without realizing you have to manually configure your `Cargo.toml` with `features = ["nova"]`.

🕵️ **The Reality:**
The doctest in `pressure.rs` is out of date and breaks the out-of-the-box test experience. The `README.md` *does* mention enabling the `nova` feature, but it doesn't show the `Cargo.toml` line clearly right before the code block, meaning impatient users will just copy the Rust code, compile it, see it fail, and give up.

💡 **The Fix:**
1. Fix the doctest in `src/layer1/physics/pressure.rs` so it passes and doesn't scare off new users.
2. Add a clear `// In Cargo.toml: scale = { version = "...", features = ["nova"] }` comment *inside* or immediately before the Rust code block in the README so copy-pasters see it.

---

Title: 🗣️ Echo: Getting Started example is broken and Doctests Fail

🤦 **The Confusion:**
Tried to run the `cargo test --doc` as part of exploring the repo, but it completely failed on `src/layer1/physics/pressure.rs` (error: "this function takes 3 arguments but 1 argument was supplied"). Also, the README's `Oral Tradition (Nova Feature)` example gives confusing compiler errors ("cannot find struct... Story in this scope") if you just copy-paste it into a new project without realizing you have to manually configure your `Cargo.toml` with `features = ["nova"]`.

🕵️ **The Reality:**
The doctest in `pressure.rs` is out of date and breaks the out-of-the-box test experience. The `README.md` *does* mention enabling the `nova` feature, but it doesn't show the `Cargo.toml` line clearly right before the code block, meaning impatient users will just copy the Rust code, compile it, see it fail, and give up.

💡 **The Fix:**
1. Fix the doctest in `src/layer1/physics/pressure.rs` so it passes and doesn't scare off new users.
2. Add a clear `// In Cargo.toml: scale = { version = "...", features = ["nova"] }` comment *inside* or immediately before the Rust code block in the README so copy-pasters see it.
