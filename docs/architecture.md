# SCALE Architecture

This document maps the high-level architecture of the SCALE project.

## High-Level Components

```mermaid
C4Component
title Component Diagram - SCALE System Architecture

Container(Main, "Main Entry", "Rust/Crossterm", "Initializes World, runs Game Loop")

Container_Boundary(Simulation, "Simulation Core") {
    Component(Layer1, "Layer 1", "Planetary Sim", "Terrain, Pops, Buildings")
}

Container(Shared, "Shared Lib", "Utilities", "GameState, Time, Input, Logs")

Container(UI, "UI Layer", "Ratatui", "Rendering Logic, Widgets")

Rel(Main, Shared, "Uses")
Rel(Main, Layer1, "Runs Systems")
Rel(Main, UI, "Calls Render")

Rel(Layer1, Shared, "Depends on")

Rel(UI, Shared, "Reads State")
Rel(UI, Layer1, "Reads Entities")
```

## The Game Loop

SCALE uses a hybrid architecture: `bevy_ecs` for logic and `ratatui` for rendering, managed by a custom loop.

```mermaid
sequenceDiagram
    participant User
    participant Main
    participant Input as InputRouter
    participant ECS as Bevy World
    participant Render as TUI Render

    loop Every Frame
        User->>Main: Key Press (Event)
        Main->>Input: route(key)
        Input->>ECS: Update Resources/Components

        opt Simulation Tick
            Main->>ECS: run_schedule()
            ECS->>ECS: Systems Update (Produce Food, Move, etc.)
        end

        Main->>Render: render(world, frame)
        Render->>ECS: Query Entities (Read-Only)
        ECS-->>Render: Entity Data
        Render-->>User: Draw to Terminal
    end
```

## Related Decisions

- [ADR 001: Layered Architecture](./adr/001-layered-architecture.md)
- [ADR 002: ECS-TUI Hybrid](./adr/002-ecs-tui-hybrid.md)
- [ADR 003: YAGNI - Excision of Layers 2 and 3](./adr/003-yagni-excision-of-layers-2-and-3.md)
