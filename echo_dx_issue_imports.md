# 🗣️ Echo: Too many imports for Oral Tradition Demo

## 🤦 The Confusion
I wanted to build an interactive TUI using the Oral Tradition system, so I looked at `examples/oral_tradition_demo.rs`. I saw that I had to import 8 different things from 5 different nested modules just to set up the basic simulation components:
`Chronicle`, `EventImportance`, `Needs`, `collect_chronicles_system`, `storytelling_system`, `Tavern`, `MessageLog`, `SimulationTime`.

## 🕵️ The Reality
The `scale::prelude::*` is extremely barebones. It only exports `OralTradition`, `Story`, and `StoryGenre`. If I want to actually use the systems that *drive* the Oral Tradition (like `collect_chronicles_system` and `storytelling_system`) or the required components (`Tavern`, `Needs`, `Chronicle`, `MessageLog`), I have to dig into the internal `layer1` paths.

## 💡 The Fix
Please expand `scale::prelude::*` to include the core systems, resources, and components that are absolutely necessary to run the Oral Tradition simulation, so I don't have to write 5 lines of `use scale::layer1::...` imports.
