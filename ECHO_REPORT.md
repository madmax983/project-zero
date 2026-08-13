# 🗣️ Echo: Getting Started DX Audit

I followed the instructions in `README.md` exactly as written and found several friction points for new users trying to get started with the Narrative and Nova features.

## 🤦 The Confusion: CFG Feature Attribute
In the "Oral Tradition (Nova Feature)" section, the Rust example includes `#![cfg(feature = "nova")]`. If a user copy-pastes this exact block into their `main.rs`, it will fail to compile with `error: expected one of \`!\` or \`[\`, found \`#\``.

## 🕵️ The Reality
The markdown `# #![cfg(feature = "nova")]` is meant to be a rustdoc hidden line (`#`), but it is being displayed as literal text or the user includes the `#` character in their copy-paste, breaking compilation.

## 💡 The Fix
Remove the `# #![cfg(feature = "nova")]` line from the visible ````rust` block in `README.md`. The warning banner is sufficient for telling users to enable the feature.

## 🤦 The Confusion: Empty Fragment Error
When the narrative generator encounters a fragment that has no options, it prints a clear and actionable error message: `Fragment '...' has no options defined.`. However, when an optional fragment (`[UNKNOWN?]`) is missing entirely, it fails silently by outputting an empty string. This is good for optional slots, but could lead to debugging friction if the user expected a fallback. (This is a minor note, the `NarrativeError` format is very readable!)
