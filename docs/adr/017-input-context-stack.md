# 17. Input Context Stack

Date: 2024-05-22

## Status

Accepted

## Context

In a complex TUI game like SCALE, input handling must support modal interactions (Main Menu, Normal Gameplay, Build Mode, Designations, Overlay/Chronicle). A monolithic `InputRouter` struct or hardcoded `match` statement quickly becomes unmanageable and prone to bugs where keys leak through modes.

## Decision

We adopt a **Stack-Based Input Context** pattern:

1.  **Resource**: `InputContextStack` (Resource) holds a `Vec<InputContext>`.
2.  **Enum**: `InputContext` defines the modes (e.g., `Normal`, `BuildMode`, `Overlay`).
3.  **Routing**: The top of the stack is the *active* context. Input is routed *only* to the handler for that context.
4.  **Transitions**: Systems push a new context to enter a mode (e.g., press 'b' -> push `BuildMode`) and pop to exit (e.g., press 'Esc' -> pop).

## Consequences

### Positive
*   **Clarity**: Each mode's input logic is isolated in its own function.
*   **Modality**: Overlays naturally block input to lower layers (e.g., opening Chronicle pauses game and captures all keys).
*   **Extensibility**: Adding a new mode (e.g., "Research Screen") just requires a new Enum variant and handler function.

### Negative
*   **State Sync**: The Input Stack is separate from other game states (like `GameState::Paused`). Developers must ensure pushing `Overlay` also pauses the simulation if desired.
*   **UX Complexity**: Deeply nested stacks can confuse users if visual feedback isn't clear about which mode is active.
