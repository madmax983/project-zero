# 883: Start Scenario Selection

**Layer:** Shared UI / startup
**Status:** Draft
**Complexity:** Medium

---

## 1. Overview

Add start scenario selection to the main menu flow.

The player should be able to pick one of the built-in scenarios before the game begins. The first UI can be plain: scenario name, short description, and difficulty tag are enough.

This spec does not add scenario content. It wires the selected scenario into startup using the framework from `882`.

---

## 2. Dependencies

- `026` Main Menu
- `882` Start Scenario Framework

---

## 3. Scope

- Add scenario selection state to `MenuState`
- Allow cycling between built-in scenarios
- Start game using the selected scenario
- Display the current scenario in the menu UI

---

## 4. Acceptance Criteria

- [ ] Menu state tracks a selected scenario.
- [ ] The main menu can cycle through built-in scenarios.
- [ ] Starting a game uses the selected scenario in `SetupConfig`.
- [ ] The menu shows scenario name and difficulty.
- [ ] Existing quit flow still works.

---

## 5. Notes

- Keep the current `Start Game` / `Quit` structure if possible.
- Do not build a nested menu system yet.
- The UI should stay simple enough to test through input routing.
