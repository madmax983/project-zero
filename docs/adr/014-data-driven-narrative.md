# 14. Data-Driven Narrative Generation

Date: 2024-05-23

## Status

Accepted

## Context

The simulation generates a high volume of events—births, deaths, construction milestones, seasonal changes, and social interactions. Originally, these events were logged using hardcoded strings or simple `format!` macros (e.g., `format!("{} has died of starvation.", pop.name)`).

This approach has several significant drawbacks:
1.  **Repetition:** Players see the exact same sentence structure repeatedly, leading to "text fatigue" and reducing immersion.
2.  **Inflexibility:** Adding variety requires modifying Rust code and recompiling.
3.  **Coupling:** Simulation logic is tightly coupled with presentation (text).
4.  **Localization:** Hardcoded strings are difficult to translate or adapt to different cultural contexts.
5.  **Lack of Depth:** Simple formatting cannot easily handle complex grammatical rules or context-sensitive flavor text (e.g., describing a death differently based on the pop's traits or the season).

We need a system that allows for rich, varied, and context-aware storytelling that can be authored independently of the codebase.

## Decision

We will implement a **Grammar-Based Narrative Generator** (`NarrativeGenerator`) driven by external data files.

### 1. Template-Fragment Architecture
The system uses two core concepts:
*   **Templates:** Define the structure of a sentence or paragraph with placeholders (slots).
    *   Example: `"[ACTOR] [VERB_EATING] a [FOOD_ITEM] with great gusto."`
*   **Fragments:** Collections of possible values for a specific slot.
    *   Example `VERB_EATING`: `["devoured", "nibbled", "wolfed down"]`
    *   Example `FOOD_ITEM`: `["turnip", "loaf of bread", "mystery meat"]`

### 2. Context Injection
Simulation systems provide a `NarrativeContext` containing key-value pairs derived from the current game state (e.g., `ACTOR` = "Bob", `LOCATION` = "The Tavern"). The generator uses this context to fill specific slots, falling back to random fragments for others.

### 3. File-Based Authoring
Lore content (templates and fragments) is stored in Markdown files (`lore/TEMPLATES.md`, `lore/FRAGMENTS.md`). This allows designers and writers to contribute to the narrative depth without touching Rust code. The Markdown format is chosen for its readability and ease of editing.

### 4. Determinism
The generator uses the simulation's `Rng` (Random Number Generator) where appropriate to ensure that narrative generation is deterministic given the same seed and sequence of events, preserving save/load consistency.

## Consequences

### Positive
*   **Variety:** A single event type can have dozens or hundreds of variations, keeping the chronicle fresh.
*   **Separation of Concerns:** Simulation logic emits *intent* (e.g., "FoodEaten"), while the narrative engine determines *presentation*.
*   **Extensibility:** New lore can be added by simply editing text files. Modders can easily add their own narrative packs.
*   **Context Sensitivity:** The system can select templates based on tags or specific context values (future expansion).

### Negative
*   **Runtime Overhead:** Constructing strings from templates and fragments is computationally more expensive than static strings.
*   **Validation Complexity:** There is no compile-time check to ensure a template's slots (e.g., `[WEAPON]`) have corresponding fragment definitions. We must implement runtime validation or comprehensive tests (like `tests/lore_integrity.rs`) to catch missing keys.
*   **Parser Maintenance:** We must maintain a custom parser for the Markdown-based lore format.
