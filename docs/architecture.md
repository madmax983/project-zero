# Architecture

This document describes the high-level architecture of the **Scale** project.

## System Context (C4 Level 1)

The system is a standalone terminal application.

```mermaid
C4Context
    title System Context Diagram for Scale

    Person(player, "Player", "Plays the game via terminal")
    System(scale, "Scale", "4X Colony Simulation Game")
    System_Ext(terminal, "Terminal Emulator", "Renders text and captures input (Crossterm)")

    Rel(player, terminal, "Inputs commands / Views TUI")
    Rel(terminal, scale, "Sends Input Events / Displays Frames")
```

## Container Diagram (C4 Level 2)

The application is a single executable composed of the Simulation Engine and the TUI Layer.

```mermaid
C4Container
    title Container Diagram for Scale

    Container(main, "Main Loop", "Rust", "Orchestrates Input, Update, Render loop")

    Container_Boundary(logic, "Simulation Logic") {
        Component(ecs, "Bevy ECS", "Rust Crate", "Entity Component System")
        Component(layers, "Simulation Layers", "Rust Modules", "Layer 1, 2, 3 logic")
    }

    Container_Boundary(ui, "User Interface") {
        Component(tui, "Ratatui", "Rust Crate", "Terminal UI Rendering")
        Component(crossterm, "Crossterm", "Rust Crate", "Terminal backend")
    }

    Rel(main, ecs, "Ticks Schedule")
    Rel(main, tui, "Draws Frame")
    Rel(layers, ecs, "Defines Systems & Components")
    Rel(tui, ecs, "Reads World State")
```

## Module Structure (Class Diagram)

This diagram shows the static code organization.

```mermaid
classDiagram
    direction TB

    class Binary_Main {
        +main()
        +run_app()
    }

    class Library_Lib {
        +mod layer1
        +mod layer2
        +mod layer3
        +mod ui
        +mod shared
    }

    class Layer1 {
        +TerrainGrid
        +Viewport
        +generate_terrain()
    }

    class Layer2 {
        %% System scale logic
    }

    class Layer3 {
        %% Galaxy scale logic
    }

    class BevyECS {
        <<External>>
        +World
        +Schedule
    }

    class Ratatui {
        <<External>>
        +Terminal
        +Frame
    }

    Binary_Main --> Library_Lib : Uses
    Binary_Main --> BevyECS : Owns
    Binary_Main --> Ratatui : Owns

    Library_Lib *-- Layer1
    Library_Lib *-- Layer2
    Library_Lib *-- Layer3

    Layer1 ..> BevyECS : Define Resources
    Binary_Main ..> Layer1 : Inserts Resources
```
