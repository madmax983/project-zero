# SCALE Architecture

This document maps the high-level architecture of the SCALE project.

## High-Level Components

```mermaid
C4Component
title Component Diagram - SCALE System Architecture

Container(Main, "Main Entry", "Rust/Crossterm", "Initializes World, runs Game Loop")

Container_Boundary(Simulation, "Simulation Core (Layer 1)") {
    Component(UtilityOrchestrator, "Utility Orchestrator", "utility_ai.rs", "Coordinates Decision Cycle")
    Component(GPU, "GPU Compute", "gpu/evaluate.rs", "Parallel Scoring")
    Component(Actions, "Action Modules", "layer1/actions/*.rs", "Generic Actions")
    Component(DomainActions, "Domain Actions", "medical.rs, funeral.rs", "Specific Logic")

    Component(Pops, "Pops", "pop.rs", "Agents with Needs & Thoughts")
    Component(CabinFever, "Cabin Fever System", "cabin_fever.rs", "Tracks Confinement & Crowding")
    Component(World, "World Entities", "farm.rs, housing.rs", "Interactable Buildings")
    Component(Resources, "Colony Resources", "resources.rs", "Global Inventory")
    Component(Map, "Map/Terrain", "map.rs", "Spatial Grid")
    Component(Integration, "Integration Bridges", "integration.rs", "Cross-Domain Logic")
}

Container(Shared, "Shared Lib", "Utilities", "GameState, Time, Input, Logs")

Container_Boundary(UI, "UI Layer") {
    Component(MapRender, "Map Module", "map.rs", "Renders Grid & Entities")
    Component(Inspector, "Inspector Module", "inspector.rs", "Renders Details")
    Component(Chronicle, "Chronicle Module", "chronicle.rs", "Renders Logs")
    Component(Status, "Status Module", "status.rs", "Renders Top Bar")
}

Rel(Main, Shared, "Uses")
Rel(Main, UtilityOrchestrator, "Runs Systems")
Rel(UtilityOrchestrator, GPU, "Dispatches Work")
Rel(Main, MapRender, "Calls Render")

Rel(UtilityOrchestrator, Actions, "Calls evaluate_*")
Rel(UtilityOrchestrator, DomainActions, "Calls evaluate_*")
Rel(UtilityOrchestrator, Pops, "Reads/Writes")
Rel(UtilityOrchestrator, World, "Queries Availability")
Rel(UtilityOrchestrator, Map, "Calculates Distance")

Rel(Pops, World, "Interacts with")
Rel(Pops, Resources, "Consumes/Produces")
Rel(Pops, CabinFever, "Accumulates Stress")

Rel(Integration, Pops, "Updates Needs/Health")
Rel(Integration, World, "Reacts to Events")
Rel(Integration, Resources, "Pollution Bridge")

Rel(MapRender, Shared, "Reads State")
Rel(MapRender, Map, "Reads Entities")
Rel(Inspector, Shared, "Reads Selection")
Rel(Inspector, Pops, "Reads Components")
```

## Integration Bridge Pattern

To avoid circular dependencies, cross-domain logic is centralized.

```mermaid
sequenceDiagram
    participant Source as Source Domain
    participant Bridge as Bridge System
    participant Target as Target Domain
    participant Events as Event Queue

    Note over Source, Target: Example: Fire affecting Health

    Source->>Events: Emit Event (e.g. FireSpread)

    loop Integration Phase
        Bridge->>Events: Read Events
        Bridge->>Source: Query Context (Fire Intensity)
        Bridge->>Target: Apply Effect (Take Damage)
    end
```

## Core Dependencies

```mermaid
classDiagram
  class Core
  class Storage
  Core --> Storage : Uses (Trait Bound)
  %% Removed the circular dependency arrow
```

## The Game Loop

SCALE uses a hybrid architecture: `bevy_ecs` for logic and `ratatui` for rendering, managed by a custom loop.

```mermaid
sequenceDiagram
    participant User
    participant Main
    participant Input as InputRouter
    participant ECS as Bevy World
    participant UI as TUI Layer

    loop Every Frame
        User->>Main: Key Press (Event)
        Main->>Input: route(key)
        Input->>ECS: Update Resources/Components

        opt Simulation Tick
            Main->>ECS: run_schedule()
            ECS->>ECS: Systems Update (Utility AI, Refining, etc.)
        end

        Main->>UI: render(world, frame)

        rect rgb(30, 30, 30)
            note right of UI: UI Rendering Phase
            UI->>ECS: Query Selection
            UI->>ECS: Query Map/Entities
            ECS-->>UI: Data
            UI-->>User: Draw Widgets
        end
    end
```

## Utility AI Decision Loop

The "Brain" of the simulation. Pops decide what to do based on internal needs and external context.

```mermaid
sequenceDiagram
    participant System as evaluate_actions_system
    participant GPU as GPU Compute
    participant Pop as Pop Entity
    participant Needs as Needs Component
    participant World as World State
    participant Memory as Utility Weights

    loop Every Tick (Staggered)
        alt GPU Enabled
            System->>World: Query/Extract Ready Pops
            System->>GPU: Upload Input Buffers
            GPU->>GPU: Parallel Scoring (Shader)
            GPU-->>System: Return Decisions
            System->>Pop: Update PopAction (Batch)
        else CPU Fallback
            System->>Pop: Check Commitment Timer
            alt Timer Expired
                System->>Needs: Read Hunger/Rest
                Needs-->>System: Urgency Scores

                rect rgb(40, 40, 50)
                    Note right of System: Evaluation Phase
                    System->>World: Query Entities (Farms, Items, etc.)

                    System->>System: Call evaluate_satisfy_hunger()
                    System->>System: Call evaluate_work()
                    System->>System: Call evaluate_haul()

                    Note over System: Actions typically return Option<(Score, Target)>
                    System->>System: Select Best Utility
                end

                System->>Pop: Update PopAction (Best Score)
            else Timer Active
                System->>Pop: Continue Current Action
            end
        end
    end
```

## UI Inspector Flow

The Inspector pattern allows detailed viewing of entities without coupling the UI to specific entity types.

```mermaid
sequenceDiagram
    participant User
    participant Input
    participant Selection as Selection Resource
    participant Inspector as UI Inspector
    participant World as ECS World

    User->>Input: Click / Select Entity
    Input->>Selection: Update Target (EntityID)

    Note over Inspector, World: Render Phase
    Inspector->>Selection: Read Target
    alt Target is Entity
        Inspector->>World: Get Components (Pop, Needs, Thoughts)
        World-->>Inspector: Component Data
        Inspector->>User: Render Detail Panels
    else Target is Tile
        Inspector->>World: Get Tile Data
        Inspector->>User: Render Tile Info
    end
```

## Platform Abstraction

SCALE supports both native (terminal) and web (browser) execution through a platform abstraction layer.

```mermaid
C4Component
title Component Diagram - Platform Abstraction

Container_Boundary(EntryPoints, "Entry Points") {
    Component(Main, "Native Binary", "src/main.rs", "Uses Crossterm Backend")
    Component(Wasm, "WASM Binary", "src/bin/wasm_app.rs", "Uses Ratzilla DOM Backend")
}

Container_Boundary(Platform, "Platform Abstraction") {
    Component(Events, "Event Types", "src/platform/mod.rs", "GameKeyEvent, GameMouseEvent")
    Component(NativeImpl, "Native Impl", "src/platform/native.rs", "From<CrosstermEvent>")
    Component(WasmImpl, "WASM Impl", "src/platform/wasm.rs", "From<WebEvent>")
}

Container(Shared, "Shared Core", "src/lib.rs", "Game Loop, Simulation, Rendering")

Rel(Main, NativeImpl, "Uses")
Rel(Wasm, WasmImpl, "Uses")
Rel(Main, Shared, "Runs")
Rel(Wasm, Shared, "Runs")
Rel(Shared, Events, "Consumes")
```

## Related Decisions

- [ADR 006: WASM Browser Support](./adr/006-wasm-browser-support.md)
- [ADR 001: Layered Architecture](./adr/001-layered-architecture.md)
- [ADR 002: ECS-TUI Hybrid](./adr/002-ecs-tui-hybrid.md)
- [ADR 003: YAGNI - Excision of Layers 2 and 3](./adr/003-yagni-excision-of-layers-2-and-3.md)
- [ADR 004: Modular UI Architecture](./adr/004-modular-ui-architecture.md)
- [ADR 005: Adopt Emergent Utility AI](./adr/005-adopt-emergent-utility-ai.md)
- [ADR 008: Modular Utility AI Structure](./adr/008-modular-utility-ai.md)
- [ADR 012: Decouple Storage from Core](./adr/012-decouple-storage-from-core.md)
- [ADR 013: GPU Accelerated Utility AI](./adr/013-gpu-accelerated-utility-ai.md)
- [ADR 014: Cabin Fever Mechanics](./adr/014-cabin-fever-mechanics.md)
- [ADR 015: Centralized Integration Bridges](./adr/015-centralized-integration-bridges.md)
